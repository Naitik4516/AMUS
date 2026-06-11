use crate::db;
use anyhow::{Context, Result};
use lofty::prelude::*;
use lofty::probe::Probe;
use rusqlite::Connection;
use std::path::Path;
use walkdir::WalkDir;

pub fn extract_metadata(path: &Path) -> Result<(String, String, String, String, u32)> {
    let tagged_file = Probe::open(path)
        .context(format!("Failed to open file: {:?}", path))?
        .read()?;

    let properties = tagged_file.properties();
    let duration = properties.duration().as_secs() as u32;

    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());

    let title = tag
        .and_then(|t| t.title().map(|s| s.into_owned()))
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string()
        });
    let artist = tag
        .and_then(|t| t.artist().map(|s| s.into_owned()))
        .unwrap_or_else(|| "Unknown Artist".to_string());
    let album = tag
        .and_then(|t| t.album().map(|s| s.into_owned()))
        .unwrap_or_else(|| "Unknown Album".to_string());
    let genre = tag
        .and_then(|t| t.genre().map(|s| s.into_owned()))
        .unwrap_or_else(|| "Unknown Genre".to_string());

    Ok((title, artist, album, genre, duration))
}

pub fn scan_directories(conn: &Connection) -> Result<()> {
    let dirs = db::get_source_dirs(conn)?;
    let audio_extensions = ["mp3", "flac", "wav", "ogg", "m4a", "aac", "opus"];

    for dir in dirs {
        let root = Path::new(&dir);
        if !root.exists() {
            continue;
        }

        for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if !audio_extensions.contains(&ext.as_str()) {
                continue;
            }

            let metadata = entry.metadata()?;
            let mtime = metadata
                .modified()?
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs() as i64;

            let path_str = path.to_str().context("Non-UTF8 path")?;

            let db_mtime = db::get_track_mtime(conn, path_str)?;

            if db_mtime.is_none() || db_mtime.unwrap() < mtime {
                let (title, artist_name, album_title, genre_name, duration) =
                    extract_metadata(path)?;

                let artist_id = db::get_or_create_artist(conn, &artist_name)?;
                let album_id = db::get_or_create_album(conn, &album_title, artist_id)?;
                let genre_id = db::get_or_create_genre(conn, &genre_name)?;

                db::update_track(
                    conn, path_str, &title, album_id, artist_id, genre_id, duration, mtime,
                )?;
                println!("Scanned/Updated: {}", path_str);
            }
        }
    }
    Ok(())
}
