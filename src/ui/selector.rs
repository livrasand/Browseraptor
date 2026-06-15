use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};

use gpui::{
    div, prelude::*, px, rgb, size, svg, App, Application, Bounds, FontWeight, Render,
    SharedString, TitlebarOptions, Window, WindowBounds, WindowOptions,
};

use crate::browser::Browser;
use crate::config::Config;
use crate::AppAssets;

/// Cache of rendered browser app icons (keyed by .app path) to avoid expensive
/// ObjC calls and PNG decoding on every render.
static ICON_CACHE: OnceLock<Mutex<HashMap<String, Arc<gpui::RenderImage>>>> = OnceLock::new();

#[derive(Debug, Clone)]
pub enum SelectorResult {
    Selected { browser: Browser, remember: bool },
    Cancelled,
}

struct BrowserItem {
    name: SharedString,
    browser: Browser,
}

struct Selector {
    url: SharedString,
    domain: SharedString,
    items: Vec<BrowserItem>,
    selected_idx: usize,
    remember: bool,
    outcome: Rc<RefCell<Option<SelectorResult>>>,
}

impl Selector {
    fn new(
        url: String,
        domain: String,
        browsers: Vec<Browser>,
        outcome: Rc<RefCell<Option<SelectorResult>>>,
    ) -> Self {
        let items = browsers
            .into_iter()
            .map(|b| {
                let name: SharedString = b.name().to_owned().into();
                BrowserItem { name, browser: b }
            })
            .collect();

        Self {
            url: url.into(),
            domain: domain.into(),
            items,
            selected_idx: 0,
            remember: false,
            outcome,
        }
    }
}

impl Render for Selector {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let accent = rgb(0x4a_90_d9);
        let bg = rgb(0x2a_2a_2e);
        let surface = rgb(0x3a_3a_3e);
        let text_primary = rgb(0xff_ff_ff);
        let text_secondary = rgb(0xaa_aa_aa);

        let domain = self.domain.clone();
        let remember = self.remember;
        let selected_idx = self.selected_idx;

        let list_elements: Vec<_> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = selected_idx == i;
                let item_bg = if is_selected { accent } else { surface };
                let name = item.name.clone();
                let browser = item.browser.clone();
                let idx = i;

                div()
                    .id(("browser", i))
                    .flex()
                    .items_center()
                    .gap_3()
                    .px(px(16.0))
                    .py(px(12.0))
                    .bg(item_bg)
                    .rounded(px(8.0))
                    .cursor_pointer()
                    .text_color(text_primary)
                    .text_base()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(browser_icon(&browser))
                    .child(name)
                    .child(div().flex_grow())
                    .child(
                        div()
                            .size(px(18.0))
                            .rounded(px(9.0))
                            .border_2()
                            .border_color(if is_selected {
                                rgb(0xff_ff_ff)
                            } else {
                                text_secondary
                            })
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(if is_selected {
                                div().size(px(8.0)).rounded(px(4.0)).bg(rgb(0xff_ff_ff))
                            } else {
                                div()
                            }),
                    )
                    .on_click(cx.listener(move |this, _event, _window, _cx| {
                        this.selected_idx = idx;
                    }))
            })
            .collect();

        div()
            .flex()
            .flex_col()
            .w(px(420.0))
            .bg(bg)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .px(px(24.0))
                    .pt(px(24.0))
                    .pb(px(12.0))
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(text_primary)
                            .child("Open with..."),
                    )
                    .child(
                        div()
                            .text_sm()
                            .mt(px(4.0))
                            .text_color(text_secondary)
                            .child(domain),
                    )
                    .child(
                        div()
                            .text_xs()
                            .mt(px(2.0))
                            .text_color(gpui::rgb(0x66_66_66))
                            .child(self.url.clone()),
                    ),
            )
            .child(
                div()
                    .id("browser-list")
                    .flex()
                    .flex_col()
                    .px(px(16.0))
                    .gap_1()
                    .overflow_y_scroll()
                    .children(list_elements),
            )
            .child(
                div()
                    .id("remember-check")
                    .flex()
                    .items_center()
                    .gap_2()
                    .px(px(24.0))
                    .py(px(12.0))
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _event, _window, _cx| {
                        this.remember = !this.remember;
                    }))
                    .child(
                        div()
                            .size(px(18.0))
                            .rounded(px(4.0))
                            .border_2()
                            .border_color(text_secondary)
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(if remember {
                                div()
                                    .size(px(18.0))
                                    .bg(accent)
                                    .rounded(px(4.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_color(rgb(0xff_ff_ff))
                                    .text_xs()
                                    .child("✓")
                            } else {
                                div()
                            }),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(text_secondary)
                            .child(format!("Always use this browser for {}", self.domain)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_3()
                    .justify_end()
                    .px(px(24.0))
                    .py(px(16.0))
                    .child(
                        div()
                            .id("cancel-btn")
                            .px(px(20.0))
                            .py(px(10.0))
                            .bg(surface)
                            .rounded(px(8.0))
                            .cursor_pointer()
                            .text_color(text_secondary)
                            .text_sm()
                            .child("Cancel")
                            .on_click(cx.listener(move |this, _event, _window, cx| {
                                *this.outcome.borrow_mut() = Some(SelectorResult::Cancelled);
                                cx.spawn(
                                    |_weak: gpui::WeakEntity<Selector>,
                                     app: &mut gpui::AsyncApp| {
                                        let _ = app.update(|app| app.quit());
                                        async {}
                                    },
                                )
                                .detach();
                            })),
                    )
                    .child(
                        div()
                            .id("open-btn")
                            .px(px(20.0))
                            .py(px(10.0))
                            .bg(accent)
                            .rounded(px(8.0))
                            .cursor_pointer()
                            .text_color(rgb(0xff_ff_ff))
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .child("Open")
                            .on_click(cx.listener(move |this, _event, _window, cx| {
                                let browser = this.items[this.selected_idx].browser.clone();
                                *this.outcome.borrow_mut() = Some(SelectorResult::Selected {
                                    browser,
                                    remember: this.remember,
                                });
                                cx.spawn(
                                    |_weak: gpui::WeakEntity<Selector>,
                                     app: &mut gpui::AsyncApp| {
                                        let _ = app.update(|app| app.quit());
                                        async {}
                                    },
                                )
                                .detach();
                            })),
                    ),
            )
    }
}

/// Selector used inside the daemon (no standalone Application::new())
#[allow(dead_code)]
pub struct DaemonSelector {
    url: gpui::SharedString,
    domain: gpui::SharedString,
    items: Vec<(gpui::SharedString, Browser)>,
    selected_idx: usize,
    editing_hotkey: Option<usize>,
    hovered_key: Option<usize>,
    hotkeys: std::collections::HashMap<String, String>,
    focus_handle: gpui::FocusHandle,
    command_tx: std::sync::mpsc::Sender<crate::app::AppCommand>,
    show_settings: bool,
    show_plugin_search: bool,
    plugin_query: String,
    channel_plugins: Option<Vec<(String, crate::plugin::ChannelPluginEntry)>>,
    installed_plugins: Vec<crate::plugin::InstalledPlugin>,
    repositories: Vec<crate::config::Repository>,
    show_repo_info: bool,
    show_add_repo: bool,
    add_repo_name: String,
    add_repo_url: String,
    repo_input_focus: u8,
}

impl DaemonSelector {
    pub fn new(
        url: String,
        domain: String,
        browsers: Vec<Browser>,
        command_tx: std::sync::mpsc::Sender<crate::app::AppCommand>,
        cx: &mut gpui::Context<Self>,
    ) -> Self {
        let items = browsers
            .into_iter()
            .map(|b| (gpui::SharedString::from(b.name().to_owned()), b))
            .collect();
        let config = crate::config::Config::load().unwrap_or_default();
        Self {
            url: url.into(),
            domain: domain.into(),
            items,
            selected_idx: 0,
            editing_hotkey: None,
            hovered_key: None,
            hotkeys: config.hotkeys,
            focus_handle: cx.focus_handle(),
            command_tx,
            show_settings: false,
            show_plugin_search: false,
            plugin_query: String::new(),
            channel_plugins: None,
            installed_plugins: config.installed_plugins,
            repositories: config.repositories,
            show_repo_info: false,
            show_add_repo: false,
            add_repo_name: String::new(),
            add_repo_url: String::new(),
            repo_input_focus: 0,
        }
    }

    fn shortcut_char(browser: &Browser) -> &'static str {
        match browser {
            Browser::Chrome => "C",
            Browser::Firefox => "F",
            Browser::Brave => "B",
            Browser::Edge => "E",
            Browser::Safari => "S",
            Browser::Arc => "A",
            Browser::Orion => "O",
            Browser::Other { .. } => "?",
        }
    }

    fn render_settings_panel(
        &self,
        text: gpui::Rgba,
        text_dim: gpui::Rgba,
        _bg: gpui::Rgba,
        cx: &mut gpui::Context<Self>,
    ) -> gpui::AnyElement {
        use gpui::IntoElement;

        let accent = rgb(0x4a_90_d9);
        let row_bg = rgb(0x28_28_28);
        let badge_bg = rgb(0x3a_3a_3a);

        let browser_rows: Vec<_> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, (name, browser))| {
                let custom_key = self
                    .hotkeys
                    .get(browser.name())
                    .cloned()
                    .unwrap_or_else(|| Self::shortcut_char(browser).to_owned());
                let is_editing = self.editing_hotkey == Some(i);

                let key_label: gpui::AnyElement = if is_editing {
                    div()
                        .px(px(6.0))
                        .py(px(2.0))
                        .bg(accent)
                        .rounded(px(4.0))
                        .text_color(text)
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .child("...")
                        .into_any_element()
                } else {
                    div()
                        .px(px(6.0))
                        .py(px(2.0))
                        .bg(badge_bg)
                        .rounded(px(4.0))
                        .text_color(accent)
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .child(custom_key)
                        .into_any_element()
                };

                div()
                    .id(("settings-row", i))
                    .flex()
                    .items_center()
                    .gap_3()
                    .px(px(12.0))
                    .py(px(8.0))
                    .bg(row_bg)
                    .rounded(px(6.0))
                    .child(daemon_browser_icon(browser))
                    .child(
                        div()
                            .flex_grow()
                            .text_color(text)
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(name.clone()),
                    )
                    .child(key_label)
                    .on_click(cx.listener(move |this, _ev, _window, cx| {
                        if this.editing_hotkey == Some(i) {
                            this.editing_hotkey = None;
                        } else {
                            this.editing_hotkey = Some(i);
                        }
                        cx.notify();
                    }))
            })
            .collect();

        div()
            .id("settings-scroll")
            .flex()
            .flex_col()
            .flex_grow()
            .overflow_y_scroll()
            .px(px(14.0))
            .pt(px(12.0))
            .pb(px(8.0))
            .gap_3()
            // Section: Browsers
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(text_dim)
                    .child("BROWSERS"),
            )
            .child(div().flex().flex_col().gap_1().children(browser_rows))
            // Section: Hotkeys hint
            .child(div().h(px(1.0)).bg(rgb(0x3a_3a_3a)).mt(px(4.0)))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(text_dim)
                    .child("HOTKEYS"),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(text_dim)
                    .child("Click a browser row above to assign a custom key."),
            )
            .into_any_element()
    }

    fn render_plugin_panel(
        &mut self,
        text: gpui::Rgba,
        text_dim: gpui::Rgba,
        cx: &mut gpui::Context<Self>,
    ) -> gpui::AnyElement {
        use gpui::IntoElement;

        let accent = rgb(0x4a_90_d9);
        let row_bg = rgb(0x28_28_28);
        let badge_bg = rgb(0x3a_3a_3a);
        let green = rgb(0x34_c7_59);
        let red = rgb(0xff_45_45);

        // If a background fetch populated the channel cache, pick it up now so
        // the UI can render results without blocking.
        if self.channel_plugins.is_none() {
            if let Some(cached) = crate::plugin::get_channel_cache() {
                self.channel_plugins = Some(cached);
            }
        }

        // ── Installed plugins ──────────────────────────────────────────
        let installed_rows: Vec<_> = self
            .installed_plugins
            .iter()
            .map(|p| {
                let name = p.name.clone();
                let ver = p.version.clone();
                let desc = p.description.clone().unwrap_or_default();
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .px(px(12.0))
                    .py(px(8.0))
                    .bg(row_bg)
                    .rounded(px(6.0))
                    .child(
                        div()
                            .size(px(28.0))
                            .rounded(px(6.0))
                            .bg(badge_bg)
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(accent)
                            .text_xs()
                            .child("⬡"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_grow()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(text)
                                    .child(name),
                            )
                            .child(div().text_xs().text_color(text_dim).child(desc)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(text_dim)
                            .child(format!("v{}", ver)),
                    )
            })
            .collect();

        // ── Channel plugins (search results) ───────────────────────────
        let query = self.plugin_query.to_lowercase();
        let channel_rows: Vec<gpui::AnyElement> = match &self.channel_plugins {
            None => vec![div()
                .text_xs()
                .text_color(text_dim)
                .child("Presiona ↵ para buscar en el canal…")
                .into_any_element()],
            Some(entries) => {
                let filtered: Vec<_> = entries
                    .iter()
                    .filter(|(id, e)| {
                        query.is_empty()
                            || id.to_lowercase().contains(&query)
                            || e.name.to_lowercase().contains(&query)
                            || e.description
                                .as_deref()
                                .unwrap_or("")
                                .to_lowercase()
                                .contains(&query)
                    })
                    .collect();

                if filtered.is_empty() {
                    vec![div()
                        .text_xs()
                        .text_color(text_dim)
                        .child("No se encontraron plugins.")
                        .into_any_element()]
                } else {
                    filtered
                        .iter()
                        .enumerate()
                        .map(|(row_idx, (id, entry))| {
                            let id = id.to_string();
                            let name = entry.name.clone();
                            let desc = entry.description.clone().unwrap_or_default();
                            let ver = entry.version.clone();
                            let already = self.installed_plugins.iter().any(|p| p.id == id);

                            div()
                                .id(("plugin-row", row_idx))
                                .flex()
                                .items_center()
                                .gap_3()
                                .px(px(12.0))
                                .py(px(8.0))
                                .bg(row_bg)
                                .rounded(px(6.0))
                                .child(
                                    div()
                                        .size(px(28.0))
                                        .rounded(px(6.0))
                                        .bg(badge_bg)
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_color(accent)
                                        .text_xs()
                                        .child("⬡"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .flex_grow()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(text)
                                                .child(name),
                                        )
                                        .child(div().text_xs().text_color(text_dim).child(desc)),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(text_dim)
                                        .child(format!("v{}", ver)),
                                )
                                .child(if already {
                                    div()
                                        .px(px(6.0))
                                        .py(px(2.0))
                                        .bg(badge_bg)
                                        .rounded(px(4.0))
                                        .text_color(green)
                                        .text_xs()
                                        .child("✓")
                                        .into_any_element()
                                } else {
                                    div()
                                        .id(("install-btn", row_idx))
                                        .px(px(6.0))
                                        .py(px(2.0))
                                        .bg(accent)
                                        .rounded(px(4.0))
                                        .text_color(rgb(0xff_ff_ff))
                                        .text_xs()
                                        .cursor_pointer()
                                        .child("Instalar")
                                        .on_click(cx.listener(move |this, _ev, _window, cx| {
                                            let plugin = crate::plugin::InstalledPlugin {
                                                id: id.clone(),
                                                name: this
                                                    .channel_plugins
                                                    .as_ref()
                                                    .and_then(|v| v.iter().find(|(i, _)| i == &id))
                                                    .map(|(_, e)| e.name.clone())
                                                    .unwrap_or_else(|| id.clone()),
                                                version: this
                                                    .channel_plugins
                                                    .as_ref()
                                                    .and_then(|v| v.iter().find(|(i, _)| i == &id))
                                                    .map(|(_, e)| e.version.clone())
                                                    .unwrap_or_default(),
                                                description: this
                                                    .channel_plugins
                                                    .as_ref()
                                                    .and_then(|v| v.iter().find(|(i, _)| i == &id))
                                                    .and_then(|(_, e)| e.description.clone()),
                                            };
                                            this.installed_plugins.push(plugin.clone());
                                            if let Ok(mut config) = crate::config::Config::load() {
                                                config.installed_plugins.push(plugin);
                                                let _ = config.save();
                                            }
                                            cx.notify();
                                        }))
                                        .into_any_element()
                                })
                                .into_any_element()
                        })
                        .collect()
                }
            }
        };

        // ── Repository rows ────────────────────────────────────────────
        let repo_rows: Vec<gpui::AnyElement> = self
            .repositories
            .iter()
            .enumerate()
            .map(|(ri, repo)| {
                let name = repo.name.clone();
                let url = repo.url.clone();
                let url_display = url.replace("/", "/\u{200B}").replace(".", ".\u{200B}");
                let enabled = repo.enabled;
                div()
                    .id(("repo-row", ri))
                    .flex()
                    .items_center()
                    .gap_2()
                    .px(px(10.0))
                    .py(px(6.0))
                    .bg(row_bg)
                    .rounded(px(6.0))
                    // toggle enabled dot
                    .child(
                        div()
                            .id(("repo-toggle", ri))
                            .size(px(10.0))
                            .rounded(px(5.0))
                            .bg(if enabled { green } else { rgb(0x55_55_55) })
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, _ev, _window, cx| {
                                if let Some(r) = this.repositories.get_mut(ri) {
                                    r.enabled = !r.enabled;
                                }
                                if let Ok(mut config) = crate::config::Config::load() {
                                    config.repositories = this.repositories.clone();
                                    let _ = config.save();
                                }
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_grow()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(if enabled { text } else { text_dim })
                                    .child(name),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(text_dim)
                                    .flex_grow()
                                    .min_w(px(0.0))
                                    .child(url_display),
                            ),
                    )
                    // remove button (only for non-official repos)
                    .child(if ri > 0 {
                        div()
                            .id(("repo-remove", ri))
                            .size(px(18.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_xs()
                            .text_color(red)
                            .cursor_pointer()
                            .child("✕")
                            .on_click(cx.listener(move |this, _ev, _window, cx| {
                                if ri < this.repositories.len() {
                                    this.repositories.remove(ri);
                                    if let Ok(mut config) = crate::config::Config::load() {
                                        config.repositories = this.repositories.clone();
                                        let _ = config.save();
                                    }
                                    cx.notify();
                                }
                            }))
                            .into_any_element()
                    } else {
                        div().into_any_element()
                    })
                    .into_any_element()
            })
            .collect();

        // ── Add repo form ──────────────────────────────────────────────
        let add_repo_form: gpui::AnyElement = if self.show_add_repo {
            let name_val = if self.add_repo_name.is_empty() {
                if self.repo_input_focus == 0 {
                    "Nombre del repositorio|".to_string()
                } else {
                    "Nombre del repositorio…".to_string()
                }
            } else if self.repo_input_focus == 0 {
                format!("{}|", self.add_repo_name)
            } else {
                self.add_repo_name.clone()
            };
            let url_val = if self.add_repo_url.is_empty() {
                if self.repo_input_focus == 1 {
                    "https://…|".to_string()
                } else {
                    "https://…".to_string()
                }
            } else if self.repo_input_focus == 1 {
                format!("{}|", self.add_repo_url)
            } else {
                self.add_repo_url.clone()
            };

            div()
                .flex()
                .flex_col()
                .gap_1()
                .p(px(8.0))
                .bg(rgb(0x22_22_22))
                .rounded(px(6.0))
                .child(
                    div()
                        .id("repo-name-input")
                        .px(px(8.0))
                        .py(px(4.0))
                        .bg(if self.repo_input_focus == 0 {
                            rgb(0x38_38_38)
                        } else {
                            row_bg
                        })
                        .rounded(px(4.0))
                        .text_xs()
                        .text_color(if self.add_repo_name.is_empty() {
                            text_dim
                        } else {
                            text
                        })
                        .cursor_pointer()
                        .child(name_val)
                        .on_click(cx.listener(|this, _ev, _window, cx| {
                            this.repo_input_focus = 0;
                            cx.notify();
                        })),
                )
                .child(
                    div()
                        .id("repo-url-input")
                        .px(px(8.0))
                        .py(px(4.0))
                        .bg(if self.repo_input_focus == 1 {
                            rgb(0x38_38_38)
                        } else {
                            row_bg
                        })
                        .rounded(px(4.0))
                        .text_xs()
                        .text_color(if self.add_repo_url.is_empty() {
                            text_dim
                        } else {
                            text
                        })
                        .cursor_pointer()
                        .child(url_val)
                        .on_click(cx.listener(|this, _ev, _window, cx| {
                            this.repo_input_focus = 1;
                            cx.notify();
                        })),
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            div()
                                .id("repo-add-confirm")
                                .flex_grow()
                                .px(px(8.0))
                                .py(px(4.0))
                                .bg(accent)
                                .rounded(px(4.0))
                                .text_xs()
                                .text_color(rgb(0xff_ff_ff))
                                .cursor_pointer()
                                .child("Agregar")
                                .on_click(cx.listener(|this, _ev, _window, cx| {
                                    let name = this.add_repo_name.trim().to_string();
                                    let url = this.add_repo_url.trim().to_string();
                                    if !name.is_empty() && !url.is_empty() {
                                        this.repositories.push(crate::config::Repository {
                                            name,
                                            url,
                                            enabled: true,
                                        });
                                        if let Ok(mut config) = crate::config::Config::load() {
                                            config.repositories = this.repositories.clone();
                                            let _ = config.save();
                                        }
                                        this.add_repo_name.clear();
                                        this.add_repo_url.clear();
                                        this.show_add_repo = false;
                                    }
                                    cx.notify();
                                })),
                        )
                        .child(
                            div()
                                .id("repo-add-cancel")
                                .px(px(8.0))
                                .py(px(4.0))
                                .bg(badge_bg)
                                .rounded(px(4.0))
                                .text_xs()
                                .text_color(text_dim)
                                .cursor_pointer()
                                .child("Cancelar")
                                .on_click(cx.listener(|this, _ev, _window, cx| {
                                    this.show_add_repo = false;
                                    this.add_repo_name.clear();
                                    this.add_repo_url.clear();
                                    cx.notify();
                                })),
                        ),
                )
                .into_any_element()
        } else {
            div().into_any_element()
        };

        // ── Search bar display text ────────────────────────────────────
        let search_display = if self.plugin_query.is_empty() {
            "Buscar plugins…".to_string()
        } else {
            format!("{}|", self.plugin_query)
        };

        // ── Main scrollable panel ──────────────────────────────────────
        let mut panel = div()
            .id("plugin-scroll")
            .flex()
            .flex_col()
            .flex_grow()
            .overflow_y_scroll()
            .px(px(14.0))
            .pt(px(12.0))
            .pb(px(8.0))
            .gap_3()
            // Search bar
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px(px(10.0))
                    .py(px(7.0))
                    .bg(rgb(0x28_28_28))
                    .rounded(px(8.0))
                    .child(div().text_color(rgb(0x88_88_88)).text_xs().child("⌕"))
                    .child(
                        div()
                            .flex_grow()
                            .text_xs()
                            .text_color(if self.plugin_query.is_empty() {
                                rgb(0x55_55_55)
                            } else {
                                rgb(0xff_ff_ff)
                            })
                            .child(search_display),
                    ),
            )
            // INSTALLED section
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(text_dim)
                    .child("INSTALADOS"),
            )
            .child(if self.installed_plugins.is_empty() {
                div()
                    .text_xs()
                    .text_color(text_dim)
                    .child("Aún no hay plugins instalados.")
                    .into_any_element()
            } else {
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .children(installed_rows)
                    .into_any_element()
            })
            // CHANNEL section
            .child(div().h(px(1.0)).bg(rgb(0x3a_3a_3a)).mt(px(4.0)))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(text_dim)
                    .child("CANAL"),
            )
            .child(div().flex().flex_col().gap_1().children(channel_rows))
            // REPOSITORIES section
            .child(div().h(px(1.0)).bg(rgb(0x3a_3a_3a)).mt(px(4.0)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    // Section title
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(text_dim)
                            .flex_grow()
                            .child("REPOSITORIOS"),
                    )
                    // Info button
                    .child(
                        div()
                            .id("repo-info-btn")
                            .size(px(16.0))
                            .rounded(px(8.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(badge_bg)
                            .text_xs()
                            .text_color(text_dim)
                            .cursor_pointer()
                            .child("i")
                            .on_click(cx.listener(|this, _ev, _window, cx| {
                                this.show_repo_info = !this.show_repo_info;
                                cx.notify();
                            })),
                    )
                    // Add button
                    .child(
                        div()
                            .id("repo-add-btn")
                            .px(px(6.0))
                            .py(px(2.0))
                            .rounded(px(4.0))
                            .bg(badge_bg)
                            .text_xs()
                            .text_color(text_dim)
                            .cursor_pointer()
                            .child("+ Añadir")
                            .on_click(cx.listener(|this, _ev, _window, cx| {
                                this.show_add_repo = !this.show_add_repo;
                                this.repo_input_focus = 0;
                                cx.notify();
                            })),
                    ),
            )
            .child(add_repo_form)
            .child(div().flex().flex_col().gap_1().children(repo_rows));

        // ── Info modal overlay ─────────────────────────────────────────
        if self.show_repo_info {
            panel = panel.child(
                div()
                    .id("repo-info-modal")
                    .absolute()
                    .inset(px(16.0))
                    .bg(rgb(0x2a_2a_2a))
                    .rounded(px(10.0))
                    .p(px(16.0))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(text)
                                    .flex_grow()
                                    .child("Repositorios"),
                            )
                            .child(
                                div()
                                    .id("repo-info-close")
                                    .size(px(20.0))
                                    .rounded(px(10.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .bg(badge_bg)
                                    .text_xs()
                                    .text_color(text_dim)
                                    .cursor_pointer()
                                    .child("✕")
                                    .on_click(cx.listener(|this, _ev, _window, cx| {
                                        this.show_repo_info = false;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(text_dim)
                            .child("Un repositorio es una fuente de plugins. Esta lista muestra todos los repositorios añadidos actualmente. Los repositorios desactivados no se utilizan. Si un plugin está en más de un repositorio, se preferirá automáticamente el repositorio que esté más arriba de la lista. Puede reordenar los repositorios pulsando prolongadamente y arrastrándolos."),
                    ),
            );
        }

        panel.into_any_element()
    }
}

impl gpui::Render for DaemonSelector {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        window.focus(&self.focus_handle);
        let bg = rgb(0x1e_1e_1e);
        let row_hover = rgb(0x3a_78_d4);
        let row_normal = rgb(0x28_28_28);
        let key_bg = rgb(0x3a_3a_3a);
        let key_edit_bg = rgb(0x4a_90_d9);
        let text = rgb(0xff_ff_ff);
        let text_dim = rgb(0x88_88_88);
        let bar_bg = rgb(0x28_28_28);
        let selected = self.selected_idx;
        let editing = self.editing_hotkey;
        let hovered = self.hovered_key;

        let url_display = self.url.clone();

        let rows: Vec<_> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, (name, browser))| {
                let is_sel = i == selected;
                let is_editing = editing == Some(i);
                let is_hovered = hovered == Some(i);
                let row_bg = if is_sel { row_hover } else { row_normal };
                let name = name.clone();
                let browser = browser.clone();
                let browser_name = browser.name().to_owned();
                let default_key = Self::shortcut_char(&browser);
                let custom_key = self
                    .hotkeys
                    .get(&browser_name)
                    .cloned()
                    .unwrap_or_else(|| default_key.to_owned());

                let badge_bg = if is_editing {
                    key_edit_bg
                } else if is_hovered {
                    rgb(0x55_55_55)
                } else {
                    key_bg
                };

                let badge_content: gpui::AnyElement = if is_editing {
                    div()
                        .text_color(text)
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .child("...")
                        .into_any_element()
                } else if is_hovered {
                    div()
                        .text_color(text)
                        .text_xs()
                        .child("✎")
                        .into_any_element()
                } else {
                    div()
                        .text_color(text_dim)
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .child(custom_key.clone())
                        .into_any_element()
                };

                div()
                    .id(("row", i))
                    .flex()
                    .items_center()
                    .gap_3()
                    .px(px(16.0))
                    .py(px(13.0))
                    .mx(px(8.0))
                    .rounded(px(10.0))
                    .bg(row_bg)
                    .cursor_pointer()
                    .child(daemon_browser_icon(&browser))
                    .child(
                        div()
                            .flex_grow()
                            .text_color(text)
                            .text_base()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(name),
                    )
                    .child(
                        div()
                            .id(("key-badge", i))
                            .size(px(28.0))
                            .rounded(px(6.0))
                            .bg(badge_bg)
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .on_mouse_move(cx.listener(move |this, _ev, _window, cx| {
                                if this.hovered_key != Some(i) {
                                    this.hovered_key = Some(i);
                                    cx.notify();
                                }
                                cx.stop_propagation();
                            }))
                            .on_mouse_up(
                                gpui::MouseButton::Left,
                                cx.listener(move |this, _ev, _window, cx| {
                                    if this.editing_hotkey == Some(i) {
                                        this.editing_hotkey = None;
                                    } else {
                                        this.editing_hotkey = Some(i);
                                    }
                                    cx.notify();
                                    cx.stop_propagation();
                                }),
                            )
                            .child(badge_content),
                    )
                    .on_mouse_up(
                        gpui::MouseButton::Left,
                        cx.listener(move |this, _ev, window, _cx| {
                            if this.editing_hotkey.is_some() {
                                return;
                            }
                            this.selected_idx = i;
                            let url = this.url.to_string();
                            let _ = crate::browser::launcher::launch(&browser, &url);
                            window.remove_window();
                        }),
                    )
            })
            .collect();

        div()
            .id("selector-root")
            .flex()
            .flex_col()
            .w(px(420.0))
            .h(px(460.0))
            .bg(bg)
            .rounded(px(14.0))
            .overflow_hidden()
            .track_focus(&self.focus_handle)
            .on_mouse_move(cx.listener(|this, _ev, _window, cx| {
                if this.hovered_key.is_some() {
                    this.hovered_key = None;
                    cx.notify();
                }
            }))
            .on_key_down(cx.listener(|this, ev: &gpui::KeyDownEvent, window, cx| {
                let key = ev.keystroke.key.to_lowercase();

                if let Some(edit_idx) = this.editing_hotkey {
                    if key == "escape" {
                        this.editing_hotkey = None;
                        cx.notify();
                        return;
                    }
                    // Build display label: include modifiers + key
                    let mut label = String::new();
                    if ev.keystroke.modifiers.platform {
                        label.push_str("⌘");
                    }
                    if ev.keystroke.modifiers.control {
                        label.push_str("⌃");
                    }
                    if ev.keystroke.modifiers.alt {
                        label.push_str("⌥");
                    }
                    if ev.keystroke.modifiers.shift {
                        label.push_str("⇧");
                    }
                    label.push_str(&ev.keystroke.key.to_uppercase());

                    let browser_name = this.items[edit_idx].1.name().to_owned();
                    this.hotkeys.insert(browser_name.clone(), label.clone());
                    this.editing_hotkey = None;
                    cx.notify();

                    // Persist to config
                    if let Ok(mut config) = crate::config::Config::load() {
                        config.hotkeys.insert(browser_name, label);
                        let _ = config.save();
                    }
                    return;
                }

                // Plugin search mode
                if this.show_plugin_search {
                    // If add-repo form is open, route keys to the focused field
                    if this.show_add_repo {
                        match key.as_str() {
                            "escape" => {
                                this.show_add_repo = false;
                                this.add_repo_name.clear();
                                this.add_repo_url.clear();
                                cx.notify();
                            }
                            "tab" => {
                                this.repo_input_focus =
                                    if this.repo_input_focus == 0 { 1 } else { 0 };
                                cx.notify();
                            }
                            "backspace" => {
                                if this.repo_input_focus == 0 {
                                    this.add_repo_name.pop();
                                } else {
                                    this.add_repo_url.pop();
                                }
                                cx.notify();
                            }
                            "return" | "enter" => {
                                if this.repo_input_focus == 0 && !this.add_repo_name.is_empty() {
                                    this.repo_input_focus = 1;
                                } else {
                                    let name = this.add_repo_name.trim().to_string();
                                    let url = this.add_repo_url.trim().to_string();
                                    if !name.is_empty() && !url.is_empty() {
                                        this.repositories.push(crate::config::Repository {
                                            name,
                                            url,
                                            enabled: true,
                                        });
                                        if let Ok(mut config) = crate::config::Config::load() {
                                            config.repositories = this.repositories.clone();
                                            let _ = config.save();
                                        }
                                        this.add_repo_name.clear();
                                        this.add_repo_url.clear();
                                        this.show_add_repo = false;
                                    }
                                }
                                cx.notify();
                            }
                            k if !ev.keystroke.modifiers.platform
                                && !ev.keystroke.modifiers.control =>
                            {
                                // Reconstruct the actual character: use key directly (it's the char)
                                let ch = if ev.keystroke.modifiers.shift {
                                    k.to_uppercase()
                                } else {
                                    k.to_string()
                                };
                                if this.repo_input_focus == 0 {
                                    this.add_repo_name.push_str(&ch);
                                } else {
                                    this.add_repo_url.push_str(&ch);
                                }
                                cx.notify();
                            }
                            _ => {}
                        }
                        return;
                    }

                    match key.as_str() {
                        "escape" => {
                            this.show_plugin_search = false;
                            this.plugin_query.clear();
                            cx.notify();
                        }
                        "backspace" => {
                            this.plugin_query.pop();
                            cx.notify();
                        }
                        "return" | "enter" => {
                            // Fetch channel asynchronously to avoid blocking the UI thread.
                            // The background worker will populate plugin::get_channel_cache()
                            // and send AppCommand::ChannelFetched when done — the main loop
                            // will call cx.notify(), and the component will pick up the
                            // cached value on the next render.
                            let _ = crate::plugin::fetch_channel_async(this.command_tx.clone());
                            this.channel_plugins = None;
                            cx.notify();
                        }
                        k if k.len() == 1
                            && !ev.keystroke.modifiers.platform
                            && !ev.keystroke.modifiers.control =>
                        {
                            let ch = if ev.keystroke.modifiers.shift {
                                k.to_uppercase()
                            } else {
                                k.to_string()
                            };
                            this.plugin_query.push_str(&ch);
                            cx.notify();
                        }
                        _ => {
                            tracing::debug!(
                                "Plugin search unhandled key: key={:?}, len={}",
                                key,
                                key.chars().count()
                            );
                        }
                    }
                    return;
                }

                // Normal mode: launch by hotkey
                let hit = this.items.iter().position(|(_, b)| {
                    let stored = this
                        .hotkeys
                        .get(b.name())
                        .cloned()
                        .unwrap_or_else(|| Self::shortcut_char(b).to_owned());
                    stored.to_lowercase() == key
                });
                if let Some(idx) = hit {
                    let url = this.url.to_string();
                    let browser = this.items[idx].1.clone();
                    let _ = crate::browser::launcher::launch(&browser, &url);
                    window.remove_window();
                } else if key == "escape" {
                    window.remove_window();
                }
            }))
            // main content: browser list, settings or plugin search
            .child(if self.show_plugin_search {
                self.render_plugin_panel(text, text_dim, cx)
            } else if self.show_settings {
                self.render_settings_panel(text, text_dim, bg, cx)
            } else {
                div()
                    .flex()
                    .flex_col()
                    .flex_grow()
                    .gap_1()
                    .pt(px(10.0))
                    .pb(px(6.0))
                    .children(rows)
                    .into_any_element()
            })
            // divider
            .child(div().h(px(1.0)).mx(px(8.0)).bg(rgb(0x3a_3a_3a)))
            // bottom bar: URL + settings toggle + cancel
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px(px(14.0))
                    .py(px(10.0))
                    .bg(bar_bg)
                    // plugin search icon
                    .child(
                        div()
                            .id("plugin-btn")
                            .size(px(22.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .text_color(if self.show_plugin_search {
                                rgb(0x4a_90_d9)
                            } else {
                                text_dim
                            })
                            .text_base()
                            .child(
                                svg()
                                    .path("assets/icons/lucide--blocks.svg")
                                    .size(px(16.0))
                                    .text_color(if self.show_plugin_search {
                                        rgb(0x4a_90_d9)
                                    } else {
                                        text_dim
                                    }),
                            )
                            .on_click(cx.listener(|this, _ev, _window, cx| {
                                this.show_plugin_search = !this.show_plugin_search;
                                if this.show_plugin_search {
                                    this.show_settings = false;
                                    this.plugin_query.clear();
                                    this.channel_plugins = None;
                                    // Fetch channel on open asynchronously (non-blocking).
                                    let _ =
                                        crate::plugin::fetch_channel_async(this.command_tx.clone());
                                }
                                cx.notify();
                            })),
                    )
                    // settings gear icon
                    .child(
                        div()
                            .id("settings-btn")
                            .size(px(22.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .text_color(if self.show_settings {
                                rgb(0x4a_90_d9)
                            } else {
                                text_dim
                            })
                            .text_base()
                            .child(
                                svg()
                                    .path("assets/icons/lucide--cog.svg")
                                    .size(px(16.0))
                                    .text_color(if self.show_settings {
                                        rgb(0x4a_90_d9)
                                    } else {
                                        text_dim
                                    }),
                            )
                            .on_click(cx.listener(|this, _ev, _window, cx| {
                                this.show_settings = !this.show_settings;
                                if this.show_settings {
                                    this.show_plugin_search = false;
                                }
                                this.editing_hotkey = None;
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .flex_grow()
                            .text_color(text_dim)
                            .text_sm()
                            .child(url_display),
                    )
                    .child(
                        div()
                            .id("cancel-x")
                            .size(px(22.0))
                            .rounded(px(11.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .text_color(text_dim)
                            .text_sm()
                            .child("✕")
                            .on_click(cx.listener(|_this, _ev, window, _cx| {
                                window.remove_window();
                            })),
                    ),
            )
    }
}

fn daemon_browser_icon(browser: &Browser) -> gpui::AnyElement {
    use gpui::IntoElement;

    // On macOS, try quick candidate locations for the .app bundle and avoid the
    // expensive `mdfind` fallback (which spawns a process). Results are cached
    // so subsequent renders are instant.
    #[cfg(target_os = "macos")]
    {
        use std::path::Path;
        let exec = browser.exec_name();
        let candidates = vec![
            format!("/Applications/{}.app", exec),
            format!(
                "{}/Applications/{}.app",
                std::env::var("HOME").unwrap_or_default(),
                exec
            ),
        ];
        for p in &candidates {
            if Path::new(p).exists() {
                // Check cache first to avoid redundant ObjC calls / PNG decode.
                let cache = ICON_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
                if let Some(cached) = cache.lock().unwrap().get(p) {
                    let src = gpui::ImageSource::from(cached.clone());
                    return div()
                        .size(px(32.0))
                        .rounded(px(6.0))
                        .overflow_hidden()
                        .child(gpui::img(src).size(px(32.0)))
                        .into_any_element();
                }
                // Compute and cache
                if let Some(render_image) = app_icon_image(p) {
                    let cached = Arc::new(render_image);
                    cache.lock().unwrap().insert(p.clone(), cached.clone());
                    let src = gpui::ImageSource::from(cached);
                    return div()
                        .size(px(32.0))
                        .rounded(px(6.0))
                        .overflow_hidden()
                        .child(gpui::img(src).size(px(32.0)))
                        .into_any_element();
                }
            }
        }
    }

    // Fallback: colored letter badge
    let color = match browser {
        Browser::Chrome => rgb(0x42_85_F4),
        Browser::Firefox => rgb(0xFF_71_33),
        Browser::Brave => rgb(0xFB_54_23),
        Browser::Edge => rgb(0x00_78_D4),
        Browser::Safari => rgb(0x00_7A_FF),
        Browser::Arc => rgb(0x97_5B_FD),
        Browser::Orion => rgb(0x4A_90_D9),
        Browser::Other { .. } => rgb(0x88_88_88),
    };
    div()
        .size(px(32.0))
        .rounded(px(6.0))
        .bg(color)
        .flex()
        .items_center()
        .justify_center()
        .text_color(rgb(0xff_ff_ff))
        .text_xs()
        .font_weight(FontWeight::BOLD)
        .child(match browser {
            Browser::Chrome => "C",
            Browser::Firefox => "F",
            Browser::Brave => "B",
            Browser::Edge => "E",
            Browser::Safari => "S",
            Browser::Arc => "A",
            Browser::Orion => "O",
            Browser::Other { .. } => "?",
        })
        .into_any_element()
}

/// Extract the app icon as a RenderImage using NSWorkspace on macOS.
#[cfg(target_os = "macos")]
fn app_icon_image(app_path: &str) -> Option<gpui::RenderImage> {
    use cocoa::base::nil;
    use objc::{class, msg_send, sel, sel_impl};
    use std::ffi::CString;

    let png_bytes: Vec<u8> = unsafe {
        let path_cstr = CString::new(app_path).ok()?;
        let ns_string: cocoa::base::id = msg_send![
            class!(NSString),
            stringWithUTF8String: path_cstr.as_ptr()
        ];
        if ns_string == nil {
            return None;
        }

        let workspace: cocoa::base::id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let ns_image: cocoa::base::id = msg_send![workspace, iconForFile: ns_string];
        if ns_image == nil {
            return None;
        }

        // Resize to 32x32
        let target_size = cocoa::foundation::NSSize {
            width: 32.0,
            height: 32.0,
        };
        let _: () = msg_send![ns_image, setSize: target_size];

        // Get PNG representation
        let tiff: cocoa::base::id = msg_send![ns_image, TIFFRepresentation];
        if tiff == nil {
            return None;
        }
        let bitmap_rep: cocoa::base::id = msg_send![
            class!(NSBitmapImageRep),
            imageRepWithData: tiff
        ];
        if bitmap_rep == nil {
            return None;
        }

        // NSBitmapImageFileTypePNG = 4
        let props: cocoa::base::id = msg_send![class!(NSDictionary), dictionary];
        let png_data: cocoa::base::id = msg_send![
            bitmap_rep,
            representationUsingType: 4u64
            properties: props
        ];
        if png_data == nil {
            return None;
        }

        let len: usize = msg_send![png_data, length];
        let bytes: *const u8 = msg_send![png_data, bytes];
        std::slice::from_raw_parts(bytes, len).to_vec()
    };

    let img = image::load_from_memory_with_format(&png_bytes, image::ImageFormat::Png).ok()?;
    let rgba = img.into_rgba8();
    let frame = image::Frame::new(rgba);
    Some(gpui::RenderImage::new(vec![frame]))
}

fn browser_icon(browser: &Browser) -> impl IntoElement {
    let colors = match browser {
        Browser::Chrome => (rgb(0x42_85_F4), rgb(0xEA_43_35), rgb(0x34_A8_53)),
        Browser::Firefox => (rgb(0xFF_71_33), rgb(0xFF_71_33), rgb(0xFF_71_33)),
        Browser::Brave => (rgb(0xFB_54_23), rgb(0xFB_54_23), rgb(0xFB_54_23)),
        Browser::Edge => (rgb(0x00_78_D4), rgb(0x00_78_D4), rgb(0x00_78_D4)),
        Browser::Safari => (rgb(0x00_7A_FF), rgb(0x00_7A_FF), rgb(0x00_7A_FF)),
        Browser::Arc => (rgb(0x97_5B_FD), rgb(0x97_5B_FD), rgb(0x97_5B_FD)),
        Browser::Orion => (rgb(0x4A_90_D9), rgb(0x4A_90_D9), rgb(0x4A_90_D9)),
        Browser::Other { .. } => (rgb(0x88_88_88), rgb(0x88_88_88), rgb(0x88_88_88)),
    };

    div()
        .size(px(20.0))
        .flex()
        .items_center()
        .justify_center()
        .gap(px(2.0))
        .child(div().w(px(6.0)).h(px(16.0)).bg(colors.0).rounded(px(2.0)))
        .child(div().w(px(6.0)).h(px(16.0)).bg(colors.1).rounded(px(2.0)))
        .child(div().w(px(6.0)).h(px(16.0)).bg(colors.2).rounded(px(2.0)))
}

pub fn run_selector_standalone(url: &str, browsers: &[Browser]) -> Option<SelectorResult> {
    let b_list = if browsers.is_empty() {
        vec![Browser::Chrome, Browser::Firefox, Browser::Safari]
    } else {
        browsers.to_vec()
    };

    let url_owned = url.to_string();
    let domain = url::Url::parse(url)
        .and_then(|u| {
            u.host_str()
                .map(|s| s.to_string())
                .ok_or(url::ParseError::InvalidDomainCharacter)
        })
        .unwrap_or_default();
    let domain_clone = domain.clone();

    let outcome: Rc<RefCell<Option<SelectorResult>>> = Rc::new(RefCell::new(None));
    let outcome_clone = outcome.clone();

    Application::new()
        .with_assets(AppAssets::new())
        .run(move |cx: &mut App| {
            let bounds = Bounds::centered(None, size(px(420.0), px(460.0)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Browseraptor — Select Browser".into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    ..Default::default()
                },
                |_, cx| cx.new(|_| Selector::new(url_owned, domain, b_list, outcome_clone)),
            )
            .unwrap();
        });

    let result = outcome.borrow().clone();
    if let Some(SelectorResult::Selected {
        ref browser,
        remember,
    }) = result
    {
        if remember && !domain_clone.is_empty() {
            if let Ok(mut config) = Config::load() {
                config.remembered.push(crate::config::Remembered {
                    domain: domain_clone,
                    browser: browser.clone(),
                });
                let _ = config.save();
            }
        }
    }

    result
}
