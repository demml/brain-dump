use std::sync::Mutex;

use brain_dump_core::db;

mod commands;
use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = db::default_db_path().expect("cannot determine database path");
    let conn = db::open_db(&db_path).expect("failed to open database");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            db: Mutex::new(conn),
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_node,
            commands::update_node,
            commands::delete_node,
            commands::get_node,
            commands::list_nodes,
            commands::create_edge,
            commands::delete_edge,
            commands::add_tag,
            commands::remove_tag,
            commands::list_tags,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri application");
}
