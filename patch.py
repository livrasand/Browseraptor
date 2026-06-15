import sys

with open("src/main.rs", "r") as f:
    content = f.read()

old_block = """        let cx_clone = cx.to_async();
        
        cx.background_executor()
            .spawn(async move {
                loop {
                    if let Ok(cmd) = rx.try_recv() {
                        let browsers_for_selector = browsers_for_selector.clone();
                        let config = config.clone();
                        let _ = cx_clone.update(|cx| {
                            match cmd {
                                AppCommand::ShowSelector(url) => {
                                    let url = url.unwrap_or_else(|| "https://example.com".into());
                                    let b = browsers_for_selector;
                                    cx.activate(true);
                                    let bounds = gpui::Bounds::centered(None, gpui::size(gpui::px(420.0), gpui::px(460.0)), cx);
                                    cx.open_window(
                                        gpui::WindowOptions {
                                            window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
                                            titlebar: Some(gpui::TitlebarOptions {
                                                title: Some("Browseraptor — Select Browser".into()),
                                                appears_transparent: false,
                                                traffic_light_position: None,
                                            }),
                                            ..Default::default()
                                        },
                                        |_, cx| cx.new(|_| SelectorPlaceholder(url, b)),
                                    ).ok();
                                }
                                AppCommand::ShowSettings => {
                                    let b = browsers_for_selector;
                                    let c = config;
                                    cx.activate(true);
                                    let bounds = gpui::Bounds::centered(None, gpui::size(gpui::px(600.0), gpui::px(480.0)), cx);
                                    cx.open_window(
                                        gpui::WindowOptions {
                                            window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
                                            titlebar: Some(gpui::TitlebarOptions {
                                                title: Some("Browseraptor Settings".into()),
                                                appears_transparent: false,
                                                traffic_light_position: None,
                                            }),
                                            ..Default::default()
                                        },
                                        |_, cx| cx.new(|_| ui::settings::SettingsWindow::new(b, c)),
                                    ).ok();
                                }
                                AppCommand::OpenWith(name) => {
                                    if let Some(b) = find_browser(&name, &browsers_for_selector) {
                                        let _ = crate::browser::launcher::launch(&b, "https://example.com");
                                    }
                                }
                                AppCommand::RefreshBrowsers => {
                                    tracing::info!("Refreshing browser list…");
                                }
                                AppCommand::Quit => {
                                    tracing::info!("Shutting down…");
                                    cx.quit();
                                }
                            }
                        });
                    }
                    
                    // Sleep for 100ms instead of blocking on recv()
                    cx.background_executor().timer(std::time::Duration::from_millis(100)).await;
                }
            }).detach();"""

new_block = """        let cx_clone = cx.to_async();
        let (async_tx, mut async_rx) = futures::channel::mpsc::unbounded::<AppCommand>();
        
        std::thread::spawn(move || {
            while let Ok(cmd) = rx.recv() {
                let _ = async_tx.unbounded_send(cmd);
            }
        });

        cx.spawn(|_| async move {
            use futures::StreamExt;
            while let Some(cmd) = async_rx.next().await {
                let browsers_for_selector = browsers_for_selector.clone();
                let config = config.clone();
                let _ = cx_clone.update(|cx| {
                    match cmd {
                        AppCommand::ShowSelector(url) => {
                            let url = url.unwrap_or_else(|| "https://example.com".into());
                            let b = browsers_for_selector;
                            cx.activate(true);
                            let bounds = gpui::Bounds::centered(None, gpui::size(gpui::px(420.0), gpui::px(460.0)), cx);
                            cx.open_window(
                                gpui::WindowOptions {
                                    window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
                                    titlebar: Some(gpui::TitlebarOptions {
                                        title: Some("Browseraptor — Select Browser".into()),
                                        appears_transparent: false,
                                        traffic_light_position: None,
                                    }),
                                    ..Default::default()
                                },
                                |_, cx| cx.new(|_| SelectorPlaceholder(url, b)),
                            ).ok();
                        }
                        AppCommand::ShowSettings => {
                            let b = browsers_for_selector;
                            let c = config;
                            cx.activate(true);
                            let bounds = gpui::Bounds::centered(None, gpui::size(gpui::px(600.0), gpui::px(480.0)), cx);
                            cx.open_window(
                                gpui::WindowOptions {
                                    window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
                                    titlebar: Some(gpui::TitlebarOptions {
                                        title: Some("Browseraptor Settings".into()),
                                        appears_transparent: false,
                                        traffic_light_position: None,
                                    }),
                                    ..Default::default()
                                },
                                |_, cx| cx.new(|_| ui::settings::SettingsWindow::new(b, c)),
                            ).ok();
                        }
                        AppCommand::OpenWith(name) => {
                            if let Some(b) = find_browser(&name, &browsers_for_selector) {
                                let _ = crate::browser::launcher::launch(&b, "https://example.com");
                            }
                        }
                        AppCommand::RefreshBrowsers => {
                            tracing::info!("Refreshing browser list…");
                        }
                        AppCommand::Quit => {
                            tracing::info!("Shutting down…");
                            cx.quit();
                        }
                    }
                });
            }
        }).detach();"""

if old_block in content:
    with open("src/main.rs", "w") as f:
        f.write(content.replace(old_block, new_block))
    print("Success")
else:
    print("Old block not found!")
