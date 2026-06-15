use gpui::{
    div, prelude::*, px, rgb, size, App, Bounds, FontWeight, Render, TitlebarOptions,
    Window, WindowBounds, WindowOptions,
};

pub struct DefaultBrowserPrompt;

impl DefaultBrowserPrompt {
    pub fn show(cx: &mut App) {
        let bounds = Bounds::centered(None, size(px(480.0), px(280.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Set Default Browser".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                focus: true,
                show: true,
                ..Default::default()
            },
            |_window, cx| cx.new(|_| DefaultBrowserPrompt),
        )
        .ok();
    }
}

impl Render for DefaultBrowserPrompt {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bg = rgb(0x2a_2a_2e);
        let surface = rgb(0x3a_3a_3e);
        let text_primary = rgb(0xff_ff_ff);
        let text_secondary = rgb(0xaa_aa_aa);
        let accent = rgb(0x4a_90_d9);

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(bg)
            .p(px(32.0))
            .gap_4()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(text_primary)
                    .child("Set Browseraptor as Default"),
            )
            .child(
                div()
                    .text_base()
                    .text_color(text_secondary)
                    .child("Browseraptor works best when set as your default browser."),
            )
            .child(div().flex_grow())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_3()
                    .justify_end()
                    .child(
                        div()
                            .id("btn-skip")
                            .px(px(16.0))
                            .py(px(8.0))
                            .bg(surface)
                            .rounded(px(6.0))
                            .cursor_pointer()
                            .text_color(text_primary)
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .child("Skip")
                            .on_click(cx.listener(|_this, _event, window, _cx| {
                                window.remove_window();
                            })),
                    )
                    .child(
                        div()
                            .id("btn-auto-set")
                            .px(px(16.0))
                            .py(px(8.0))
                            .bg(accent)
                            .rounded(px(6.0))
                            .cursor_pointer()
                            .text_color(rgb(0xff_ff_ff))
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .child("Set as Default")
                            .on_click(cx.listener(|_this, _event, window, _cx| {
                                match crate::default_browser::set_as_default_browser() {
                                    Ok(_) => {
                                        tracing::info!("Set default browser automatically");
                                        window.remove_window();
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to auto-set default browser: {}", e);
                                        crate::default_browser::open_default_browser_settings();
                                        window.remove_window();
                                    }
                                }
                            })),
                    ),
            )
    }
}
