mod commands;
mod store;
mod window;

use tauri::{Manager, WindowEvent};
use tauri_plugin_global_shortcut::ShortcutState;
use tauri_plugin_log::{Target, TargetKind};

pub struct AppState {
    pub store: store::Store,
    pub hotkey: Option<String>,
}

pub fn run() {
    // Set KIROSHI_KEEP_OPEN to stop the window hiding when it loses focus
    // (useful with devtools open).
    let hide_on_blur = std::env::var_os("KIROSHI_KEEP_OPEN").is_none();

    tauri::Builder::default()
        // Must be registered first: a second launch hands over to the running
        // instance, which shows its window, and exits.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            window::show(app)
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(if cfg!(debug_assertions) {
                    log::LevelFilter::Debug
                } else {
                    log::LevelFilter::Info
                })
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                ])
                .build(),
        )
        // Start with Windows (toggled from the tray menu): a Run registry entry
        // that launches straight into the tray.
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .arg("--hidden")
                .build(),
        )
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        window::toggle(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let hotkey = window::register_hotkey(app.handle(), &data_dir);
            app.manage(AppState {
                store: store::Store::new(&data_dir),
                hotkey,
            });
            window::build_tray(app)?;
            log::info!("Kiroshi {} started", env!("CARGO_PKG_VERSION"));
            // `--hidden` is for starting with Windows: tray only.
            if !std::env::args().any(|a| a == "--hidden") {
                window::show(app.handle());
            }
            Ok(())
        })
        .on_window_event(move |window, event| match event {
            WindowEvent::Focused(false) if hide_on_blur => {
                let _ = window.hide();
            }
            // Alt+F4 hides; quitting is in the tray menu.
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window.hide();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::list_items,
            commands::save_item,
            commands::delete_item,
            commands::hide_window,
            commands::open_data_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Kiroshi");
}
