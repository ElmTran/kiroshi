//! The search box window, the tray icon and the global hotkey.

use std::fs;
use std::path::Path;

use serde::Deserialize;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Emitter, Manager, WebviewWindow};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

const DEFAULT_HOTKEY: &str = "Alt+H";
const FALLBACK_HOTKEY: &str = "Ctrl+Alt+K";

pub fn show(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        // Wake first: the "shown" event below needs a running page.
        set_awake(&w, true);
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
        set_awake(&w, false);
    }
}

/// While the window is hidden, the page is suspended (scripts and timers
/// paused) and WebView2 is asked to keep its memory low; showing undoes both.
#[cfg(windows)]
pub fn set_awake(window: &WebviewWindow, awake: bool) {
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
        ICoreWebView2_3, ICoreWebView2_19,
    };
    use webview2_com::TrySuspendCompletedHandler;
    use windows_core::Interface;

    let result = window.with_webview(move |webview| unsafe {
        let controller = webview.controller();
        let Ok(core) = controller.CoreWebView2() else {
            return;
        };
        let level = if awake {
            COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL
        } else {
            COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW
        };
        // Visibility goes first: a visible WebView cannot be suspended, and
        // becoming visible resumes it.
        let _ = controller.SetIsVisible(awake);
        if let Ok(core19) = core.cast::<ICoreWebView2_19>() {
            let _ = core19.SetMemoryUsageTargetLevel(level);
        }
        let Ok(core3) = core.cast::<ICoreWebView2_3>() else {
            return;
        };
        if awake {
            let _ = core3.Resume();
        } else {
            let handler = TrySuspendCompletedHandler::create(Box::new(|result, suspended| {
                match result {
                    Ok(()) => log::debug!("webview suspended: {suspended}"),
                    Err(e) => log::warn!("could not suspend the webview: {e}"),
                }
                Ok(())
            }));
            if let Err(e) = core3.TrySuspend(&handler) {
                log::warn!("could not suspend the webview: {e}");
            }
        }
    });
    if let Err(e) = result {
        log::warn!("could not reach the webview: {e}");
    }
}

#[cfg(not(windows))]
pub fn set_awake(_window: &WebviewWindow, _awake: bool) {}

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
    let autostart_on = app.autolaunch().is_enabled().unwrap_or(false);
    let autostart_item = CheckMenuItem::with_id(
        app,
        "autostart",
        "开机启动",
        true,
        autostart_on,
        None::<&str>,
    )?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &show_item,
            &folder_item,
            &PredefinedMenuItem::separator(app)?,
            &autostart_item,
            &PredefinedMenuItem::separator(app)?,
            &quit_item,
        ],
    )?;

    let mut tray = TrayIconBuilder::new()
        .tooltip("Kiroshi")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "show" => show(app),
            "folder" => {
                if let Err(e) = open_folder(app.state::<crate::AppState>().store.path()) {
                    log::warn!("{e}");
                }
            }
            "autostart" => toggle_autostart(app, &autostart_item),
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

/// Flips "start with Windows". The menu toggles its own check mark on click,
/// so it is reset afterwards to what the registry actually says.
fn toggle_autostart(app: &AppHandle, item: &CheckMenuItem<tauri::Wry>) {
    let autolaunch = app.autolaunch();
    let enabled = autolaunch.is_enabled().unwrap_or(false);
    let result = if enabled {
        autolaunch.disable()
    } else {
        autolaunch.enable()
    };
    match result {
        Ok(()) => log::info!("start with Windows: {}", !enabled),
        Err(e) => log::warn!("could not change start with Windows: {e}"),
    }
    let _ = item.set_checked(autolaunch.is_enabled().unwrap_or(enabled));
}

/// Opens Explorer with the data file selected.
pub fn open_folder(file: &Path) -> Result<(), String> {
    std::process::Command::new("explorer")
        .arg(format!("/select,{}", file.display()))
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("无法打开文件夹：{e}"))
}
