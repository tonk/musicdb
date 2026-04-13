mod commands;
mod db;
mod error;
mod models;
mod state;

use std::sync::Mutex;

use state::AppState;
use tauri::Manager;

fn resolve_db_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&data_dir)?;
    Ok(data_dir.join("musicdb.sqlite"))
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
            let db_path = resolve_db_path(app.handle())?;
            let db_path_str = db_path
                .to_str()
                .ok_or("invalid DB path")?
                .to_owned();

            let pool = tauri::async_runtime::block_on(db::init_pool(&db_path_str))?;

            app.manage(AppState {
                db: pool,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
