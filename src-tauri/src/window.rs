//! The search box window, the tray icon and the global hotkey.

use std::fs;
use std::path::Path;

use serde::Deserialize;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

const DEFAULT_HOTKEY: &str = "Alt+Space";
const FALLBACK_HOTKEY: &str = "Ctrl+Alt+K";

pub fn show(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.center();
        let _ = w.show();
        let _ = w.set_focus();
        // The frontend reloads the list and selects the search text.
        let _ = w.emit("shown", ());
    }
}

pub fn hide(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
}

pub fn toggle(app: &AppHandle) {
    let active = app
        .get_webview_window("main")
        .is_some_and(|w| w.is_visible().unwrap_or(false) && w.is_focused().unwrap_or(false));
    if active { hide(app) } else { show(app) }
}

#[derive(Deserialize, Default)]
struct Settings {
    hotkey: Option<String>,
}

/// Registers the hotkey from `settings.json` (created with the default on
/// first run). If another program already owns it, falls back once. Returns
/// the hotkey that is active, if any.
pub fn register_hotkey(app: &AppHandle, data_dir: &Path) -> Option<String> {
    let file = data_dir.join("settings.json");
    if !file.exists() {
        let _ = fs::create_dir_all(data_dir);
        let _ = fs::write(
            &file,
            format!("{{\n  \"hotkey\": \"{DEFAULT_HOTKEY}\"\n}}\n"),
        );
    }
    let wanted = fs::read_to_string(&file)
        .ok()
        .and_then(|text| serde_json::from_str::<Settings>(&text).ok())
        .and_then(|s| s.hotkey)
        .filter(|h| !h.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_HOTKEY.to_string());

    for hotkey in [wanted.as_str(), FALLBACK_HOTKEY] {
        match app.global_shortcut().register(hotkey) {
            Ok(()) => {
                log::info!("global hotkey: {hotkey}");
                return Some(hotkey.to_string());
            }
            Err(e) => log::warn!("could not register hotkey {hotkey}: {e}"),
        }
    }
    None
}

pub fn build_tray(app: &App) -> tauri::Result<()> {
    let show_item = MenuItem::with_id(app, "show", "显示 Kiroshi", true, None::<&str>)?;
    let folder_item = MenuItem::with_id(app, "folder", "打开数据文件夹", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &folder_item, &quit_item])?;

    let mut tray = TrayIconBuilder::new()
        .tooltip("Kiroshi")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show(app),
            "folder" => {
                if let Err(e) = open_folder(app.state::<crate::AppState>().store.path()) {
                    log::warn!("{e}");
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

/// Opens Explorer with the data file selected.
pub fn open_folder(file: &Path) -> Result<(), String> {
    std::process::Command::new("explorer")
        .arg(format!("/select,{}", file.display()))
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("无法打开文件夹：{e}"))
}
