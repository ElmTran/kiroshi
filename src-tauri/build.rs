// Every command the frontend may call is listed here. tauri-build turns each
// into an `allow-<name>` permission, and capabilities/main.json grants them.
const COMMANDS: &[&str] = &[
    "app_info",
    "list_items",
    "save_item",
    "delete_item",
    "hide_window",
    "open_data_folder",
];

fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    )
    .expect("tauri-build failed");
}
