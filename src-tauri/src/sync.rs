use crate::artist_pic_fetcher;
use crate::db::{self, DbPool};
use crate::scanner;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_store::StoreExt;

const AUDIO_EXTENSIONS: [&str; 7] = ["mp3", "flac", "wav", "ogg", "m4a", "aac", "opus"];

/// Debounce window for coalescing filesystem events before scanning them.
const WATCHER_DEBOUNCE: Duration = Duration::from_millis(500);

#[derive(Clone)]
pub struct SyncManager {
    watcher: Arc<parking_lot::Mutex<Option<RecommendedWatcher>>>,
    task: Arc<parking_lot::Mutex<Option<JoinHandle<()>>>>,
    scanning: Arc<AtomicBool>,
    scan_lock: Arc<parking_lot::Mutex<()>>,
}

/// Holds the scan lock and keeps the realtime watcher paused until dropped.
pub struct ScanGuard<'a> {
    manager: &'a SyncManager,
    _lock: parking_lot::MutexGuard<'a, ()>,
}

impl Drop for ScanGuard<'_> {
    fn drop(&mut self) {
        self.manager.set_scanning(false);
    }
}

impl SyncManager {
    pub fn new() -> Self {
        Self {
            watcher: Arc::new(parking_lot::Mutex::new(None)),
            task: Arc::new(parking_lot::Mutex::new(None)),
            scanning: Arc::new(AtomicBool::new(false)),
            scan_lock: Arc::new(parking_lot::Mutex::new(())),
        }
    }

    pub fn set_scanning(&self, active: bool) {
        self.scanning.store(active, Ordering::Release);
    }

    /// Serialize full-library scans (startup, manual, CLI) and pause the realtime watcher for the duration. The scanning flag is cleared on every exit path — success, error or panic — when the guard drops.
    pub fn try_start_scan(&self) -> Result<ScanGuard<'_>, String> {
        let lock = self
            .scan_lock
            .try_lock()
            .ok_or_else(|| "scan already in progress".to_string())?;
        self.set_scanning(true);
        Ok(ScanGuard {
            manager: self,
            _lock: lock,
        })
    }

    pub fn init(&self, app: &AppHandle) {
        let app_handle = app.clone();

        // Startup Sync
        tauri::async_runtime::spawn(async move {
            let sync_on_startup = get_setting(&app_handle, "syncOnStartup", true).unwrap_or(true);
            if sync_on_startup {
                let _ = app_handle.emit(
                    "scan-progress",
                    crate::scanner::ScanProgress {
                        current: 0,
                        total: 100,
                        message: "Performing startup sync...".to_string(),
                    },
                );
                if let Some(sync_manager) = app_handle.try_state::<SyncManager>() {
                    let sync_manager = sync_manager.inner().clone();
                    let pool = app_handle.state::<DbPool>();
                    let pool = pool.inner().clone();
                    let handle_for_scan = app_handle.clone();
                    let _ = tokio::task::spawn_blocking(move || {
                        let _guard = match sync_manager.try_start_scan() {
                            Ok(guard) => guard,
                            Err(_) => return,
                        };
                        if let Ok(mut conn) = pool.get() {
                            let _ = scanner::scan_directories(&mut conn, &handle_for_scan);
                        }
                    })
                    .await;
                }
            }

            // Retry failed artist image fetches from previous runs
            {
                let pool = app_handle.state::<DbPool>();
                let pool = pool.inner().clone();
                let handle_for_fetch = app_handle.clone();
                let _ = tokio::task::spawn_blocking(move || {
                    if let Ok(conn) = pool.get() {
                        let fetch_pic =
                            get_setting(&handle_for_fetch, "autoFetchArtistPic", true).unwrap_or(true);
                        if fetch_pic {
                            if let Ok(artists) = db::get_artists_needing_fetch(&conn) {
                                if !artists.is_empty() {
                                    let app_dir = handle_for_fetch
                                        .path()
                                        .app_data_dir()
                                        .map_err(|e| {
                                            tracing::warn!(error = %e, "failed to get app dir");
                                            e
                                        })
                                        .ok();
                                    if let Some(app_dir) = app_dir {
                                        let artists_map: HashMap<i64, String> =
                                            artists.into_iter().collect();
                                        let pool_clone = pool.clone();
                                        let app_handle_clone = handle_for_fetch.clone();
                                        let app_dir_clone = app_dir.clone();
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
                })
                .await;
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

    pub fn refresh_watcher(&self, app: &AppHandle) -> Result<(), String> {
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
        )
        .map_err(|e| format!("failed to create watcher: {e}"))?;

        let pool = app_handle.state::<DbPool>();
        let conn = pool.get().map_err(|e| format!("failed to get db connection: {e}"))?;
        let source_dirs = db::get_source_dirs(&conn).map_err(|e| format!("failed to get source dirs: {e}"))?;

        for dir in source_dirs {
            let path = Path::new(&dir);
            if path.exists() {
                let _ = watcher.watch(path, RecursiveMode::Recursive);
            }
        }

        *watcher_lock = Some(watcher);

        let scanning = self.scanning.clone();
        let handle = tauri::async_runtime::spawn(async move {
            // Coalesce Create/Modify events and scan them in one batched pass after a short idle window, instead of spawning a scan per event.
            let mut pending_paths: Vec<PathBuf> = Vec::new();

            loop {
                let event = match tokio::time::timeout(WATCHER_DEBOUNCE, rx.recv()).await {
                    Ok(Some(event)) => event,
                    Ok(None) => {
                        flush_pending_scan(&app_handle, &mut pending_paths).await;
                        break;
                    }
                    Err(_) => {
                        flush_pending_scan(&app_handle, &mut pending_paths).await;
                        continue;
                    }
                };

                if scanning.load(Ordering::Acquire)
                    && matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_))
                {
                    continue;
                }

                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) => {
                        pending_paths.extend(event.paths.into_iter().filter(|p| {
                            let ext = p
                                .extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("")
                                .to_lowercase();
                            p.is_file() && AUDIO_EXTENSIONS.contains(&ext.as_ref())
                        }));
                    }
                    EventKind::Remove(_) => {
                        flush_pending_scan(&app_handle, &mut pending_paths).await;

                        let paths_to_remove: Vec<String> = event
                            .paths
                            .into_iter()
                            .map(|p| p.to_string_lossy().to_string())
                            .collect();

                        if !paths_to_remove.is_empty() {
                            let pool = app_handle.state::<DbPool>();
                            let pool = pool.inner().clone();
                            let handle_for_emit = app_handle.clone();
                            let _ = tokio::task::spawn_blocking(move || {
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
                                            let tx = conn
                                                .transaction()
                                                .map_err(crate::error::Error::Db)?;
                                            db::delete_tracks_by_paths(&tx, &tracks_to_delete)?;
                                            tx.commit().map_err(crate::error::Error::Db)?;
                                            let _ = handle_for_emit.emit("library-updated", ());
                                        }
                                        Ok(())
                                    })();
                                }
                            }).await;
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

// Scan a batch of pending audio files (with blacklist filtering)
async fn flush_pending_scan(app_handle: &AppHandle, pending: &mut Vec<PathBuf>) {
    if pending.is_empty() {
        return;
    }
    let paths_to_scan = std::mem::take(pending);
    let pool = app_handle.state::<DbPool>();
    let pool = pool.inner().clone();
    let handle_for_scan = app_handle.clone();
    let _ = tokio::task::spawn_blocking(move || {
        if let Ok(mut conn) = pool.get() {
            if let Ok(blacklist_entries) = db::get_scan_blacklist(&conn) {
                let blacklist: std::collections::HashMap<String, (i64, String)> =
                    blacklist_entries
                        .into_iter()
                        .map(|e| (e.path, (e.mtime, e.reason)))
                        .collect();

                let mut filtered = paths_to_scan;
                filtered.retain(|p| {
                    let path_str = p.to_string_lossy().to_string();
                    if let Some(&(bl_mtime, _)) = blacklist.get(&path_str) {
                        if bl_mtime == -1 {
                            return false;
                        }
                        let current_mtime = std::fs::metadata(p)
                            .ok()
                            .and_then(|m| m.modified().ok())
                            .and_then(|t| {
                                t.duration_since(std::time::UNIX_EPOCH).ok()
                            })
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);
                        if current_mtime == bl_mtime {
                            return false;
                        }
                        let _ = db::remove_from_scan_blacklist(&conn, &path_str);
                    }
                    true
                });
                let _ = scanner::scan_files(&mut conn, &handle_for_scan, filtered);
            } else {
                let _ = scanner::scan_files(&mut conn, &handle_for_scan, paths_to_scan);
            }
        }
    })
    .await;
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
