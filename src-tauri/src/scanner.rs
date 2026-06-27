use crate::artist_pic_fetcher;
use crate::db;
use crate::error::{Error, Result};
use crate::sync;
use image::ImageFormat;
use lofty::picture::Picture;
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::ItemKey;
use rayon::prelude::*;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::mpsc;
use tauri::{AppHandle, Emitter, Manager};
use walkdir::WalkDir;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanProgress {
    pub current: usize,
    pub total: usize,
    pub message: String,
}

pub(crate) struct TrackMetadata {
    pub(crate) path: String,
    pub(crate) title: String,
    pub(crate) artists: Vec<String>,
    pub(crate) album: String,
    pub(crate) album_artist: Option<String>,
    pub(crate) release_year: Option<u32>,
    pub(crate) duration: u32,
    pub(crate) mtime: i64,
    pub(crate) file_size: u64,
    pub(crate) picture: Option<Picture>,
    pub(crate) track_number: Option<u32>,
}

pub(crate) fn extract_metadata(path: &Path) -> anyhow::Result<TrackMetadata> {
    let tagged_file = Probe::open(path)?.read()?;

    let properties = tagged_file.properties();
    let duration = properties.duration().as_secs() as u32;

    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());

    let meta = fs::metadata(path)?;
    let mtime = meta
        .modified()?
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;
    let file_size = meta.len();

    let (title, artists, album, album_artist, release_year, picture, track_number) =
        if let Some(t) = tag {
            (
                t.title().map(|s| s.into_owned()).unwrap_or_else(|| {
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Unknown")
                        .to_string()
                }),
                t.artist()
                    .map(|s| s.into_owned())
                    .unwrap_or_else(|| "Unknown Artist".to_string())
                    .split(", ")
                    .map(String::from)
                    .collect(),
                t.album()
                    .map(|s| s.into_owned())
                    .unwrap_or_else(|| "Unknown Album".to_string()),
                t.get_string(&ItemKey::AlbumArtist)
                    .map(|s| s.to_owned())
                    .or_else(|| t.artist().map(|s| s.into_owned())),
                t.get_string(&ItemKey::RecordingDate)
                    .and_then(|s| s.parse::<u32>().ok())
                    .or_else(|| t.year().map(|y| y as u32)),
                t.pictures().first().cloned(),
                t.track(),
            )
        } else {
            (
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                vec!["Unknown Artist".to_string()],
                "Unknown Album".to_string(),
                None,
                None,
                None,
                None,
            )
        };

    Ok(TrackMetadata {
        path: path.to_string_lossy().to_string(),
        title,
        artists,
        album,
        album_artist,
        release_year,
        duration,
        mtime,
        file_size,
        picture,
        track_number,
    })
}

pub fn save_image_to_app_dir(app_dir: &Path, source_path: &str, subdir: &str) -> Result<String> {
    let img = image::open(source_path)
        .map_err(|e| Error::Unknown(format!("Failed to open image: {e}")))?;

    let data = std::fs::read(source_path).map_err(|e| Error::Io(e))?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let hash = hex::encode(hasher.finalize());

    let filename = format!("{hash}.webp");
    let dest_dir = app_dir.join(subdir);
    if !dest_dir.exists() {
        std::fs::create_dir_all(&dest_dir).map_err(Error::Io)?;
    }

    let dest_path = dest_dir.join(&filename);
    if !dest_path.exists() {
        img.save_with_format(&dest_path, ImageFormat::WebP)
            .map_err(|e| Error::Unknown(format!("Failed to save image: {e}")))?;
    }

    Ok(filename)
}

fn save_picture(app_dir: &Path, picture: &Picture) -> anyhow::Result<String> {
    let data = picture.data();
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hex::encode(hasher.finalize());

    let img = image::load_from_memory(data)?.thumbnail(500, 500);

    let filename = format!("{}.webp", hash);
    let covers_dir = app_dir.join("covers");
    if !covers_dir.exists() {
        fs::create_dir_all(&covers_dir)?;
    }

    let dest_path = covers_dir.join(&filename);
    if !dest_path.exists() {
        img.save_with_format(dest_path, ImageFormat::WebP)?;
    }

    Ok(filename)
}

pub fn scan_directories(conn: &mut Connection, app_handle: &AppHandle) -> Result<()> {
    let source_dirs = db::get_source_dirs(conn)?;
    let audio_extensions = ["mp3", "flac", "wav", "ogg", "m4a", "aac", "opus"];

    let _ = app_handle.emit(
        "scan-progress",
        ScanProgress {
            current: 0,
            total: 100,
            message: "Starting scan...".to_string(),
        },
    );

    println!("Starting scan of source directories: {:?}", source_dirs);
    // 1. Discovery
    let mut files_on_disk = Vec::new();
    for dir in &source_dirs {
        let root = Path::new(dir);
        if !root.exists() {
            continue;
        }

        let _ = app_handle.emit(
            "scan-progress",
            ScanProgress {
                current: 10,
                total: 100,
                message: format!("Searching: {}", dir),
            },
        );

        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if audio_extensions.contains(&ext.as_ref()) {
                files_on_disk.push(path.to_path_buf());
            }
        }
    }

    println!("Discovered {} audio files on disk", files_on_disk.len());
    // 2. Differential Analysis
    let _ = app_handle.emit(
        "scan-progress",
        ScanProgress {
            current: 20,
            total: 100,
            message: "Analyzing changes...".to_string(),
        },
    );

    let db_tracks = db::get_all_track_paths_and_mtimes(conn)?;

    let mut to_scan = Vec::new();
    let mut disk_paths_set = HashMap::new();

    for path in files_on_disk {
        let path_str = path.to_string_lossy().to_string();
        let mtime = fs::metadata(&path)
            .and_then(|m| m.modified())
            .map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0)
            })
            .unwrap_or(0);

        disk_paths_set.insert(path_str.clone(), mtime);

        match db_tracks.get(&path_str) {
            Some(&db_mtime) if db_mtime >= mtime => continue,
            _ => to_scan.push(path),
        }
    }

    // Identify removed tracks
    let mut removed_paths = Vec::new();
    for path in db_tracks.keys() {
        let is_in_source = source_dirs.iter().any(|d| path.starts_with(d));
        if is_in_source && !disk_paths_set.contains_key(path) {
            removed_paths.push(path.clone());
        }
    }

    if !removed_paths.is_empty() {
        let _ = app_handle.emit(
            "scan-progress",
            ScanProgress {
                current: 25,
                total: 100,
                message: format!("Cleaning up {} removed tracks...", removed_paths.len()),
            },
        );
        let tx = conn.transaction().map_err(Error::Db)?;
        db::delete_tracks_by_paths(&tx, &removed_paths)?;
        tx.commit().map_err(Error::Db)?;
    }

    scan_files(conn, app_handle, to_scan)?;

    let _ = app_handle.emit(
        "scan-progress",
        ScanProgress {
            current: 100,
            total: 100,
            message: "Scan complete!".to_string(),
        },
    );
    let _ = app_handle.emit("library-updated", ());

    Ok(())
}

pub fn scan_files(
    conn: &mut Connection,
    app_handle: &AppHandle,
    to_scan: Vec<std::path::PathBuf>,
) -> Result<()> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| Error::Unknown(e.to_string()))?;

    let total = to_scan.len();
    if total == 0 {
        return Ok(());
    }

    let (tx_progress, _) = mpsc::channel();

    let metadata_results: Vec<TrackMetadata> = to_scan
        .into_par_iter()
        .filter_map(|path| {
            let result = extract_metadata(&path);

            let _ = tx_progress.send(
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string(),
            );

            match result {
                Ok(m) => Some(m),
                Err(e) => {
                    eprintln!("Failed to scan {:?}: {}", path, e);
                    None
                }
            }
        })
        .collect();

    println!("Extracted metadata for {} files", metadata_results.len());

    // 4. Batch Database Update
    println!("Updating db...");
    let _ = app_handle.emit(
        "scan-progress",
        ScanProgress {
            current: 75,
            total: 100,
            message: "Saving to database...".to_string(),
        },
    );

    let pool = app_handle.state::<db::DbPool>();

    // Process all cover art in parallel before opening the DB transaction
    let metadata_with_covers: Vec<(TrackMetadata, Option<String>)> = metadata_results
        .into_par_iter()
        .map(|meta| {
            let cover_url = meta.picture.as_ref().and_then(|pic| {
                save_picture(&app_dir, pic)
                    .inspect_err(|e| eprintln!("Failed to save picture: {e}"))
                    .ok()
            });
            (meta, cover_url)
        })
        .collect();

    let mut artist_cache = HashMap::new();
    let mut unique_artists_to_fetch = HashMap::new();
    let mut album_cache = HashMap::new();

    let track_count = metadata_with_covers.len();
    let progress_step = if track_count > 0 {
        (track_count / 20).max(1)
    } else {
        1
    };

    let tx = conn.transaction().map_err(Error::Db)?;

    for (i, (meta, cover_url)) in metadata_with_covers.iter().enumerate() {
        let artist_names: Vec<String> = meta.artists.clone();
        let mut artist_ids = Vec::new();

        for name in &artist_names {
            let cache_key = name.to_lowercase();
            let id = if let Some(&id) = artist_cache.get(&cache_key) {
                id
            } else {
                let id = db::get_or_create_artist(&tx, name)?;
                artist_cache.insert(cache_key, id);
                unique_artists_to_fetch.insert(id, name.clone());
                id
            };
            artist_ids.push(id);
        }

        let album_key = meta.album.to_lowercase();
        let album_id = if let Some(&id) = album_cache.get(&album_key) {
            id
        } else {
            let id = db::get_or_create_album(
                &tx,
                &meta.album,
                cover_url.as_deref(),
                meta.release_year.map(|y| y as i32),
                meta.album_artist.as_deref(),
            )?;
            album_cache.insert(album_key, id);
            id
        };

        let track_id = db::update_track(
            &tx,
            &meta.path,
            &meta.title,
            meta.duration,
            meta.release_year.map(|y| y as i32),
            meta.mtime,
            meta.file_size as i64,
            cover_url.as_deref(),
        )?;

        db::set_track_artists(&tx, track_id, &artist_ids)?;
        db::set_track_album(
            &tx,
            track_id,
            album_id,
            meta.track_number.unwrap_or(1) as i32,
        )?;

        if i % progress_step == 0 && i > 0 {
            let pct = 75 + (i * 20 / track_count).min(20);
            let _ = app_handle.emit(
                "scan-progress",
                ScanProgress {
                    current: pct,
                    total: 100,
                    message: format!("Saving to database ({}/{})", i, track_count),
                },
            );
        }
    }

    tx.commit().map_err(Error::Db)?;

    let _ = app_handle.emit(
        "scan-progress",
        ScanProgress {
            current: 100,
            total: 100,
            message: "Updates saved".to_string(),
        },
    );
    let _ = app_handle.emit("library-updated", ());

    // Background artist pic fetch for unique artists
    if !unique_artists_to_fetch.is_empty() {
        let fetch_pic = sync::get_setting(app_handle, "autoFetchArtistPic", true).unwrap_or(true);
        let fetch_banner =
            sync::get_setting(app_handle, "autoFetchArtistBanner", true).unwrap_or(true);

        if fetch_pic || fetch_banner {
            let app_handle_clone = app_handle.clone();
            let app_dir_clone = app_dir.clone();
            let pool_clone = pool.inner().clone();

            tokio::spawn(async move {
                let _ = artist_pic_fetcher::fetch_artist_images(
                    &unique_artists_to_fetch,
                    &app_dir_clone,
                    pool_clone,
                    &app_handle_clone,
                    fetch_pic,
                    fetch_banner,
                )
                .await;
            });
        }
    }

    Ok(())
}
