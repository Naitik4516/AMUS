use crate::artist_pic_fetcher;
use crate::db;
use crate::error::{Error, Result};
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
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use walkdir::WalkDir;

#[derive(Debug, Clone, serde::Serialize)]
struct ScanProgress {
    current: usize,
    total: usize,
    message: String,
}

pub(crate) struct TrackMetadata {
    pub(crate) path: String,
    pub(crate) title: String,
    pub(crate) artist: String,
    pub(crate) album: String,
    pub(crate) album_artist: Option<String>,
    pub(crate) genre: String,
    pub(crate) duration: u32,
    pub(crate) mtime: i64,
    pub(crate) picture: Option<Picture>,
}

pub(crate) fn extract_metadata(path: &Path) -> anyhow::Result<TrackMetadata> {
    let tagged_file = Probe::open(path)?.read()?;

    let properties = tagged_file.properties();
    let duration = properties.duration().as_secs() as u32;

    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());

    let mtime = fs::metadata(path)?
        .modified()?
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;

    let (title, artist, album, album_artist, genre, picture) = if let Some(t) = tag {
        (
            t.title().map(|s| s.into_owned()).unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string()
            }),
            t.artist()
                .map(|s| s.into_owned())
                .unwrap_or_else(|| "Unknown Artist".to_string()),
            t.album()
                .map(|s| s.into_owned())
                .unwrap_or_else(|| "Unknown Album".to_string()),
            t.get_string(&ItemKey::AlbumArtist).map(|s| s.to_string()),
            t.genre()
                .map(|s| s.into_owned())
                .unwrap_or_else(|| "Unknown Genre".to_string()),
            t.pictures().first().cloned(),
        )
    } else {
        (
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string(),
            "Unknown Artist".to_string(),
            "Unknown Album".to_string(),
            None,
            "Unknown Genre".to_string(),
            None,
        )
    };

    Ok(TrackMetadata {
        path: path.to_string_lossy().to_string(),
        title,
        artist,
        album,
        album_artist,
        genre,
        duration,
        mtime,
        picture,
    })
}

fn save_picture(app_dir: &Path, picture: &Picture) -> anyhow::Result<String> {
    let data = picture.data();
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hex::encode(hasher.finalize());

    let ext = match picture.mime_type() {
        Some(lofty::picture::MimeType::Jpeg) => "jpg",
        Some(lofty::picture::MimeType::Png) => "png",
        Some(lofty::picture::MimeType::Gif) => "gif",
        Some(lofty::picture::MimeType::Bmp) => "bmp",
        _ => "img",
    };

    let filename = format!("{}.{}", hash, ext);
    let covers_dir = app_dir.join("covers");
    if !covers_dir.exists() {
        fs::create_dir_all(&covers_dir)?;
    }

    let dest_path = covers_dir.join(&filename);
    if !dest_path.exists() {
        fs::write(dest_path, data)?;
    }

    Ok(filename)
}

pub fn scan_directories(conn: &mut Connection, app_handle: &AppHandle) -> Result<()> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| Error::Unknown(e.to_string()))?;

    let source_dirs = db::get_source_dirs(conn)?;
    let audio_extensions = ["mp3", "flac", "wav", "ogg", "m4a", "aac", "opus"];


    println!("Starting scan of source directories: {:?}", source_dirs);
    // 1. Discovery
    let mut files_on_disk = Vec::new();
    for dir in &source_dirs {
        let root = Path::new(dir);
        if !root.exists() {
            continue;
        }

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
        db::delete_tracks_by_paths(conn, &removed_paths)?;
    }


    println!("Extracting metadata...");
    // 3. Parallel Metadata Extraction
    let total = to_scan.len();
    if total == 0 {
        return Ok(());
    }

    let progress = Mutex::new(0);

    let metadata_results: Vec<TrackMetadata> = to_scan
        .into_par_iter()
        .filter_map(|path| {
            let result = extract_metadata(&path);

            let mut p = progress.lock().unwrap();
            *p += 1;
            if *p % 10 == 0 || *p == total {
                let _ = app_handle.emit(
                    "scan-progress",
                    ScanProgress {
                        current: *p,
                        total,
                        message: format!(
                            "Scanning: {}",
                            path.file_name().and_then(|n| n.to_str()).unwrap_or("")
                        ),
                    },
                );
            }

            match result {
                Ok(m) => Some(m),
                Err(e) => {
                    eprintln!("Failed to scan {:?}: {}", path, e);
                    None
                }
            }
        })
        .collect();

    // 4. Batch Database Update
    println!("Updating db...");
    let pool = app_handle.state::<db::DbPool>();

    // Caches for lookups
    let mut artist_cache = HashMap::new();
    let mut album_cache = HashMap::new();
    let mut genre_cache = HashMap::new();

    let tx = conn.transaction().map_err(Error::Db)?;

    for meta in metadata_results {
        let artist_names: Vec<String> = meta.artist
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let mut artist_ids = Vec::new();
        let mut primary_artist_id = None;

        for name in &artist_names {
            let cache_key = name.to_lowercase();
            let id = if let Some(&id) = artist_cache.get(&cache_key) {
                id
            } else {
                let id = db::get_or_create_artist(&tx, name)?;
                artist_cache.insert(cache_key, id);
                id
            };
            artist_ids.push(id);
            if primary_artist_id.is_none() {
                primary_artist_id = Some(id);
            }
        }

        let artist_id = primary_artist_id.unwrap_or_else(|| {
            // Fallback: shouldn't happen since artist_names is non-empty from extract_metadata
            db::get_or_create_artist(&tx, "Unknown Artist").unwrap_or(1)
        });

        let album_artist_id = if let Some(ref aa) = meta.album_artist {
            let cache_key = aa.to_lowercase();
            if let Some(&id) = artist_cache.get(&cache_key) {
                id
            } else {
                let id = db::get_or_create_artist(&tx, aa)?;
                artist_cache.insert(cache_key, id);
                id
            }
        } else {
            artist_id
        };

        let cover_url = if let Some(pic) = meta.picture {
            match save_picture(&app_dir, &pic) {
                Ok(filename) => Some(filename),
                Err(e) => {
                    eprintln!("Failed to save picture: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let album_key = format!("{}:{}", meta.album.to_lowercase(), album_artist_id);
        let album_id = if let Some(&id) = album_cache.get(&album_key) {
            id
        } else {
            let id = db::get_or_create_album(&tx, &meta.album, album_artist_id, cover_url.as_deref())?;
            album_cache.insert(album_key, id);
            id
        };

        let genre_key = meta.genre.to_lowercase();
        let genre_id = if let Some(&id) = genre_cache.get(&genre_key) {
            id
        } else {
            let id = db::get_or_create_genre(&tx, &meta.genre)?;
            genre_cache.insert(genre_key, id);
            id
        };

        db::update_track(
            &tx,
            &meta.path,
            &meta.title,
            album_id,
            artist_id,
            genre_id,
            meta.duration,
            meta.mtime,
            cover_url.as_deref(),
        )?;

        db::set_track_artists(&tx, db::get_track_id_by_path(&tx, &meta.path)?, &artist_ids)?;

        // Background artist pic fetch for all artists
        for &aid in &artist_ids {
            let artist_name = artist_cache.iter()
                .find(|(_, id)| **id == aid)
                .map(|(name, _)| name.clone())
                .unwrap_or_default();
            if artist_name.is_empty() || artist_name == "Unknown Artist" {
                continue;
            }
            let app_dir_clone = app_dir.clone();
            let pool_clone = pool.inner().clone();
            tokio::spawn(async move {
                match artist_pic_fetcher::fetch_artist_image(&artist_name, &app_dir_clone).await {
                    Ok(filename) => {
                        if let Ok(conn) = pool_clone.get() {
                            let _ = db::update_artist_profile_picture(&conn, aid, &filename);
                        }
                    }
                    Err(e) => eprintln!("Failed to fetch artist image for {}: {}", artist_name, e),
                }
            });
        }
    }

    tx.commit().map_err(Error::Db)?;

    Ok(())
}
