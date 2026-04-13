mod commands;
mod db;
mod error;
mod models;
mod state;

use std::sync::Mutex;
use tokio::sync::RwLock;

use state::{AppState, DatabaseConfig, DatabaseEntry};
use tauri::Manager;

fn load_db_config(
    app: &tauri::AppHandle,
) -> Result<(DatabaseConfig, String), Box<dyn std::error::Error>> {
    let data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&data_dir)?;

    let cfg_path = data_dir.join("databases.json");

    let config: DatabaseConfig = if cfg_path.exists() {
        let content = std::fs::read_to_string(&cfg_path)?;
        serde_json::from_str(&content)?
    } else {
        // First run: create a "Default" entry pointing at the existing musicdb.sqlite
        let default_path = data_dir
            .join("musicdb.sqlite")
            .to_string_lossy()
            .to_string();
        DatabaseConfig {
            current: "Default".to_string(),
            databases: vec![DatabaseEntry {
                name: "Default".to_string(),
                path: default_path,
            }],
        }
    };

    // Resolve the active path (fall back to first entry if current name not found)
    let current_path = config
        .databases
        .iter()
        .find(|db| db.name == config.current)
        .or_else(|| config.databases.first())
        .map(|db| db.path.clone())
        .ok_or("No databases configured")?;

    // Persist config so it always exists on disk
    let json = serde_json::to_string_pretty(&config)?;
    std::fs::write(&cfg_path, json)?;

    Ok((config, current_path))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let (db_config, db_path) = load_db_config(app.handle())?;
            let pool = tauri::async_runtime::block_on(db::init_pool(&db_path))?;

            app.manage(AppState {
                db: RwLock::new(pool),
                db_config: Mutex::new(db_config),
                undo_buffer: Mutex::new(None),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::items::get_item,
            commands::items::create_item,
            commands::items::update_item,
            commands::items::delete_item,
            commands::items::undo_delete,
            commands::items::list_items,
            commands::items::search_items,
            commands::items::get_statistics,
            commands::artists::list_artists,
            commands::artists::autocomplete_artists,
            commands::artists::create_artist,
            commands::artists::get_artist_items,
            commands::genres::list_genres,
            commands::genres::create_genre,
            commands::import::import_txt_file,
            commands::import::preview_csv,
            commands::import::import_csv,
            commands::import::import_audio_folder,
            commands::export::export_csv,
            commands::export::export_json,
            commands::cover_art::save_cover_art,
            commands::cover_art::fetch_caa_cover,
            commands::musicbrainz::lookup_release,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::get_all_settings,
            commands::settings::move_database,
            commands::settings::reset_database,
            commands::settings::list_databases,
            commands::settings::current_database,
            commands::settings::create_database,
            commands::settings::switch_database,
            commands::settings::rename_database,
            commands::settings::delete_database,
            commands::settings::backup_database,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
