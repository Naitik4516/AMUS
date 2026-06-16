pub mod artist_pic_fetcher;
pub mod commands;
pub mod db;
pub mod engine;
pub mod error;
pub mod models;
pub mod scanner;

use engine::{AudioEngine, Player};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rodio::DeviceSinkBuilder;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle();
            let app_dir = app_handle
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");
            let db_path = app_dir.join("music.db");

            let manager = SqliteConnectionManager::file(db_path);
            let pool = Pool::new(manager).expect("failed to create db pool");

            {
                let conn = pool.get().expect("failed to get db connection");
                db::init_db(&conn).expect("failed to initialize database");
            }

            let stream = DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
            let engine = AudioEngine::new(&stream.mixer());
            let engine_arc = std::sync::Arc::new(parking_lot::Mutex::new(engine));

            app.manage(pool);
            app.manage(stream);
            app.manage(Player {
                engine: engine_arc.clone(),
            });

            engine::engine::spawn_playback_monitor(app_handle.clone(), engine_arc);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::add_source,
            commands::get_source_dirs,
            commands::remove_source,
            commands::scan_library,
            commands::get_all_tracks,
            commands::get_favorite_tracks,
            commands::get_recently_played,
            commands::get_most_played_tracks,
            commands::get_track_details,
            commands::search_tracks,
            commands::get_artists,
            commands::get_artist,
            commands::get_all_albums,
            commands::get_album,
            commands::get_albums,
            commands::get_tracks_by_album,
            commands::get_tracks_by_artist,
            commands::get_playlists,
            commands::get_tracks_by_playlist,
            commands::create_playlist,
            commands::add_track_to_playlist,
            commands::remove_track_from_playlist,
            commands::delete_playlist,
            commands::rename_playlist,
            commands::toggle_favorite,
            commands::get_similar_songs,
            commands::get_playlist_cover_arts,
            commands::play_track,
            commands::toggle_playback,
            commands::set_volume,
            commands::seek,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
