use crate::db::{self, DbPool};
use crate::scanner;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_store::StoreExt;

pub struct SyncManager {
    watcher: Arc<parking_lot::Mutex<Option<RecommendedWatcher>>>,
}

impl SyncManager {
    pub fn new() -> Self {
        Self {
            watcher: Arc::new(parking_lot::Mutex::new(None)),
        }
    }

    pub fn init(&self, app: &AppHandle) {
        let app_handle = app.clone();
        
        // 1. Startup Sync
        tauri::async_runtime::spawn(async move {
            if let Ok(sync_on_startup) = get_setting(&app_handle, "syncOnStartup", true) {
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
                    let pool = app_handle.state::<DbPool>();
                    if let Ok(mut conn) = pool.get() {
                        let _ = scanner::scan_directories(&mut conn, &app_handle);
                    }
                }
            }
            
            // 2. Real-time Watcher
            if let Ok(realtime_sync) = get_setting(&app_handle, "realtimeSync", true) {
                if realtime_sync {
                    let manager = app_handle.state::<SyncManager>();
                    let _ = manager.refresh_watcher(&app_handle);
                }
            }
        });
    }

    pub fn refresh_watcher(&self, app: &AppHandle) -> notify::Result<()> {
        let mut watcher_lock = self.watcher.lock();
        
        // Stop existing watcher if any
        if let Some(old_watcher) = watcher_lock.take() {
            // Watchers stop when dropped, but we can also explicitly unwatch if we want
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

        // Store the watcher
        *watcher_lock = Some(watcher);

        tauri::async_runtime::spawn(async move {
            let audio_extensions = ["mp3", "flac", "wav", "ogg", "m4a", "aac", "opus"];

            while let Some(event) = rx.recv().await {
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) => {
                        let paths_to_scan: Vec<PathBuf> = event.paths.into_iter()
                            .filter(|p| {
                                let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                                p.is_file() && audio_extensions.contains(&ext.as_ref())
                            })
                            .collect();
                        
                        if !paths_to_scan.is_empty() {
                            let pool = app_handle.state::<DbPool>();
                            if let Ok(mut conn) = pool.get() {
                                let _ = scanner::scan_files(&mut conn, &app_handle, paths_to_scan);
                            }
                        }
                    }
                    EventKind::Remove(_) => {
                        let paths_to_remove: Vec<String> = event.paths.into_iter()
                            .map(|p| p.to_string_lossy().to_string())
                            .collect();

                        if !paths_to_remove.is_empty() {
                            let pool = app_handle.state::<DbPool>();
                            if let Ok(mut conn) = pool.get() {
                                let _ = (|| -> Result<(), crate::error::Error> {
                                    let mut tracks_to_delete = Vec::new();
                                    for path in &paths_to_remove {
                                        let mut stmt = conn.prepare(
                                            "SELECT path FROM track WHERE path = ? OR path LIKE ? || '/%' OR path LIKE ? || '\\%'"
                                        ).map_err(crate::error::Error::Db)?;
                                        let rows = stmt.query_map(rusqlite::params![path, path, path], |row| row.get::<_, String>(0))
                                            .map_err(crate::error::Error::Db)?;
                                        for r in rows {
                                            if let Ok(p) = r {
                                                tracks_to_delete.push(p);
                                            }
                                        }
                                    }

                                    if !tracks_to_delete.is_empty() {
                                        let tx = conn.transaction().map_err(crate::error::Error::Db)?;
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
