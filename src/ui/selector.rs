use std::cell::RefCell;
use std::rc::Rc;

#[cfg(target_os = "macos")]
use std::collections::HashMap;
#[cfg(target_os = "macos")]
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
#[cfg(target_os = "macos")]
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
#[derive(Clone)]
struct ConfigRuleForm {
    domain_pattern: String,
    browser: String,
    profile: String,
}

pub struct DaemonSelector {
    url: gpui::SharedString,
    items: Vec<(gpui::SharedString, Browser)>,
    selected_idx: usize,
    editing_hotkey: Option<usize>,
    hovered_key: Option<usize>,
    hovered_icon: Option<usize>,
    hotkeys: std::collections::HashMap<String, String>,
    focus_handle: gpui::FocusHandle,
    command_tx: std::sync::mpsc::Sender<crate::app::AppCommand>,
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
    dev_plugin_error: Option<String>,
    dev_plugin_loading: bool,
    expanded_plugin: Option<usize>,
    // UI de configuracion del plugin
    showing_config: bool,
    config_plugin_idx: usize,
    config_default_browser: String,
    config_default_profile: String,
    config_rules: Vec<ConfigRuleForm>,
    config_focus_kind: u8, // 0=none, 1=default_browser, 2=default_profile, 3=rule_field
    config_focus_rule: usize, // indice de regla cuando focus_kind==3
    config_focus_field: u8, // 0=domain, 1=browser, 2=profile cuando focus_kind==3
    config_browser_dropdown_idx: usize,
    config_save_error: Option<String>,
}

#[allow(dead_code)]
impl DaemonSelector {
    pub fn new(
        url: String,
        _domain: String,
        _browsers: Vec<Browser>,
        command_tx: std::sync::mpsc::Sender<crate::app::AppCommand>,
        cx: &mut gpui::Context<Self>,
    ) -> Self {
        let config = crate::config::Config::load().unwrap_or_default();
        // Only use manually added browsers from config (no auto-detection).
        let items: Vec<(gpui::SharedString, Browser)> = config
            .custom_browsers
            .iter()
            .map(|b| (gpui::SharedString::from(b.name().to_owned()), b.clone()))
            .collect();
        Self {
            url: url.into(),
            items,
            selected_idx: 0,
            editing_hotkey: None,
            hovered_key: None,
            hovered_icon: None,
            hotkeys: config.hotkeys,
            focus_handle: cx.focus_handle(),
            command_tx,
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
            dev_plugin_error: None,
            dev_plugin_loading: false,
            expanded_plugin: None,
            showing_config: false,
            config_plugin_idx: 0,
            config_default_browser: String::new(),
            config_default_profile: String::new(),
            config_rules: Vec::new(),
            config_focus_kind: 0,
            config_focus_rule: 0,
            config_focus_field: 0,
            config_browser_dropdown_idx: 0,
            config_save_error: None,
        }
    }

    fn shortcut_char(browser: &Browser) -> &'static str {
        match browser {
            Browser::Chrome { .. } => "C",
            Browser::Firefox { .. } => "F",
            Browser::Brave { .. } => "B",
            Browser::Edge { .. } => "E",
            Browser::Safari { .. } => "S",
            Browser::Arc { .. } => "A",
            Browser::Orion { .. } => "O",
            Browser::Other { .. } => "?",
        }
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
            .enumerate()
            .map(|(i, p)| {
                let name = p.name.clone();
                let ver = p.version.clone();
                let desc = p.description.clone().unwrap_or_default();
                let is_dev = p.local_wasm_path.is_some();
                let is_expanded = self.expanded_plugin == Some(i);

                let row = div()
                    .id(("plugin-installed-row", i))
                    .flex()
                    .items_center()
                    .gap_3()
                    .flex_grow()
                    .min_w(px(0.0))
                    .px(px(12.0))
                    .py(px(8.0))
                    .bg(if is_expanded { rgb(0x22_2a_2a) } else { row_bg })
                    .rounded(px(6.0))
                    .cursor_pointer()
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
                            .min_w(px(0.0))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(text)
                                    .child(name),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(text_dim)
                                    .overflow_hidden()
                                    .min_w(px(0.0))
                                    .child(desc),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(text_dim)
                            .child(format!("v{}", ver)),
                    )
                    .child(if is_dev {
                        div()
                            .px(px(4.0))
                            .py(px(1.0))
                            .ml(px(4.0))
                            .bg(rgb(0x2d_6b_3a))
                            .rounded(px(3.0))
                            .text_color(green)
                            .text_xs()
                            .child("Dev")
                            .into_any_element()
                    } else {
                        div().into_any_element()
                    })
                    .child(
                        div()
                            .id(("expand-plugin", i))
                            .size(px(18.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_xs()
                            .text_color(text_dim)
                            .child(if is_expanded { "▲" } else { "▼" }),
                    )
                    .on_click(cx.listener(move |this, _ev, _window, cx| {
                        if this.expanded_plugin == Some(i) {
                            this.expanded_plugin = None;
                        } else {
                            this.expanded_plugin = Some(i);
                        }
                        cx.notify();
                    }));

                // Contenedor principal
                let mut col = div().flex().flex_col().gap_1();
                col = col.child(row);

                // Panel expandido con detalles
                if is_expanded {
                    let mut detail_col = div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .px(px(12.0))
                        .py(px(10.0))
                        .ml(px(40.0))
                        .bg(rgb(0x22_22_22))
                        .rounded(px(6.0))
                        .min_w(px(0.0));

                    // ID
                    detail_col = detail_col.child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(text_dim)
                                    .flex_shrink()
                                    .child("ID:"),
                            )
                            .child(
                                div()
                                    .flex_grow()
                                    .min_w(px(0.0))
                                    .text_xs()
                                    .text_color(text)
                                    .child(p.id.clone()),
                            ),
                    );

                    // Descripcion completa
                    if let Some(ref desc_full) = p.description {
                        detail_col = detail_col.child(
                            div()
                                .flex()
                                .gap_2()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(text_dim)
                                        .flex_shrink()
                                        .child("Desc:"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .flex_grow()
                                        .min_w(px(0.0))
                                        .overflow_hidden()
                                        .text_xs()
                                        .text_color(text)
                                        .child(desc_full.clone()),
                                ),
                        );
                    }

                    // Ruta local (solo dev)
                    if let Some(ref path) = p.local_wasm_path {
                        detail_col = detail_col.child(
                            div()
                                .flex()
                                .gap_2()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(text_dim)
                                        .flex_shrink()
                                        .child("WASM:"),
                                )
                                .child(
                                    div()
                                        .flex_grow()
                                        .min_w(px(0.0))
                                        .text_xs()
                                        .text_color(text_dim)
                                        .overflow_hidden()
                                        .child(path.clone()),
                                ),
                        );
                    }

                    // Botones de accion
                    let mut actions = div().flex().items_center().gap_2().mt(px(4.0));

                    // Abrir carpeta (solo dev)
                    if let Some(ref path) = p.local_wasm_path {
                        let dir_path = std::path::Path::new(path)
                            .parent()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_default();
                        let dir_path_clone = dir_path.clone();
                        actions = actions.child(
                            div()
                                .id(("open-plugin-dir", i))
                                .px(px(8.0))
                                .py(px(4.0))
                                .rounded(px(4.0))
                                .bg(accent)
                                .text_xs()
                                .text_color(rgb(0xff_ff_ff))
                                .cursor_pointer()
                                .child("Abrir carpeta")
                                .on_click(cx.listener(move |_this, _ev, _window, _cx| {
                                    #[cfg(target_os = "macos")]
                                    {
                                        let _ = std::process::Command::new("open")
                                            .arg(&dir_path_clone)
                                            .spawn();
                                    }
                                })),
                        );
                    }

                    // Configurar plugin
                    if p.local_wasm_path.is_some() {
                        actions = actions.child(
                            div()
                                .id(("configure-plugin-btn", i))
                                .px(px(8.0))
                                .py(px(4.0))
                                .rounded(px(4.0))
                                .bg(rgb(0x2a_4a_2a))
                                .text_xs()
                                .text_color(green)
                                .cursor_pointer()
                                .child("Configurar")
                                .on_click(cx.listener(move |this, _ev, _window, cx| {
                                    cx.stop_propagation();
                                    if i >= this.installed_plugins.len() {
                                        return;
                                    }
                                    let installed = this.installed_plugins[i].clone();
                                    let wasm_path =
                                        installed.local_wasm_path.clone().unwrap_or_default();
                                    match crate::plugin::load_installed_plugin(&installed) {
                                        Ok(plugin) => {
                                            tracing::warn!(
                                                "plugin loaded: {}, has get_config? {}",
                                                plugin.manifest.id,
                                                plugin.has_function("get_config")
                                            );
                                            match plugin.get_config() {
                                                Ok(json_str) => {
                                                    tracing::warn!(
                                                        "get_config OK, len={}",
                                                        json_str.len()
                                                    );
                                                    if let Ok(val) =
                                                        serde_json::from_str::<serde_json::Value>(
                                                            &json_str,
                                                        )
                                                    {
                                                        let default_browser = val
                                                            .get("default_browser")
                                                            .and_then(|v| v.as_str())
                                                            .unwrap_or("")
                                                            .to_string();
                                                        let default_profile = val
                                                            .get("default_profile")
                                                            .and_then(|v| v.as_str())
                                                            .unwrap_or("")
                                                            .to_string();
                                                        let rules = val
                                                            .get("rules")
                                                            .and_then(|v| v.as_array())
                                                            .map(|arr| {
                                                                arr.iter()
                                                                    .map(|r| ConfigRuleForm {
                                                                        domain_pattern: r
                                                                            .get("domain_pattern")
                                                                            .and_then(|v| {
                                                                                v.as_str()
                                                                            })
                                                                            .unwrap_or("")
                                                                            .to_string(),
                                                                        browser: r
                                                                            .get("browser")
                                                                            .and_then(|v| {
                                                                                v.as_str()
                                                                            })
                                                                            .unwrap_or("")
                                                                            .to_string(),
                                                                        profile: r
                                                                            .get("profile")
                                                                            .and_then(|v| {
                                                                                v.as_str()
                                                                            })
                                                                            .unwrap_or("")
                                                                            .to_string(),
                                                                    })
                                                                    .collect()
                                                            })
                                                            .unwrap_or_default();
                                                        this.config_plugin_idx = i;
                                                        this.config_default_browser =
                                                            default_browser;
                                                        this.config_default_profile =
                                                            default_profile;
                                                        this.config_rules = rules;
                                                        this.config_focus_kind = 0;
                                                        this.config_save_error = None;
                                                        this.showing_config = true;
                                                        cx.notify();
                                                    } else {
                                                        this.config_plugin_idx = i;
                                                        this.config_save_error = Some(format!(
                                                            "Error: JSON invalido del plugin: {}",
                                                            &json_str[..json_str.len().min(200)]
                                                        ));
                                                        this.showing_config = true;
                                                        cx.notify();
                                                    }
                                                }
                                                Err(e) => {
                                                    this.config_plugin_idx = i;
                                                    this.config_save_error = Some(format!(
                                                        "Error al leer config: {} (wasm: {})",
                                                        e, wasm_path
                                                    ));
                                                    this.showing_config = true;
                                                    cx.notify();
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            this.config_plugin_idx = i;
                                            this.config_save_error = Some(format!(
                                                "Error al cargar plugin: {} (wasm: {})",
                                                e, wasm_path
                                            ));
                                            this.showing_config = true;
                                            cx.notify();
                                        }
                                    }
                                })),
                        );
                    }

                    // Desinstalar
                    actions = actions.child(
                        div()
                            .id(("uninstall-plugin-btn", i))
                            .px(px(8.0))
                            .py(px(4.0))
                            .rounded(px(4.0))
                            .bg(rgb(0x5a_1a_1a))
                            .text_xs()
                            .text_color(red)
                            .cursor_pointer()
                            .child("Desinstalar")
                            .on_click(cx.listener(move |this, _ev, _window, cx| {
                                if i < this.installed_plugins.len() {
                                    this.installed_plugins.remove(i);
                                    this.expanded_plugin = None;
                                    this.showing_config = false;
                                    if let Ok(mut config) = crate::config::Config::load() {
                                        config.installed_plugins = this.installed_plugins.clone();
                                        let _ = config.save();
                                    }
                                    cx.notify();
                                }
                            })),
                    );

                    detail_col = detail_col.child(actions);
                    col = col.child(detail_col);
                }

                col.into_any_element()
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
                                        .min_w(px(0.0))
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(text)
                                                .child(name),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(text_dim)
                                                .overflow_hidden()
                                                .min_w(px(0.0))
                                                .child(desc),
                                        ),
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
                                                local_wasm_path: None,
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
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(text_dim)
                            .flex_grow()
                            .child("INSTALADOS"),
                    )
                    // Install Dev Plugin button
                    .child(
                        div()
                            .id("install-dev-btn")
                            .px(px(6.0))
                            .py(px(2.0))
                            .rounded(px(4.0))
                            .bg(rgb(0x2d_6b_3a))
                            .text_xs()
                            .text_color(green)
                            .cursor_pointer()
                            .child("+ Dev")
                            .on_click(cx.listener(|this, _ev, _window, cx| {
                                if this.dev_plugin_loading {
                                    return;
                                }
                                this.dev_plugin_loading = true;
                                this.dev_plugin_error = None;
                                cx.notify();

                                match install_dev_plugin() {
                                    Ok(plugin) => {
                                        this.installed_plugins.push(plugin.clone());
                                        if let Ok(mut config) = crate::config::Config::load() {
                                            config.installed_plugins.push(plugin);
                                            let _ = config.save();
                                        }
                                        this.dev_plugin_loading = false;
                                        cx.notify();
                                    }
                                    Err(e) if e == "cancelado" => {
                                        this.dev_plugin_loading = false;
                                        cx.notify();
                                    }
                                    Err(e) => {
                                        this.dev_plugin_error = Some(e);
                                        this.dev_plugin_loading = false;
                                        cx.notify();
                                    }
                                }
                            })),
                    ),
            )
            .child(
                if self.installed_plugins.is_empty() && self.dev_plugin_error.is_none() {
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
                },
            )
            // Dev plugin error message
            .child(if let Some(ref err) = self.dev_plugin_error {
                div()
                    .px(px(10.0))
                    .py(px(6.0))
                    .bg(rgb(0x3a_1a_1a))
                    .rounded(px(6.0))
                    .text_xs()
                    .text_color(red)
                    .child(format!("Error: {}", err))
                    .into_any_element()
            } else {
                div().into_any_element()
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

    // ── Config UI ─────────────────────────────────────────────────────

    /// Guarda la configuracion del plugin llamando a set_config() en el WASM.
    fn save_config(&mut self, cx: &mut gpui::Context<Self>) {
        let idx = self.config_plugin_idx;
        if idx >= self.installed_plugins.len() {
            self.config_save_error = Some("Plugin not found".to_string());
            cx.notify();
            return;
        }

        // Construir JSON con los valores del formulario
        let rules: Vec<serde_json::Value> = self
            .config_rules
            .iter()
            .map(|r| {
                let mut obj = serde_json::json!({
                    "domain_pattern": r.domain_pattern,
                    "browser": r.browser,
                });
                if !r.profile.is_empty() {
                    obj.as_object_mut().unwrap().insert(
                        "profile".to_string(),
                        serde_json::Value::String(r.profile.clone()),
                    );
                }
                obj
            })
            .collect();

        let config_json = serde_json::json!({
            "rules": rules,
            "default_browser": if self.config_default_browser.is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::Value::String(self.config_default_browser.clone())
            },
            "default_profile": if self.config_default_profile.is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::Value::String(self.config_default_profile.clone())
            },
        });

        let json_str = serde_json::to_string_pretty(&config_json).unwrap();

        match crate::plugin::load_installed_plugin(&self.installed_plugins[idx]) {
            Ok(plugin) => match plugin.set_config(&json_str) {
                Ok(resp) => {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&resp) {
                        if val.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                            self.showing_config = false;
                            self.config_save_error = None;
                        } else {
                            let err = val
                                .get("error")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown error");
                            self.config_save_error = Some(err.to_string());
                        }
                    } else {
                        self.config_save_error = Some(format!("Respuesta invalida: {}", resp));
                    }
                    cx.notify();
                }
                Err(e) => {
                    self.config_save_error = Some(format!("Error al guardar: {}", e));
                    cx.notify();
                }
            },
            Err(e) => {
                self.config_save_error = Some(format!("Error al cargar plugin: {}", e));
                cx.notify();
            }
        }
    }

    /// Renderiza el panel de configuracion del plugin.
    fn render_config_panel(
        &mut self,
        text: gpui::Rgba,
        text_dim: gpui::Rgba,
        cx: &mut gpui::Context<Self>,
    ) -> gpui::AnyElement {
        use gpui::IntoElement;

        let accent = rgb(0x4a_90_d9);
        let badge_bg = rgb(0x3a_3a_3a);
        let green = rgb(0x34_c7_59);
        let red = rgb(0xff_45_45);
        let field_bg = rgb(0x22_22_22);
        let field_focus_bg = rgb(0x38_38_38);

        let plugin_name = self
            .installed_plugins
            .get(self.config_plugin_idx)
            .map(|p| p.name.clone())
            .unwrap_or_default();

        // Helper: construye display string para campo editable
        fn field_display(value: &str, focused: bool, placeholder: &str) -> String {
            if value.is_empty() {
                if focused {
                    "|".to_string()
                } else {
                    placeholder.to_string()
                }
            } else if focused {
                format!("{}|", value)
            } else {
                value.to_string()
            }
        }

        // ── Panel scrollable ────────────────────────────────────────
        let mut panel = div()
            .id("config-panel")
            .flex()
            .flex_col()
            .flex_grow()
            .overflow_y_scroll()
            .gap_3()
            .px(px(14.0))
            .pt(px(12.0))
            .pb(px(8.0))
            // Header
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
                            .child(format!("Configurar: {}", plugin_name)),
                    )
                    .child(
                        div()
                            .id("config-close")
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
                                this.showing_config = false;
                                this.config_save_error = None;
                                cx.notify();
                            })),
                    ),
            );

        // Default browser - select dropdown
        let default_browser_display = if self.config_default_browser.is_empty() {
            "Select browser…".to_string()
        } else {
            format!("{} ", self.config_default_browser)
        };
        let mut db_row = div()
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .text_xs()
                    .text_color(text_dim)
                    .child("Default Browser"),
            )
            .child(
                div()
                    .id("cfg-default-browser")
                    .px(px(8.0))
                    .py(px(4.0))
                    .bg(if self.config_focus_kind == 1 {
                        field_focus_bg
                    } else {
                        field_bg
                    })
                    .rounded(px(4.0))
                    .flex_grow()
                    .min_w(px(40.0))
                    .text_xs()
                    .text_color(text)
                    .cursor_pointer()
                    .child(default_browser_display)
                    .on_click(cx.listener(|this, _ev, _window, cx| {
                        this.config_focus_kind = 1;
                        this.config_browser_dropdown_idx = 0;
                        cx.notify();
                    })),
            );
        if self.config_focus_kind == 1 && !self.items.is_empty() {
            let mut dd = div()
                .flex()
                .flex_col()
                .mx(px(4.0))
                .rounded(px(4.0))
                .overflow_hidden()
                .bg(rgb(0x2a_2a_2e));
            for (di, (name, _browser)) in self.items.iter().enumerate() {
                let is_hl = di == self.config_browser_dropdown_idx;
                let browser_name = name.to_string();
                let mut dd_item = div()
                    .id(("cfg-browser-opt", di))
                    .px(px(8.0))
                    .py(px(4.0))
                    .text_xs()
                    .text_color(text);
                if is_hl {
                    dd_item = dd_item.bg(rgb(0x3a_3a_3e));
                }
                let bn_display = browser_name.clone();
                let dd_item = dd_item
                    .cursor_pointer()
                    .child(bn_display)
                    .on_click(cx.listener(move |this, _ev, _window, cx| {
                        this.config_default_browser = browser_name.clone();
                        this.config_focus_kind = 2;
                        cx.notify();
                    }));
                dd = dd.child(dd_item);
            }
            db_row = db_row.child(dd);
        }
        panel = panel.child(db_row);

        // Default profile
        panel = panel.child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_xs()
                        .text_color(text_dim)
                        .child("Default Profile"),
                )
                .child({
                    let display = field_display(
                        &self.config_default_profile,
                        self.config_focus_kind == 2,
                        "default…",
                    );
                    div()
                        .id("cfg-default-profile")
                        .px(px(8.0))
                        .py(px(4.0))
                        .bg(if self.config_focus_kind == 2 {
                            field_focus_bg
                        } else {
                            field_bg
                        })
                        .rounded(px(4.0))
                        .flex_grow()
                        .min_w(px(40.0))
                        .text_xs()
                        .text_color(text)
                        .cursor_pointer()
                        .child(display)
                        .on_click(cx.listener(|this, _ev, _window, cx| {
                            this.config_focus_kind = 2;
                            cx.notify();
                        }))
                        .into_any_element()
                }),
        );

        // Rules header
        panel = panel.child(
            div()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(text_dim)
                .child("Rules"),
        );

        // Cada regla
        let rules_len = self.config_rules.len();
        for ri in 0..rules_len {
            let rule = &self.config_rules[ri];
            let focused = self.config_focus_kind == 3 && self.config_focus_rule == ri;
            let f_domain = focused && self.config_focus_field == 0;
            let f_browser = focused && self.config_focus_field == 1;
            let f_profile = focused && self.config_focus_field == 2;

            let val_domain = rule.domain_pattern.clone();
            let val_browser = rule.browser.clone();
            let val_profile = rule.profile.clone();

            let mut row = div().id(("cfg-rule-row", ri)).flex().items_center().gap_1();

            // Domain
            row = row.child({
                let d = field_display(&val_domain, f_domain, "domain");
                div()
                    .id(("cfg-rule-{}-domain", ri))
                    .px(px(8.0))
                    .py(px(4.0))
                    .bg(if f_domain { field_focus_bg } else { field_bg })
                    .rounded(px(4.0))
                    .flex_grow()
                    .min_w(px(40.0))
                    .text_xs()
                    .text_color(text)
                    .cursor_pointer()
                    .child(d)
                    .on_click(cx.listener(move |this, _ev, _window, cx| {
                        this.config_focus_kind = 3;
                        this.config_focus_rule = ri;
                        this.config_focus_field = 0;
                        cx.notify();
                    }))
                    .into_any_element()
            });

            row = row.child(div().text_xs().text_color(text_dim).child("→"));

            // Browser - select dropdown
            let browser_display = if val_browser.is_empty() {
                "Select…".to_string()
            } else {
                val_browser.clone()
            };
            let is_browser_focused = f_browser;
            let mut browser_cell = div().flex().flex_col().flex_grow().child(
                div()
                    .id(("cfg-rule-{}-browser", ri))
                    .px(px(8.0))
                    .py(px(4.0))
                    .bg(if is_browser_focused {
                        field_focus_bg
                    } else {
                        field_bg
                    })
                    .rounded(px(4.0))
                    .min_w(px(40.0))
                    .text_xs()
                    .text_color(text)
                    .cursor_pointer()
                    .child(browser_display)
                    .on_click(cx.listener(move |this, _ev, _window, cx| {
                        this.config_focus_kind = 3;
                        this.config_focus_rule = ri;
                        this.config_focus_field = 1;
                        this.config_browser_dropdown_idx = 0;
                        cx.notify();
                    })),
            );
            if is_browser_focused && !self.items.is_empty() {
                let mut dd = div()
                    .flex()
                    .flex_col()
                    .mx(px(4.0))
                    .rounded(px(4.0))
                    .overflow_hidden()
                    .bg(rgb(0x2a_2a_2e));
                for (di, (n, _)) in self.items.iter().enumerate() {
                    let is_hl = di == self.config_browser_dropdown_idx;
                    let browser_name = n.to_string();
                    let flat_id = ri * 10000 + di;
                    let mut dd_item = div()
                        .id(("cfg-rbo", flat_id))
                        .px(px(8.0))
                        .py(px(4.0))
                        .text_xs()
                        .text_color(text);
                    if is_hl {
                        dd_item = dd_item.bg(rgb(0x3a_3a_3e));
                    }
                    let bn_display = browser_name.clone();
                    dd_item = dd_item
                        .cursor_pointer()
                        .child(bn_display)
                        .on_click(cx.listener(move |this, _ev, _window, cx| {
                            this.config_focus_kind = 3;
                            this.config_focus_rule = ri;
                            this.config_focus_field = 2;
                            this.config_rules[ri].browser = browser_name.clone();
                            cx.notify();
                        }));
                    dd = dd.child(dd_item);
                }
                browser_cell = browser_cell.child(dd);
            }
            row = row.child(browser_cell.into_any_element());

            // Profile
            row = row.child({
                let d = field_display(&val_profile, f_profile, "profile");
                div()
                    .id(("cfg-rule-{}-profile", ri))
                    .px(px(8.0))
                    .py(px(4.0))
                    .bg(if f_profile { field_focus_bg } else { field_bg })
                    .rounded(px(4.0))
                    .flex_grow()
                    .min_w(px(40.0))
                    .text_xs()
                    .text_color(text)
                    .cursor_pointer()
                    .child(d)
                    .on_click(cx.listener(move |this, _ev, _window, cx| {
                        this.config_focus_kind = 3;
                        this.config_focus_rule = ri;
                        this.config_focus_field = 2;
                        cx.notify();
                    }))
                    .into_any_element()
            });

            // Remove button
            row = row.child(
                div()
                    .id(("cfg-rule-remove", ri))
                    .size(px(18.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_xs()
                    .text_color(red)
                    .cursor_pointer()
                    .child("✕")
                    .on_click(cx.listener(move |this, _ev, _window, cx| {
                        if ri < this.config_rules.len() {
                            this.config_rules.remove(ri);
                            if this.config_focus_kind == 3 {
                                if this.config_focus_rule == ri && this.config_focus_rule > 0 {
                                    this.config_focus_rule -= 1;
                                } else if this.config_focus_rule >= this.config_rules.len() {
                                    this.config_focus_rule =
                                        this.config_rules.len().saturating_sub(1);
                                }
                            }
                            cx.notify();
                        }
                    })),
            );

            panel = panel.child(row);
        }

        // Add rule button
        panel = panel.child(
            div()
                .id("cfg-add-rule")
                .flex()
                .items_center()
                .justify_center()
                .px(px(8.0))
                .py(px(6.0))
                .bg(rgb(0x22_2a_22))
                .rounded(px(4.0))
                .text_xs()
                .text_color(green)
                .cursor_pointer()
                .child("+ Add Rule")
                .on_click(cx.listener(|this, _ev, _window, cx| {
                    this.config_rules.push(ConfigRuleForm {
                        domain_pattern: String::new(),
                        browser: String::new(),
                        profile: String::new(),
                    });
                    let last = this.config_rules.len() - 1;
                    this.config_focus_kind = 3;
                    this.config_focus_rule = last;
                    this.config_focus_field = 0;
                    cx.notify();
                })),
        );

        // Error message
        if let Some(ref err) = self.config_save_error {
            let err_clone = err.clone();
            panel = panel.child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px(px(8.0))
                    .py(px(4.0))
                    .bg(rgb(0x3a_1a_1a))
                    .rounded(px(4.0))
                    .child(
                        div()
                            .flex_grow()
                            .text_xs()
                            .text_color(red)
                            .overflow_hidden()
                            .child(err.clone()),
                    )
                    .child(
                        div()
                            .id("copy-error-btn")
                            .px(px(4.0))
                            .py(px(1.0))
                            .rounded(px(3.0))
                            .bg(rgb(0x5a_3a_3a))
                            .text_xs()
                            .text_color(red)
                            .cursor_pointer()
                            .child("copy")
                            .on_click(cx.listener(move |_this, _ev, _window, _cx| {
                                #[cfg(target_os = "macos")]
                                {
                                    use std::process::{Command, Stdio};
                                    let mut child = Command::new("pbcopy")
                                        .stdin(Stdio::piped())
                                        .spawn()
                                        .expect("failed to launch pbcopy");
                                    use std::io::Write;
                                    let _ = child.stdin.take().map(|mut stdin| {
                                        let _ = stdin.write_all(err_clone.as_bytes());
                                    });
                                    let _ = child.wait();
                                }
                            })),
                    ),
            );
        }

        // Save + Cancel buttons
        panel = panel.child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .id("config-save-btn")
                        .flex_grow()
                        .px(px(8.0))
                        .py(px(4.0))
                        .bg(accent)
                        .rounded(px(4.0))
                        .text_xs()
                        .text_color(rgb(0xff_ff_ff))
                        .cursor_pointer()
                        .child("Save")
                        .on_click(cx.listener(|this, _ev, _window, cx| {
                            this.save_config(cx);
                        })),
                )
                .child(
                    div()
                        .id("config-cancel-btn")
                        .px(px(8.0))
                        .py(px(4.0))
                        .bg(badge_bg)
                        .rounded(px(4.0))
                        .text_xs()
                        .text_color(text_dim)
                        .cursor_pointer()
                        .child("Cancel")
                        .on_click(cx.listener(|this, _ev, _window, cx| {
                            this.showing_config = false;
                            this.config_save_error = None;
                            cx.notify();
                        })),
                ),
        );

        panel.into_any_element()
    }
}

impl gpui::Render for DaemonSelector {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        use gpui::IntoElement;
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

        let url_str = self.url.as_ref();
        let url_display: gpui::SharedString = if url_str.is_empty() {
            "Browseraptor".into()
        } else if url_str.len() > 52 {
            format!("{}…", &url_str[..52]).into()
        } else {
            self.url.clone()
        };

        let rows: Vec<_> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, (name, browser))| {
                let is_sel = i == selected;
                let is_editing = editing == Some(i);
                let is_hovered = hovered == Some(i);
                let is_icon_hovered = self.hovered_icon == Some(i);
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
                    .child(if is_icon_hovered {
                        div()
                            .id(("icon-remove", i))
                            .size(px(32.0))
                            .rounded(px(6.0))
                            .bg(rgb(0x5a_1a_1a))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(0xff_45_45))
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .cursor_pointer()
                            .child("✕")
                            .on_mouse_move(cx.listener(move |this, _ev, _window, cx| {
                                if this.hovered_icon != Some(i) {
                                    this.hovered_icon = Some(i);
                                    cx.notify();
                                }
                                cx.stop_propagation();
                            }))
                            .on_mouse_up(
                                gpui::MouseButton::Left,
                                cx.listener(move |this, _ev, _window, cx| {
                                    if i < this.items.len() {
                                        let name = this.items[i].1.name().to_owned();
                                        this.items.remove(i);
                                        this.hovered_icon = None;
                                        if i <= this.selected_idx && this.selected_idx > 0 {
                                            this.selected_idx -= 1;
                                        }
                                        // Remove from config
                                        if let Ok(mut config) = crate::config::Config::load() {
                                            config.custom_browsers.retain(|b| b.name() != name);
                                            let _ = config.save();
                                        }
                                        cx.notify();
                                    }
                                }),
                            )
                            .into_any_element()
                    } else {
                        div()
                            .on_mouse_move(cx.listener(move |this, _ev, _window, cx| {
                                if this.hovered_icon != Some(i) {
                                    this.hovered_icon = Some(i);
                                    cx.notify();
                                }
                                cx.stop_propagation();
                            }))
                            .child(daemon_browser_icon(&browser))
                            .into_any_element()
                    })
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

        // Add Browser button – opens file picker
        let border_color = rgb(0x3a_3a_3e);
        let accent_add = rgb(0x4a_90_d9);
        let add_browser_btn = div()
            .id("add-browser")
            .flex()
            .items_center()
            .gap_3()
            .px(px(16.0))
            .py(px(13.0))
            .mx(px(8.0))
            .bg(row_normal)
            .rounded(px(10.0))
            .border_1()
            .border_color(border_color)
            .cursor_pointer()
            .child(
                div()
                    .size(px(28.0))
                    .rounded(px(8.0))
                    .bg(accent_add)
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(rgb(0xff_ff_ff))
                    .text_base()
                    .font_weight(FontWeight::BOLD)
                    .child("+"),
            )
            .child(
                div()
                    .flex_grow()
                    .text_color(text_dim)
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .child("Add Browser..."),
            )
            .on_click(cx.listener(move |this, _ev, _window, cx| {
                if let Some(app_path) = pick_browser_app() {
                    let name = browser_name_from_app(&app_path);
                    // Skip if already added
                    if this.items.iter().any(|(_, b)| b.name() == name) {
                        return;
                    }
                    let browser = Browser::Other {
                        name: name.clone(),
                        app_path: Some(app_path.clone()),
                    };
                    this.items
                        .push((gpui::SharedString::from(name.clone()), browser.clone()));
                    // Save to config
                    if let Ok(mut config) = crate::config::Config::load() {
                        config.custom_browsers.push(browser);
                        let _ = config.save();
                    }
                    cx.notify();
                }
            }));

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
                if this.hovered_key.is_some() || this.hovered_icon.is_some() {
                    this.hovered_key = None;
                    this.hovered_icon = None;
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

                // ── Config form editing mode ──────────────────────────
                if this.showing_config {
                    match key.as_str() {
                        "escape" => {
                            if this.config_focus_kind == 1
                                || (this.config_focus_kind == 3 && this.config_focus_field == 1)
                            {
                                // Close dropdown without selecting
                                this.config_focus_kind = 0;
                            } else {
                                this.showing_config = false;
                                this.config_save_error = None;
                            }
                            cx.notify();
                        }
                        "backspace" => {
                            match this.config_focus_kind {
                                1 => {}
                                2 => {
                                    this.config_default_profile.pop();
                                }
                                3 => {
                                    let r = this.config_focus_rule;
                                    if r < this.config_rules.len() {
                                        match this.config_focus_field {
                                            0 => {
                                                this.config_rules[r].domain_pattern.pop();
                                            }
                                            1 => {}
                                            _ => {
                                                this.config_rules[r].profile.pop();
                                            }
                                        };
                                    }
                                }
                                _ => {}
                            }
                            cx.notify();
                        }
                        "tab" => {
                            // Cycle focus: default_browser → default_profile → rules → wrap
                            match this.config_focus_kind {
                                0 | 1 => {
                                    // Select current browser in dropdown, move to profile
                                    if !this.items.is_empty()
                                        && this.config_browser_dropdown_idx < this.items.len()
                                    {
                                        this.config_default_browser = this.items
                                            [this.config_browser_dropdown_idx]
                                            .0
                                            .to_string();
                                    }
                                    this.config_focus_kind = 2;
                                }
                                2 => {
                                    if !this.config_rules.is_empty() {
                                        this.config_focus_kind = 3;
                                        this.config_focus_rule = 0;
                                        this.config_focus_field = 0;
                                    } else {
                                        this.config_focus_kind = 1;
                                    }
                                }
                                3 => {
                                    if this.config_focus_field == 1 {
                                        // Select browser in dropdown, move to profile
                                        if !this.items.is_empty()
                                            && this.config_browser_dropdown_idx < this.items.len()
                                        {
                                            this.config_rules[this.config_focus_rule].browser =
                                                this.items[this.config_browser_dropdown_idx]
                                                    .0
                                                    .to_string();
                                        }
                                        this.config_focus_field = 2;
                                    } else if this.config_focus_field < 2 {
                                        this.config_focus_field += 1;
                                    } else if this.config_focus_rule + 1 < this.config_rules.len() {
                                        this.config_focus_rule += 1;
                                        this.config_focus_field = 0;
                                    } else {
                                        this.config_focus_kind = 1;
                                    }
                                }
                                _ => {
                                    this.config_focus_kind = 1;
                                }
                            }
                            cx.notify();
                        }
                        "return" | "enter" => {
                            if this.config_focus_kind == 1 && !this.items.is_empty() {
                                // Select browser from dropdown, move to profile
                                if this.config_browser_dropdown_idx < this.items.len() {
                                    this.config_default_browser =
                                        this.items[this.config_browser_dropdown_idx].0.to_string();
                                }
                                this.config_focus_kind = 2;
                                cx.notify();
                                return;
                            }
                            if this.config_focus_kind == 3
                                && this.config_focus_field == 1
                                && !this.items.is_empty()
                            {
                                // Select browser in rule dropdown, move to profile
                                if this.config_browser_dropdown_idx < this.items.len() {
                                    this.config_rules[this.config_focus_rule].browser =
                                        this.items[this.config_browser_dropdown_idx].0.to_string();
                                }
                                this.config_focus_field = 2;
                                cx.notify();
                                return;
                            }
                            // Save and close
                            this.save_config(cx);
                        }
                        "up" | "arrowup" => {
                            if !this.items.is_empty()
                                && (this.config_focus_kind == 1
                                    || (this.config_focus_kind == 3
                                        && this.config_focus_field == 1))
                            {
                                if this.config_browser_dropdown_idx > 0 {
                                    this.config_browser_dropdown_idx -= 1;
                                } else {
                                    this.config_browser_dropdown_idx = this.items.len() - 1;
                                }
                                cx.notify();
                                return;
                            }
                        }
                        "down" | "arrowdown" => {
                            if !this.items.is_empty()
                                && (this.config_focus_kind == 1
                                    || (this.config_focus_kind == 3
                                        && this.config_focus_field == 1))
                            {
                                if this.config_browser_dropdown_idx + 1 < this.items.len() {
                                    this.config_browser_dropdown_idx += 1;
                                } else {
                                    this.config_browser_dropdown_idx = 0;
                                }
                                cx.notify();
                                return;
                            }
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
                            match this.config_focus_kind {
                                0 | 1 => {
                                    // Typing in default_browser: try to match browser name
                                    this.config_focus_kind = 1;
                                    this.config_default_browser.push_str(&ch);
                                    // Auto-highlight first matching browser in dropdown
                                    let query = this.config_default_browser.to_lowercase();
                                    if !query.is_empty() {
                                        for (di, (n, _)) in this.items.iter().enumerate() {
                                            if n.to_lowercase().starts_with(&query) {
                                                this.config_browser_dropdown_idx = di;
                                                break;
                                            }
                                        }
                                    }
                                }
                                2 => {
                                    this.config_default_profile.push_str(&ch);
                                }
                                3 => {
                                    let r = this.config_focus_rule;
                                    if r < this.config_rules.len() {
                                        match this.config_focus_field {
                                            0 => this.config_rules[r].domain_pattern.push_str(&ch),
                                            1 => {
                                                // Typing in rule browser: auto-highlight matching
                                                this.config_rules[r].browser.push_str(&ch);
                                                let query =
                                                    this.config_rules[r].browser.to_lowercase();
                                                if !query.is_empty() {
                                                    for (di, (n, _)) in
                                                        this.items.iter().enumerate()
                                                    {
                                                        if n.to_lowercase().starts_with(&query) {
                                                            this.config_browser_dropdown_idx = di;
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                            _ => this.config_rules[r].profile.push_str(&ch),
                                        }
                                    }
                                }
                                _ => {}
                            }
                            cx.notify();
                        }
                        _ => {}
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
            .child({
                let content: gpui::AnyElement = if self.showing_config {
                    self.render_config_panel(text, text_dim, cx)
                } else if self.show_plugin_search {
                    self.render_plugin_panel(text, text_dim, cx)
                } else {
                    div()
                        .id("browser-scroll")
                        .flex()
                        .flex_col()
                        .flex_grow()
                        .overflow_y_scroll()
                        .gap_1()
                        .pt(px(10.0))
                        .pb(px(6.0))
                        .children(rows)
                        .child(add_browser_btn)
                        .into_any_element()
                };
                content
            })
            // divider (fixed at bottom)
            .child(div().h(px(1.0)).mx(px(8.0)).bg(rgb(0x3a_3a_3a)))
            // bottom bar: URL + plugins + settings + close (fixed at bottom)
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
                                    this.plugin_query.clear();
                                    this.channel_plugins = None;
                                    // Fetch channel on open asynchronously (non-blocking).
                                    let _ =
                                        crate::plugin::fetch_channel_async(this.command_tx.clone());
                                }
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .flex_grow()
                            .overflow_hidden()
                            .whitespace_nowrap()
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

/// Render a browser's app icon (32x32).
/// If `app_path_override` is provided, it is used as the .app path first;
/// otherwise the browser's stored `app_path()` is tried, then path guessing.
fn daemon_browser_icon_with_path(
    browser: &Browser,
    app_path_override: Option<&str>,
) -> gpui::AnyElement {
    use gpui::IntoElement;

    // On macOS, try to locate the .app bundle and extract its icon.
    // Priority: 1) app_path_override  2) browser.app_path()  3) path guessing
    #[cfg(target_os = "macos")]
    {
        use std::path::Path;

        // Collect candidate paths
        let mut app_candidates: Vec<String> = Vec::new();

        // 1) Explicit override from manual selection
        if let Some(p) = app_path_override {
            app_candidates.push(p.to_string());
        }

        // 2) Stored app_path from detection
        if let Some(p) = browser.app_path() {
            if !app_candidates.iter().any(|c| c == p) {
                app_candidates.push(p.to_string());
            }
        }

        // 3) Guess from exec_name
        let exec = browser.exec_name();
        app_candidates.push(format!("/Applications/{}.app", exec));
        app_candidates.push(format!(
            "{}/Applications/{}.app",
            std::env::var("HOME").unwrap_or_default(),
            exec
        ));

        for p in &app_candidates {
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
        Browser::Chrome { .. } => rgb(0x42_85_F4),
        Browser::Firefox { .. } => rgb(0xFF_71_33),
        Browser::Brave { .. } => rgb(0xFB_54_23),
        Browser::Edge { .. } => rgb(0x00_78_D4),
        Browser::Safari { .. } => rgb(0x00_7A_FF),
        Browser::Arc { .. } => rgb(0x97_5B_FD),
        Browser::Orion { .. } => rgb(0x4A_90_D9),
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
            Browser::Chrome { .. } => "C",
            Browser::Firefox { .. } => "F",
            Browser::Brave { .. } => "B",
            Browser::Edge { .. } => "E",
            Browser::Safari { .. } => "S",
            Browser::Arc { .. } => "A",
            Browser::Orion { .. } => "O",
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
    let mut rgba = img.into_rgba8();
    // GPUI's RenderImage expects BGRA format, convert from RGBA
    for pixel in rgba.pixels_mut() {
        let [r, g, b, a] = pixel.0;
        pixel.0 = [b, g, r, a];
    }
    let frame = image::Frame::new(rgba);
    Some(gpui::RenderImage::new(vec![frame]))
}

/// Backward-compatible wrapper (no path override).
fn daemon_browser_icon(browser: &Browser) -> gpui::AnyElement {
    daemon_browser_icon_with_path(browser, None)
}

fn browser_icon(browser: &Browser) -> impl IntoElement {
    let colors = match browser {
        Browser::Chrome { .. } => (rgb(0x42_85_F4), rgb(0xEA_43_35), rgb(0x34_A8_53)),
        Browser::Firefox { .. } => (rgb(0xFF_71_33), rgb(0xFF_71_33), rgb(0xFF_71_33)),
        Browser::Brave { .. } => (rgb(0xFB_54_23), rgb(0xFB_54_23), rgb(0xFB_54_23)),
        Browser::Edge { .. } => (rgb(0x00_78_D4), rgb(0x00_78_D4), rgb(0x00_78_D4)),
        Browser::Safari { .. } => (rgb(0x00_7A_FF), rgb(0x00_7A_FF), rgb(0x00_7A_FF)),
        Browser::Arc { .. } => (rgb(0x97_5B_FD), rgb(0x97_5B_FD), rgb(0x97_5B_FD)),
        Browser::Orion { .. } => (rgb(0x4A_90_D9), rgb(0x4A_90_D9), rgb(0x4A_90_D9)),
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

/// Show a native file picker (macOS) to select a .app bundle.
/// Returns the POSIX path of the selected .app, or `None` if cancelled.
fn pick_browser_app() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("osascript")
            .args([
                "-e",
                r#"set f to choose file with prompt "Select Browser Application" of type {"app"}"#,
                "-e",
                "if f is not false then return POSIX path of f",
            ])
            .output()
            .ok()?;
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if path.is_empty() {
                None
            } else {
                Some(path)
            }
        } else {
            None
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

/// Muestra un dialogo nativo para seleccionar un archivo `.wasm`.
/// Retorna la ruta completa o `None` si se cancela.
fn pick_wasm_file() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("osascript")
            .args([
                "-e",
                "set f to choose file with prompt \"Select Plugin WASM File\"",
                "-e",
                "if f is not false then return POSIX path of f",
            ])
            .output()
            .ok()?;
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if path.is_empty() {
                None
            } else {
                Some(path)
            }
        } else {
            None
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

/// Instala un plugin de desarrollo:
/// 1. Abre un dialogo para seleccionar el `.wasm`
/// 2. Lee el `manifest.json` del mismo directorio
/// 3. Valida el modulo WASM
/// 4. Retorna el `InstalledPlugin` listo para registrar
fn install_dev_plugin() -> Result<crate::plugin::InstalledPlugin, String> {
    let wasm_path = match pick_wasm_file() {
        Some(p) => p,
        None => return Err("cancelado".to_string()),
    };

    crate::plugin::load_dev_plugin_from_path(&wasm_path)
        .map_err(|e| format!("Error al cargar plugin: {}", e))
}

/// Extract the display name from a .app bundle's Info.plist.
fn browser_name_from_app(app_path: &str) -> String {
    let plist_path = format!("{}/Contents/Info.plist", app_path);
    if std::path::Path::new(&plist_path).exists() {
        use std::process::Command;
        if let Ok(output) = Command::new("plutil")
            .args(["-convert", "xml1", "-o", "-", &plist_path])
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                // Try CFBundleDisplayName first, then CFBundleName
                let name = extract_plist_str(&text, "CFBundleDisplayName")
                    .or_else(|| extract_plist_str(&text, "CFBundleName"));
                if let Some(n) = name {
                    return n;
                }
            }
        }
    }
    // Fallback: use the filename without .app
    std::path::Path::new(app_path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| app_path.to_string())
}

fn extract_plist_str(text: &str, key: &str) -> Option<String> {
    let key_tag = format!("<key>{}</key>", key);
    let pos = text.find(&key_tag)?;
    let after = &text[pos + key_tag.len()..];
    let start = after.find("<string>")? + "<string>".len();
    let end = after[start..].find("</string>")?;
    let value = after[start..start + end].trim().to_string();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

pub fn run_selector_standalone(url: &str, browsers: &[Browser]) -> Option<SelectorResult> {
    let b_list = if browsers.is_empty() {
        vec![
            Browser::Chrome { app_path: None },
            Browser::Firefox { app_path: None },
            Browser::Safari { app_path: None },
        ]
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
