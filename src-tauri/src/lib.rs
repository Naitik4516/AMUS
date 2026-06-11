pub mod commands;
pub mod db;
pub mod engine;
pub mod models;
pub mod scanner;

use engine::{AudioEngine, PlaybackState, Player};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rodio::DeviceSinkBuilder;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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

            app.manage(pool)
            app.manage(Player {
                playback_state: parking_lot::Mutex::new(PlaybackState {
                    is_playing: false,
                    position_ms: 0,
                    volume: 1.0,
                    queue_len: 0,
                }),
                engine: parking_lot::Mutex::new(engine)
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::add_source,
            commands::scan_library,
            commands::get_tracks,
            commands::toggle_favorite,
            commands::play_track,
            commands::toggle_playback,
            commands::set_volume,
            commands::seek,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
