use crate::artist_pic_fetcher;
use crate::db::{self, DbPool};
use crate::scanner;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_store::StoreExt;

const AUDIO_EXTENSIONS: [&str; 7] = ["mp3", "flac", "wav", "ogg", "m4a", "aac", "opus"];

pub struct SyncManager {
    watcher: Arc<parking_lot::Mutex<Option<RecommendedWatcher>>>,
    task: Arc<parking_lot::Mutex<Option<JoinHandle<()>>>>,
    scanning: Arc<AtomicBool>,
}

impl SyncManager {
    pub fn new() -> Self {
        Self {
            watcher: Arc::new(parking_lot::Mutex::new(None)),
            task: Arc::new(parking_lot::Mutex::new(None)),
            scanning: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_scanning(&self, active: bool) {
        self.scanning.store(active, Ordering::Relaxed);
    }

    pub fn init(&self, app: &AppHandle) {
        let app_handle = app.clone();

        // Startup Sync
        tauri::async_runtime::spawn(async move {
            let sync_on_startup = get_setting(&app_handle, "syncOnStartup", true).unwrap_or(true);
            if sync_on_startup {
                println!("Performing startup sync...");
                let _ = app_handle.emit(
                    "scan-progress",
                    crate::scanner::ScanProgress {
                        current: 0,
                        total: 100,
                        message: "Performing startup sync...".to_string(),
                    },
                );
                if let Some(sync_manager) = app_handle.try_state::<SyncManager>() {
                    sync_manager.set_scanning(true);
                }
                let pool = app_handle.state::<DbPool>();
                let pool = pool.inner().clone();
                let handle_for_scan = app_handle.clone();
                let _ = tokio::task::spawn_blocking(move || {
                    if let Ok(mut conn) = pool.get() {
                        let _ = scanner::scan_directories(&mut conn, &handle_for_scan);
                    }
                })
                .await;
                if let Some(sync_manager) = app_handle.try_state::<SyncManager>() {
                    sync_manager.set_scanning(false);
                }
            }

            // Retry failed artist image fetches from previous runs
            {
                let pool = app_handle.state::<DbPool>();
                if let Ok(conn) = pool.get() {
                    let fetch_pic =
                        get_setting(&app_handle, "autoFetchArtistPic", true).unwrap_or(true);
                    if fetch_pic {
                        if let Ok(artists) = db::get_artists_needing_fetch(&conn) {
                            if !artists.is_empty() {
                                let app_dir = app_handle
                                    .path()
                                    .app_data_dir()
                                    .map_err(|e| eprintln!("Failed to get app dir: {e}"))
                                    .ok();
                                if let Some(app_dir) = app_dir {
                                    let artists_map: HashMap<i64, String> =
                                        artists.into_iter().collect();
                                    let pool_clone = pool.inner().clone();
                                    let app_handle_clone = app_handle.clone();
                                    let app_dir_clone = app_dir.clone();
                                    println!(
                                        "Retrying artist image fetch for {} artists",
                                        artists_map.len()
                                    );
                                    tauri::async_runtime::spawn(async move {
                                        let _ = artist_pic_fetcher::fetch_artist_images(
                                            &artists_map,
                                            &app_dir_clone,
                                            pool_clone,
                                            &app_handle_clone,
                                        )
                                        .await;
                                    });
                                }
                            }
                        }
                    }
                }
            }

            // Real-time Watcher
            if let Ok(realtime_sync) = get_setting(&app_handle, "realtimeSync", true) {
                if realtime_sync {
                    let manager = app_handle.state::<SyncManager>();
                    let _ = manager.refresh_watcher(&app_handle);
                }
            }
        });
    }

    pub fn refresh_watcher(&self, app: &AppHandle) -> notify::Result<()> {
        // Cancel the previous watcher task
        {
            let mut task_lock = self.task.lock();
            if let Some(old_task) = task_lock.take() {
                old_task.abort();
            }
        }

        let mut watcher_lock = self.watcher.lock();

        if let Some(old_watcher) = watcher_lock.take() {
            drop(old_watcher);
        }

        if let Ok(realtime_sync) = get_setting(app, "realtimeSync", true) {
            if !realtime_sync {
                return Ok(());
            }
        } else {
            return Ok(());
        }

        let app_handle = app.clone();
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    let _ = tx.blocking_send(event);
                }
            },
            notify::Config::default(),
        )?;

        let pool = app_handle.state::<DbPool>();
        let conn = pool.get().expect("failed to get db connection");
        let source_dirs = db::get_source_dirs(&conn).expect("failed to get source dirs");

        for dir in source_dirs {
            let path = Path::new(&dir);
            if path.exists() {
                let _ = watcher.watch(path, RecursiveMode::Recursive);
            }
        }

        *watcher_lock = Some(watcher);

        let scanning = self.scanning.clone();
        let handle = tauri::async_runtime::spawn(async move {
            while let Some(event) = rx.recv().await {
                if scanning.load(Ordering::Relaxed)
                    && matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_))
                {
                    continue;
                }

                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) => {
                        let paths_to_scan: Vec<PathBuf> = event
                            .paths
                            .into_iter()
                            .filter(|p| {
                                let ext = p
                                    .extension()
                                    .and_then(|e| e.to_str())
                                    .unwrap_or("")
                                    .to_lowercase();
                                p.is_file() && AUDIO_EXTENSIONS.contains(&ext.as_ref())
                            })
                            .collect();

                        if !paths_to_scan.is_empty() {
                            let pool = app_handle.state::<DbPool>();
                            let pool = pool.inner().clone();
                            let handle_for_scan = app_handle.clone();
                            let _ = tokio::task::spawn_blocking(move || {
                                if let Ok(mut conn) = pool.get() {
                                    let _ = scanner::scan_files(
                                        &mut conn,
                                        &handle_for_scan,
                                        paths_to_scan,
                                    );
                                }
                            })
                            .await;
                        }
                    }
                    EventKind::Remove(_) => {
                        let paths_to_remove: Vec<String> = event
                            .paths
                            .into_iter()
                            .map(|p| p.to_string_lossy().to_string())
                            .collect();

                        if !paths_to_remove.is_empty() {
                            let pool = app_handle.state::<DbPool>();
                            if let Ok(mut conn) = pool.get() {
                                let _ = (|| -> Result<(), crate::error::Error> {
                                    let mut tracks_to_delete = Vec::new();
                                    for path in &paths_to_remove {
                                        let is_audio_file = Path::new(path)
                                            .extension()
                                            .and_then(|e| e.to_str())
                                            .map(|e| AUDIO_EXTENSIONS.contains(&e))
                                            .unwrap_or(false);

                                        if is_audio_file {
                                            let mut stmt = conn
                                                .prepare("SELECT path FROM track WHERE path = ?")
                                                .map_err(crate::error::Error::Db)?;
                                            let rows = stmt
                                                .query_map(rusqlite::params![path], |row| {
                                                    row.get::<_, String>(0)
                                                })
                                                .map_err(crate::error::Error::Db)?;
                                            for r in rows {
                                                if let Ok(p) = r {
                                                    tracks_to_delete.push(p);
                                                }
                                            }
                                        } else {
                                            let mut stmt = conn.prepare(
                                                "SELECT path FROM track WHERE path = ? OR path LIKE ? || '/%' OR path LIKE ? || '\\%'"
                                            ).map_err(crate::error::Error::Db)?;
                                            let rows = stmt
                                                .query_map(
                                                    rusqlite::params![path, path, path],
                                                    |row| row.get::<_, String>(0),
                                                )
                                                .map_err(crate::error::Error::Db)?;
                                            for r in rows {
                                                if let Ok(p) = r {
                                                    tracks_to_delete.push(p);
                                                }
                                            }
                                        }
                                    }

                                    if !tracks_to_delete.is_empty() {
                                        let tx =
                                            conn.transaction().map_err(crate::error::Error::Db)?;
                                        db::delete_tracks_by_paths(&tx, &tracks_to_delete)?;
                                        tx.commit().map_err(crate::error::Error::Db)?;
                                        let _ = app_handle.emit("library-updated", ());
                                    }
                                    Ok(())
                                })();
                            }
                        }
                    }

                    _ => {}
                }
            }
        });

        self.task.lock().replace(handle);

        Ok(())
    }
}

pub fn get_setting<R: tauri::Runtime>(
    app: &AppHandle<R>,
    key: &str,
    default: bool,
) -> Result<bool, Box<dyn std::error::Error>> {
    let stores = app.app_handle().store("settings.json")?;

    if let Some(value) = stores.get(key) {
        Ok(value.as_bool().unwrap_or(default))
    } else {
        Ok(default)
    }
}
