use serde::Serialize;
use tauri::{AppHandle, State};

use crate::AppState;
use crate::store::Item;
use crate::window;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    version: &'static str,
    /// The global hotkey that is actually registered, if any.
    hotkey: Option<String>,
    data_file: String,
}

#[tauri::command]
pub fn app_info(state: State<AppState>) -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION"),
        hotkey: state.hotkey.clone(),
        data_file: state.store.path().display().to_string(),
    }
}

#[tauri::command]
pub fn list_items(state: State<AppState>) -> Result<Vec<Item>, String> {
    state.store.list()
}

#[tauri::command]
pub fn save_item(state: State<AppState>, item: Item) -> Result<Item, String> {
    state.store.save(item)
}

#[tauri::command]
pub fn delete_item(state: State<AppState>, id: String) -> Result<(), String> {
    state.store.delete(&id)
}

#[tauri::command]
pub fn hide_window(app: AppHandle) {
    window::hide(&app);
}

#[tauri::command]
pub fn open_data_folder(state: State<AppState>) -> Result<(), String> {
    window::open_folder(state.store.path())
}
