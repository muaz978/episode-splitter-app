//! Episode Part Splitter: a tiny native shell around the offline HTML tool.
//! The page is embedded in the binary and shown in the system web view
//! (WebKit on macOS, WebView2 on Windows), so the UI and logic are exactly
//! the same as when the HTML is opened in a browser.

#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
compile_error!("This app is built for macOS and Windows only.");

use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};

use serde_json::{Map, Value};
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::{Icon, Theme, WindowBuilder},
};
use wry::{
    http::{header::CONTENT_TYPE, Response},
    WebContext, WebViewBuilder,
};

const INDEX_HTML: &str = include_str!("../assets/index.html");
const SHIM_JS: &str = include_str!("shim.js");
const ICON_RGBA: &[u8] = include_bytes!("../assets/icon64.rgba");
const APP_NAME: &str = "Episode Part Splitter";
const DATA_FOLDER: &str = "EpisodeSplitter";

#[cfg(target_os = "windows")]
const START_URL: &str = "http://app.localhost/";
#[cfg(not(target_os = "windows"))]
const START_URL: &str = "app://localhost/";

#[derive(Debug)]
enum UserEvent {
    /// Result of a native clipboard write: (request id, success).
    ClipResult(u64, bool),
}

/// Where settings are kept: one small folder per user, outside the app itself.
fn data_dir() -> PathBuf {
    let base = if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"))
    };
    base.unwrap_or_else(std::env::temp_dir).join(DATA_FOLDER)
}

/// Key-value settings saved as JSON (replaces the browser's localStorage).
struct Store {
    path: PathBuf,
    map: Map<String, Value>,
}

impl Store {
    fn load(dir: &Path) -> Self {
        let path = dir.join("state.json");
        let map = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .and_then(|v| match v {
                Value::Object(m) => Some(m),
                _ => None,
            })
            .unwrap_or_default();
        Store { path, map }
    }

    /// Write to a temp file first, then swap it in, so a crash never leaves a half-written file.
    fn save(&self) {
        if let Ok(text) = serde_json::to_string(&self.map) {
            let tmp = self.path.with_extension("json.tmp");
            if fs::write(&tmp, text).is_ok() {
                let _ = fs::rename(&tmp, &self.path);
            }
        }
    }
}

fn load_geometry(dir: &Path) -> (f64, f64) {
    let size = fs::read_to_string(dir.join("window.json"))
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .and_then(|v| Some((v.get("w")?.as_f64()?, v.get("h")?.as_f64()?)));
    match size {
        Some((w, h)) if (880.0..=6000.0).contains(&w) && (600.0..=4000.0).contains(&h) => (w, h),
        _ => (1280.0, 900.0),
    }
}

fn save_geometry(dir: &Path, logical: LogicalSize<f64>) {
    if logical.width >= 300.0 && logical.height >= 300.0 {
        let text = format!(r#"{{"w":{:.0},"h":{:.0}}}"#, logical.width, logical.height);
        let _ = fs::write(dir.join("window.json"), text);
    }
}

/// Put rich text (HTML plus a plain-text twin) on the system clipboard, so pasting
/// into an email keeps the styling.
fn write_clipboard(html: Option<&str>, text: &str) -> bool {
    match arboard::Clipboard::new() {
        Ok(mut cb) => match html {
            Some(h) => cb.set().html(h, Some(text)).is_ok(),
            None => cb.set_text(text).is_ok(),
        },
        Err(_) => false,
    }
}

/// macOS needs a menu bar for Cmd+C / Cmd+V / Cmd+A / Cmd+Z to work inside text fields.
#[cfg(target_os = "macos")]
fn install_menu() -> muda::Menu {
    use muda::{Menu, PredefinedMenuItem as P, Submenu};
    let menu = Menu::new();
    let app = Submenu::new(APP_NAME, true);
    let _ = app.append_items(&[
        &P::about(None, None),
        &P::separator(),
        &P::services(None),
        &P::separator(),
        &P::hide(None),
        &P::hide_others(None),
        &P::show_all(None),
        &P::separator(),
        &P::quit(None),
    ]);
    let edit = Submenu::new("Edit", true);
    let _ = edit.append_items(&[
        &P::undo(None),
        &P::redo(None),
        &P::separator(),
        &P::cut(None),
        &P::copy(None),
        &P::paste(None),
        &P::select_all(None),
    ]);
    let window = Submenu::new("Window", true);
    let _ = window.append_items(&[&P::minimize(None), &P::maximize(None), &P::separator(), &P::close_window(None)]);
    let _ = menu.append_items(&[&app, &edit, &window]);
    menu.init_for_nsapp();
    window.set_as_windows_menu_for_nsapp();
    menu
}

fn main() -> wry::Result<()> {
    let dir = data_dir();
    let _ = fs::create_dir_all(&dir);
    let store = Rc::new(RefCell::new(Store::load(&dir)));

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    #[cfg(target_os = "macos")]
    let menu = install_menu();

    let (w, h) = load_geometry(&dir);
    let window = WindowBuilder::new()
        .with_title(APP_NAME)
        .with_inner_size(LogicalSize::new(w, h))
        .with_min_inner_size(LogicalSize::new(880.0, 600.0))
        .with_window_icon(Icon::from_rgba(ICON_RGBA.to_vec(), 64, 64).ok())
        .build(&event_loop)
        .expect("could not create the window");

    // Same page colour as the app, so there is no white flash while it loads.
    let background = match window.theme() {
        Theme::Dark => (9, 16, 20, 255),
        _ => (228, 237, 240, 255),
    };

    let state_json = serde_json::to_string(&store.borrow().map).unwrap_or_else(|_| "{}".into());
    let shim = SHIM_JS.replace("__STATE__", &state_json);

    let mut context = WebContext::new(Some(dir.join("webview")));
    let ipc_store = Rc::clone(&store);
    let webview = WebViewBuilder::new_with_web_context(&mut context)
        .with_background_color(background)
        .with_initialization_script(shim)
        .with_custom_protocol("app".into(), |_id, request| {
            if request.uri().path() == "/" || request.uri().path() == "/index.html" {
                Response::builder()
                    .header(CONTENT_TYPE, "text/html; charset=utf-8")
                    .body(INDEX_HTML.as_bytes().into())
                    .unwrap()
            } else {
                Response::builder().status(404).body(Vec::new().into()).unwrap()
            }
        })
        .with_navigation_handler(|url| {
            url.starts_with("app://") || url.starts_with("http://app.localhost") || url == "about:blank"
        })
        .with_new_window_req_handler(|_, _| wry::NewWindowResponse::Deny)
        .with_ipc_handler(move |request| {
            let Ok(msg) = serde_json::from_str::<Value>(request.body()) else { return };
            let text_of = |key: &str| msg.get(key).and_then(Value::as_str).map(str::to_owned);
            match msg.get("t").and_then(Value::as_str) {
                Some("set") => {
                    if let (Some(k), Some(v)) = (text_of("k"), text_of("v")) {
                        let mut s = ipc_store.borrow_mut();
                        s.map.insert(k, Value::String(v));
                        s.save();
                    }
                }
                Some("del") => {
                    if let Some(k) = text_of("k") {
                        let mut s = ipc_store.borrow_mut();
                        s.map.remove(&k);
                        s.save();
                    }
                }
                Some("clear") => {
                    let mut s = ipc_store.borrow_mut();
                    s.map.clear();
                    s.save();
                }
                Some("clip") => {
                    let id = msg.get("id").and_then(Value::as_u64).unwrap_or(0);
                    let html = text_of("html");
                    let text = text_of("text").unwrap_or_default();
                    let ok = write_clipboard(html.as_deref(), &text);
                    let _ = proxy.send_event(UserEvent::ClipResult(id, ok));
                }
                _ => {}
            }
        })
        .with_url(START_URL)
        .build(&window)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        // These stay alive for as long as the window does.
        let _keep = (&context, &store);
        #[cfg(target_os = "macos")]
        let _keep_menu = &menu;

        match event {
            Event::UserEvent(UserEvent::ClipResult(id, ok)) => {
                let _ = webview.evaluate_script(&format!("window.__clipDone({id},{ok})"));
            }
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                save_geometry(&dir, size.to_logical(window.scale_factor()));
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}
