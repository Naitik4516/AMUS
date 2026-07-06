pub mod artist_pic_fetcher;
pub mod commands;
pub mod db;
pub mod error;
pub mod models;
pub mod player;
pub mod scanner;
pub mod sync;

use crate::player::actor::PlayerCommand;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::sync::atomic::{AtomicBool, Ordering};
use sync::SyncManager;
use tauri::{
    Emitter, Manager, WebviewUrl, WebviewWindowBuilder,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_positioner::{Position, WindowExt};

pub(crate) struct MiniPlayerPinned(AtomicBool);

fn build_tray_menu(app: &tauri::AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let play_pause = MenuItem::with_id(app, "play_pause", "Play/Pause", true, None::<&str>)?;
    let previous = MenuItem::with_id(app, "previous", "Previous", true, None::<&str>)?;
    let next = MenuItem::with_id(app, "next", "Next", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let show_miniplayer = MenuItem::with_id(
        app,
        "show_miniplayer",
        "Show Miniplayer",
        true,
        None::<&str>,
    )?;
    let show = MenuItem::with_id(app, "show", "Show/Hide", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    Menu::with_items(
        app,
        &[
            &play_pause,
            &previous,
            &next,
            &separator,
            &show_miniplayer,
            &show,
            &separator,
            &quit,
        ],
    )
}

fn handle_tray_menu(app: &tauri::AppHandle, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        "play_pause" => {
            // let player = app.state::<Player>();
            // let engine = player.engine.lock();
            // if engine.is_paused() {
            //     engine.resume();
            // } else {
            //     engine.pause();
            // }
        }
        "next" => {
            // let player = app.state::<Player>();
            // let mut engine = player.engine.lock();
            // let _ = engine.upcoming_context();
            // let state = engine.get_playback_state();
            // let _ = app.emit("track-changed", &state);
        }
        "previous" => {
            // let player = app.state::<Player>();
            // let mut engine = player.engine.lock();
            // let _ = engine.play_previous();
            // let state = engine.get_playback_state();
            // let _ = app.emit("track-changed", &state);
        }
        "show_miniplayer" => {
            toggle_popup(app);
        }
        "show" => {
            if let Some(window) = app.get_webview_window("main") {
                if window.is_visible().unwrap_or(false) {
                    let _ = window.hide();
                } else {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}

fn toggle_popup(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("mini-player") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    } else if let Ok(window) = create_popup(app) {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn create_popup(app: &tauri::AppHandle) -> tauri::Result<tauri::WebviewWindow> {
    let window =
        WebviewWindowBuilder::new(app, "mini-player", WebviewUrl::App("/miniplayer".into()))
            .title("Amus - Mini Player")
            .inner_size(400.0, 200.0)
            .resizable(false)
            .decorations(false)
            .transparent(true)
            .shadow(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .build()?;

    let app_clone = app.clone();
    window.on_window_event(move |event| match event {
        tauri::WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            if let Some(w) = app_clone.get_webview_window("mini-player") {
                let _ = w.hide();
            }
        }
        tauri::WindowEvent::Focused(false) => {
            if let Some(state) = app_clone.try_state::<MiniPlayerPinned>() {
                if !state.0.load(Ordering::Relaxed) {
                    if let Some(w) = app_clone.get_webview_window("mini-player") {
                        let _ = w.hide();
                        let _ = w.as_ref().window().move_window(Position::BottomRight);
                    }
                }
            }
        }
        _ => {}
    });

    Ok(window)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        let action_id = match shortcut.to_string().as_str() {
                            "MediaPlayPause" => "global_play_pause",
                            "MediaTrackNext" => "global_next_track",
                            "MediaTrackPrevious" => "global_prev_track",
                            "MediaStop" => "global_stop",
                            _ => return,
                        };
                        let _ = app.emit("global-shortcut", action_id);
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let app_handle = app.handle();

            let app_dir = app_handle
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");
            let db_path = app_dir.join("music.db");

            let manager = SqliteConnectionManager::file(db_path).with_init(|c| {
                c.execute_batch(
                    "PRAGMA foreign_keys = ON;\n\
                    PRAGMA journal_mode = WAL;\n\
                    PRAGMA synchronous = NORMAL;\n\
                    PRAGMA temp_store = MEMORY;\n\
                    PRAGMA busy_timeout = 5000;",
                )
            });
            let pool = Pool::new(manager).expect("failed to create db pool");

            {
                let mut conn = pool.get().expect("failed to get db connection");
                db::init_db(&mut conn).expect("failed to initialize database");
            }

            let handle =
                crate::player::actor::PlayerActor::spawn(app.handle().clone(), pool.clone());
            app.manage(commands::PlayerHandle(handle));

            let sync_manager = SyncManager::new();
            sync_manager.init(app_handle);
            app.manage(sync_manager);

            app.manage(pool);
            app.manage(MiniPlayerPinned(AtomicBool::new(true)));

            // System Tray
            let tray_menu = build_tray_menu(app_handle)?;

            TrayIconBuilder::new()
                .icon(app_handle.default_window_icon().cloned().unwrap())
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_popup(tray.app_handle());
                    }
                })
                .on_menu_event(handle_tray_menu)
                .build(app_handle)?;

            // Prevent closing the main window if "keepRunningInBg" is true
            let handle = app_handle.clone();
            if let Some(main_win) = app_handle.get_webview_window("main") {
                main_win.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        let keep_in_bg =
                            sync::get_setting(&handle, "keepRunningInBg", true).unwrap_or(true);
                        if keep_in_bg {
                            api.prevent_close();
                            if let Some(w) = handle.get_webview_window("main") {
                                let _ = w.hide();
                            }
                        }
                    }
                });
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                // best-effort flush; the periodic 30s checkpoint is the real safety net
                let _ = window
                    .state::<commands::PlayerHandle>()
                    .0
                    .send(PlayerCommand::Shutdown);
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::add_source,
            commands::get_source_dirs,
            commands::remove_source,
            commands::refresh_watcher,
            commands::has_music,
            commands::scan_library,
            commands::get_all_tracks,
            commands::get_favorite_tracks,
            commands::get_recently_played,
            commands::get_most_played_tracks,
            commands::get_track_details,
            commands::global_search,
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
            commands::get_playlist,
            commands::toggle_favorite,
            commands::play_context,
            commands::play_pause,
            commands::next,
            commands::previous,
            commands::seek,
            commands::set_volume,
            commands::set_repeat,
            commands::toggle_shuffle,
            commands::enqueue_next,
            commands::enqueue_end,
            commands::enqueue_end_many,
            commands::remove_from_queue,
            commands::clear_queue,
            commands::reorder_queue,
            commands::set_autoplay,
            commands::get_current_state,
            commands::get_top_artists,
            commands::get_top_albums,
            commands::get_forgotten_tracks,
            commands::get_unplayed_tracks,
            commands::get_recently_added,
            commands::fetch_artist_images,
            commands::save_image,
            commands::update_artist,
            commands::update_album,
            commands::update_playlist,
            commands::get_stats_overview,
            commands::get_top_tracks_with_stats,
            commands::get_top_artists_with_stats,
            commands::get_top_albums_with_stats,
            commands::get_listening_time_trend,
            commands::get_streak_data,
            commands::get_library_growth,
            commands::get_format_distribution,
            commands::get_data_age,
            commands::get_heatmap_hourly,
            commands::get_heatmap_weekday,
            commands::get_favorite_trends,
            commands::get_playback_history_timeline,
            commands::toggle_mini_player_pin,
            commands::quit_app,
            commands::toggle_mini_player,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
