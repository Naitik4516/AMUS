use crate::artist_pic_fetcher;
use crate::db;
use crate::error::{Error, Result};
use crate::sync::{self, SyncManager};
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
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use tauri::{AppHandle, Emitter, Manager};
use walkdir::WalkDir;

/// Unique suffix for temporary cover files written concurrently.
static COVER_TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

const PHASE_META_START: usize = 25;
const PHASE_META_END: usize = 55;
const PHASE_COVER_START: usize = 55;
const PHASE_COVER_END: usize = 75;
const PHASE_DB_START: usize = 75;
const PHASE_DB_END: usize = 95;

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
    pub(crate) genre: Option<String>,
    pub(crate) bitrate: Option<u32>,
    pub(crate) sample_rate: u32,
    pub(crate) bit_depth: Option<u8>,
    pub(crate) channels: u8,
    pub(crate) audio_format: String,
    pub(crate) codec: Option<String>,
    pub(crate) bpm: Option<f32>,
    pub(crate) replaygain_track_gain: Option<f32>,
    pub(crate) replaygain_track_peak: Option<f32>,
    pub(crate) replaygain_album_gain: Option<f32>,
    pub(crate) replaygain_album_peak: Option<f32>,
    pub(crate) encoder: Option<String>,
    pub(crate) plain_lyrics: Option<String>,
    pub(crate) synced_lyrics: Option<String>,
    pub(crate) lyrics_source: String,
}

fn split_artists(input: &str) -> Vec<String> {
    let normalized = input
        .replace(" feat. ", ", ")
        .replace(" ft. ", ", ")
        .replace(" featuring ", ", ")
        .replace("; ", ", ")
        .replace(";", ", ");
    normalized
        .split(", ")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// lofty exposes container file types, not codecs; map to a human-readable
/// codec label. MPEG containers cover MP2/MP3 (lofty 0.21 limitation).
fn file_type_to_codec(file_type: lofty::file::FileType) -> Option<String> {
    let label = match file_type {
        lofty::file::FileType::Mpeg => "MPEG Audio",
        lofty::file::FileType::Mp4 | lofty::file::FileType::Aac => "AAC",
        lofty::file::FileType::Flac => "FLAC",
        lofty::file::FileType::Opus => "Opus",
        lofty::file::FileType::Vorbis => "Vorbis",
        lofty::file::FileType::Speex => "Speex",
        lofty::file::FileType::Wav | lofty::file::FileType::Aiff => "PCM",
        lofty::file::FileType::WavPack => "WavPack",
        lofty::file::FileType::Ape => "Monkey's Audio",
        lofty::file::FileType::Mpc => "Musepack",
        lofty::file::FileType::Custom(_) => return None,
        _ => return None,
    };
    Some(label.to_string())
}

pub(crate) fn extract_metadata(path: &Path) -> anyhow::Result<TrackMetadata> {
    let tagged_file = Probe::open(path)?.read()?;

    let properties = tagged_file.properties();
    let duration = properties.duration().as_secs() as u32;
    let sample_rate = properties.sample_rate().unwrap_or(0);
    let bit_depth = properties.bit_depth();
    let channels = properties.channels().unwrap_or(0);
    let bitrate = properties.audio_bitrate();
    let audio_format = format!("{:?}", tagged_file.file_type());

    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());

    let meta = fs::metadata(path)?;
    let mtime = meta
        .modified()?
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;
    let file_size = meta.len();

    // LRC sidecar: look for a .lrc file with same stem
    let lrc_path = path.with_extension("lrc");
    let lrc_content = if lrc_path.exists() {
        fs::read_to_string(&lrc_path).ok()
    } else {
        None
    };

    let (
        title,
        artists,
        album,
        album_artist,
        release_year,
        picture,
        track_number,
        genre,
        bpm,
        rg_track_gain,
        rg_track_peak,
        rg_album_gain,
        rg_album_peak,
        encoder,
        plain_lyrics,
        synced_lyrics,
        lyrics_source,
    ) = if let Some(t) = tag {
        let embedded_plain = t.get_string(&ItemKey::Lyrics).map(|s| s.to_owned());

        let tag_bpm = t
            .get_string(&ItemKey::Bpm)
            .or_else(|| t.get_string(&ItemKey::IntegerBpm))
            .and_then(|s| s.parse::<f32>().ok());
        let tag_rg_track_gain = t
            .get_string(&ItemKey::ReplayGainTrackGain)
            .and_then(|s| s.parse::<f32>().ok());
        let tag_rg_track_peak = t
            .get_string(&ItemKey::ReplayGainTrackPeak)
            .and_then(|s| s.parse::<f32>().ok());
        let tag_rg_album_gain = t
            .get_string(&ItemKey::ReplayGainAlbumGain)
            .and_then(|s| s.parse::<f32>().ok());
        let tag_rg_album_peak = t
            .get_string(&ItemKey::ReplayGainAlbumPeak)
            .and_then(|s| s.parse::<f32>().ok());

        let lyrics_src;
        let (tag_plain, tag_synced) = match (&embedded_plain, &lrc_content) {
            (Some(p), Some(lrc)) => {
                lyrics_src = "embedded+lrc";
                (Some(p.clone()), Some(lrc.clone()))
            }
            (Some(p), None) => {
                lyrics_src = "embedded";
                (Some(p.clone()), None)
            }
            (None, Some(lrc)) => {
                lyrics_src = "lrc_file";
                (None, Some(lrc.clone()))
            }
            (None, None) => {
                lyrics_src = "embedded";
                (None, None)
            }
        };

        (
            t.title().map(|s| s.into_owned()).unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string()
            }),
            split_artists(
                t.artist()
                    .map(|s| s.into_owned())
                    .unwrap_or_else(|| "Unknown Artist".to_string())
                    .as_str(),
            ),
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
            t.get_string(&ItemKey::Genre).map(|s| s.to_owned()),
            tag_bpm,
            tag_rg_track_gain,
            tag_rg_track_peak,
            tag_rg_album_gain,
            tag_rg_album_peak,
            t.get_string(&ItemKey::EncoderSoftware)
                .map(|s| s.to_owned()),
            tag_plain,
            tag_synced,
            lyrics_src.to_string(),
        )
    } else {
        let lyrics_src;
        let (tag_plain, tag_synced) = match &lrc_content {
            Some(lrc) => {
                lyrics_src = "lrc_file";
                (None, Some(lrc.clone()))
            }
            None => {
                lyrics_src = "embedded";
                (None, None)
            }
        };

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
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            tag_plain,
            tag_synced,
            lyrics_src.to_string(),
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
        genre,
        bitrate,
        sample_rate,
        bit_depth,
        channels,
        audio_format,
        codec: file_type_to_codec(tagged_file.file_type()),
        bpm,
        replaygain_track_gain: rg_track_gain,
        replaygain_track_peak: rg_track_peak,
        replaygain_album_gain: rg_album_gain,
        replaygain_album_peak: rg_album_peak,
        encoder,
        plain_lyrics,
        synced_lyrics,
        lyrics_source,
    })
}

fn picture_content_hash(picture: &Picture) -> String {
    let mut hasher = Sha256::new();
    hasher.update(picture.data());
    hex::encode(hasher.finalize())
}

fn encode_and_save_cover(covers_dir: &Path, hash: &str, picture: &Picture) -> anyhow::Result<()> {
    let dest_path = covers_dir.join(format!("{hash}.webp"));
    if dest_path.exists() {
        return Ok(());
    }

    let img = image::load_from_memory(picture.data())?.thumbnail(500, 500);

    let tmp_id = COVER_TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp_path = covers_dir.join(format!(".{hash}.{tmp_id}.tmp.webp"));

    img.save_with_format(&tmp_path, ImageFormat::WebP)?;

    match fs::rename(&tmp_path, &dest_path) {
        Ok(()) => Ok(()),
        Err(_) if dest_path.exists() => {
            let _ = fs::remove_file(&tmp_path);
            Ok(())
        }
        Err(e) => {
            let _ = fs::remove_file(&tmp_path);
            Err(e.into())
        }
    }
}

pub fn save_image_to_app_dir(app_dir: &Path, source_path: &str, subdir: &str) -> Result<String> {
    let data = std::fs::read(source_path).map_err(Error::Io)?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let hash = hex::encode(hasher.finalize());

    let filename = format!("{hash}.webp");
    let dest_dir = app_dir.join(subdir);
    if !dest_dir.exists() {
        std::fs::create_dir_all(&dest_dir).map_err(Error::Io)?;
    }

    let dest_path = dest_dir.join(&filename);
    if dest_path.exists() {
        return Ok(filename);
    }

    let img = image::load_from_memory(&data)
        .map_err(|e| Error::Unknown(format!("Failed to open image: {e}")))?;

    let tmp_id = COVER_TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp_path = dest_dir.join(format!(".{hash}.{tmp_id}.tmp.webp"));
    img.save_with_format(&tmp_path, ImageFormat::WebP)
        .map_err(|e| Error::Unknown(format!("Failed to save image: {e}")))?;

    match std::fs::rename(&tmp_path, &dest_path) {
        Ok(()) => {}
        Err(_) if dest_path.exists() => {
            let _ = std::fs::remove_file(&tmp_path);
        }
        Err(e) => {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(Error::Io(e));
        }
    }

    Ok(filename)
}

fn save_picture(app_dir: &Path, picture: &Picture) -> anyhow::Result<String> {
    let hash = picture_content_hash(picture);
    let filename = format!("{hash}.webp");
    let covers_dir = app_dir.join("covers");

    if covers_dir.join(&filename).exists() {
        return Ok(filename);
    }

    fs::create_dir_all(&covers_dir)?;
    if covers_dir.join(&filename).exists() {
        return Ok(filename);
    }

    encode_and_save_cover(&covers_dir, &hash, picture)?;
    Ok(filename)
}

pub fn ensure_track_in_db(conn: &Connection, path: &Path, app_dir: &Path) -> Result<i64> {
    let meta = extract_metadata(path).map_err(|e| Error::Unknown(e.to_string()))?;

    let mut artist_ids = Vec::new();
    for name in &meta.artists {
        let id = db::get_or_create_artist(conn, name)?;
        artist_ids.push(id);
    }

    let album_id =
        db::get_or_create_album(conn, &meta.album, None, meta.release_year.map(|y| y as i32))?;

    if let Some(ref aa) = meta.album_artist {
        db::set_album_artist_by_id(conn, album_id, aa)?;
    }

    let track_id = db::update_track(
        conn,
        &meta.path,
        &meta.title,
        meta.duration,
        meta.release_year.map(|y| y as i32),
        meta.mtime,
        meta.file_size as i64,
        None,
        meta.genre.as_deref(),
        meta.bitrate,
        meta.sample_rate,
        meta.bit_depth,
        meta.channels,
        &meta.audio_format,
        meta.codec.as_deref(),
        meta.bpm,
        meta.replaygain_track_gain,
        meta.replaygain_track_peak,
        meta.replaygain_album_gain,
        meta.replaygain_album_peak,
        meta.encoder.as_deref(),
    )?;

    let cover_url = meta.picture.as_ref().and_then(|pic| {
        save_picture(app_dir, pic)
            .inspect_err(
                |e| tracing::warn!(error = %e, path = %path.display(), "failed to save picture"),
            )
            .ok()
    });
    if let Some(ref url) = cover_url {
        let _ = conn.execute(
            "UPDATE track SET cover_art = ?1 WHERE id = ?2",
            rusqlite::params![url, track_id],
        );
        let _ = conn.execute(
            "UPDATE album SET cover_art = COALESCE(album.cover_art, ?1) WHERE id = ?2",
            rusqlite::params![url, album_id],
        );
    }

    db::clear_track_artists(conn, track_id)?;
    for &aid in &artist_ids {
        db::bulk_insert_track_artists(conn, &[(track_id, aid)])?;
    }
    db::clear_track_album(conn, track_id)?;
    db::bulk_insert_track_albums(
        conn,
        &[(album_id, track_id, meta.track_number.unwrap_or(1) as i32)],
    )?;

    if let Some(ref genre_str) = meta.genre {
        let genre_names: Vec<&str> = genre_str
            .split(['/', ','].as_ref())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        if !genre_names.is_empty() {
            db::clear_track_genres(conn, track_id)?;
            for name in &genre_names {
                let gid = db::get_or_create_genre(conn, name)?;
                db::bulk_insert_track_genres(conn, &[(track_id, gid)])?;
            }
        }
    }

    db::set_track_lyrics(
        conn,
        track_id,
        meta.plain_lyrics.as_deref(),
        meta.synced_lyrics.as_deref(),
        &meta.lyrics_source,
    )?;

    Ok(track_id)
}

/// True if `path` is exactly `dir` or nested directly under it.
/// Unlike a naive `starts_with`, `/music` does not match `/music2/...`.
fn is_path_within(path: &str, dir: &str) -> bool {
    let dir = dir.trim_end_matches(['/', '\\']);
    if dir.is_empty() {
        return true;
    }
    path == dir
        || path
            .strip_prefix(dir)
            .is_some_and(|rest| rest.starts_with('/') || rest.starts_with('\\'))
}

pub fn scan_directories(conn: &mut Connection, app_handle: &AppHandle) -> Result<()> {
    // Pause realtime file watcher while scanning. The flag is cleared on every
    // exit path — success, error or panic — when the guard drops.
    struct ScanFlagGuard<'a>(Option<&'a SyncManager>);
    impl Drop for ScanFlagGuard<'_> {
        fn drop(&mut self) {
            if let Some(m) = self.0 {
                m.set_scanning(false);
            }
        }
    }
    let sync_manager = app_handle.try_state::<SyncManager>();
    if let Some(m) = &sync_manager {
        m.set_scanning(true);
    }
    let _scan_flag_guard = sync_manager.map(|s| ScanFlagGuard(Some(s.inner())));

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

    // Filter out blacklisted files
    let blacklist_entries = db::get_scan_blacklist(conn)?;
    let blacklist: std::collections::HashMap<String, (i64, String)> = blacklist_entries
        .into_iter()
        .map(|e| (e.path, (e.mtime, e.reason)))
        .collect();

    files_on_disk.retain(|path| {
        let path_str = path.to_string_lossy().to_string();
        if let Some(&(bl_mtime, ref _reason)) = blacklist.get(&path_str) {
            let mtime = fs::metadata(path)
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            if bl_mtime == -1 || bl_mtime == mtime {
                // User-deleted (never rescan) or corrupted file with unchanged mtime
                return false;
            }
            // File changed since blacklisting — remove from blacklist and allow rescan
            let _ = db::remove_from_scan_blacklist(conn, &path_str);
        }
        true
    });

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
        let is_in_source = source_dirs.iter().any(|d| is_path_within(path, d));
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

fn emit_scan_progress(app_handle: &AppHandle, current: usize, total: usize, message: &str) {
    let _ = app_handle.emit(
        "scan-progress",
        ScanProgress {
            current,
            total,
            message: message.to_string(),
        },
    );
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

    emit_scan_progress(app_handle, PHASE_META_START, 100, "Reading metadata...");

    let failed_paths = std::sync::Mutex::new(Vec::new());

    let metadata_results: Vec<TrackMetadata> = to_scan
        .into_par_iter()
        .filter_map(|path| {
            let path_str = path.to_string_lossy().to_string();
            match extract_metadata(&path) {
                Ok(m) => Some(m),
                Err(e) => {
                    tracing::warn!(error = %e, path = %path.display(), "failed to scan");
                    failed_paths
                        .lock()
                        .unwrap()
                        .push((path_str, format!("corrupted: {}", e)));
                    None
                }
            }
        })
        .collect();

    let failed_paths = failed_paths.into_inner().unwrap();

    emit_scan_progress(app_handle, PHASE_META_END, 100, "Metadata read");

    let track_count = metadata_results.len();
    if track_count == 0 {
        return Ok(());
    }

    emit_scan_progress(app_handle, PHASE_COVER_START, 100, "Saving cover art...");

    let covers_dir = app_dir.join("covers");
    fs::create_dir_all(&covers_dir).map_err(Error::Io)?;

    let mut stripped_metadata: Vec<TrackMetadata> = Vec::with_capacity(track_count);
    let mut track_cover_hashes: Vec<Option<String>> = Vec::with_capacity(track_count);
    let mut unique_pictures: HashMap<String, Picture> = HashMap::new();

    for mut meta in metadata_results {
        if let Some(pic) = meta.picture.take() {
            let hash = picture_content_hash(&pic);
            unique_pictures.entry(hash.clone()).or_insert(pic);
            track_cover_hashes.push(Some(hash));
        } else {
            track_cover_hashes.push(None);
        }
        stripped_metadata.push(meta);
    }

    // Only encode covers that are not already on disk.
    let to_encode: Vec<(String, Picture)> = unique_pictures
        .into_iter()
        .filter(|(hash, _)| !covers_dir.join(format!("{hash}.webp")).exists())
        .collect();

    let encode_total = to_encode.len();
    if encode_total > 0 {
        let progress = AtomicUsize::new(0);
        let progress_step = (encode_total / 20).max(1);
        let range = PHASE_COVER_END - PHASE_COVER_START;

        to_encode.into_par_iter().for_each(|(hash, pic)| {
            if let Err(e) = encode_and_save_cover(&covers_dir, &hash, &pic) {
                tracing::warn!(error = %e, hash = %hash, "failed to save picture");
            }
            let n = progress.fetch_add(1, Ordering::Relaxed) + 1;
            if n == 1 || n % progress_step == 0 || n == encode_total {
                let pct = PHASE_COVER_START + (n * range / encode_total);
                emit_scan_progress(
                    app_handle,
                    pct,
                    100,
                    &format!("Saving cover art ({n}/{encode_total})"),
                );
            }
        });
    }

    // Map each track to its cover filename (hash.webp); pictures already freed.
    let metadata_with_covers: Vec<(TrackMetadata, Option<String>)> = stripped_metadata
        .into_iter()
        .zip(track_cover_hashes)
        .map(|(meta, hash)| {
            let cover_url = hash.map(|h| format!("{h}.webp"));
            (meta, cover_url)
        })
        .collect();

    emit_scan_progress(app_handle, PHASE_COVER_END, 100, "Cover art saved");

    let mut artist_cache = HashMap::new();
    let mut unique_artists_to_fetch = HashMap::new();
    let mut album_cache = HashMap::new();
    let mut album_artists: HashMap<i64, String> = HashMap::new();

    let mut track_artist_pairs: Vec<(i64, i64)> = Vec::new();
    let mut track_album_entries: Vec<(i64, i64, i32)> = Vec::new();

    const BATCH_SIZE: usize = 100;
    let mut tx = conn.transaction().map_err(Error::Db)?;
    let progress_step = (track_count / 20).max(1);

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
            )?;
            album_cache.insert(album_key, id);
            id
        };

        if let Some(ref aa) = meta.album_artist {
            album_artists.entry(album_id).or_insert_with(|| aa.clone());
        }

        let track_id = match db::update_track(
            &tx,
            &meta.path,
            &meta.title,
            meta.duration,
            meta.release_year.map(|y| y as i32),
            meta.mtime,
            meta.file_size as i64,
            cover_url.as_deref(),
            meta.genre.as_deref(),
            meta.bitrate,
            meta.sample_rate,
            meta.bit_depth,
            meta.channels,
            &meta.audio_format,
            meta.codec.as_deref(),
            meta.bpm,
            meta.replaygain_track_gain,
            meta.replaygain_track_peak,
            meta.replaygain_album_gain,
            meta.replaygain_album_peak,
            meta.encoder.as_deref(),
        ) {
            Ok(id) => id,
            Err(e) => {
                tracing::warn!(error = %e, path = %meta.path, "failed to write track to DB");
                continue;
            }
        };

        db::clear_track_artists(&tx, track_id)?;
        for &artist_id in &artist_ids {
            track_artist_pairs.push((track_id, artist_id));
        }

        db::clear_track_album(&tx, track_id)?;
        track_album_entries.push((album_id, track_id, meta.track_number.unwrap_or(1) as i32));

        if let Some(ref genre_str) = meta.genre {
            let genre_names: Vec<&str> = genre_str
                .split(['/', ','].as_ref())
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if !genre_names.is_empty() {
                db::clear_track_genres(&tx, track_id)?;
                for name in &genre_names {
                    if let Ok(gid) = db::get_or_create_genre(&tx, name) {
                        let _ = db::bulk_insert_track_genres(&tx, &[(track_id, gid)]);
                    }
                }
            }
        }

        let _ = db::set_track_lyrics(
            &tx,
            track_id,
            meta.plain_lyrics.as_deref(),
            meta.synced_lyrics.as_deref(),
            &meta.lyrics_source,
        );

        // Commit periodically so long scans don't starve other write operations
        if i > 0 && i % BATCH_SIZE == 0 {
            let _ = db::bulk_insert_track_artists(&tx, &track_artist_pairs);
            let _ = db::bulk_insert_track_albums(&tx, &track_album_entries);
            for (&album_id, album_artist_name) in &album_artists {
                let _ = db::set_album_artist_by_id(&tx, album_id, album_artist_name);
            }
            tx.commit().map_err(Error::Db)?;
            tx = conn.transaction().map_err(Error::Db)?;
            track_artist_pairs.clear();
            track_album_entries.clear();
            album_artists.clear();
        }

        if i % progress_step == 0 || i == track_count - 1 {
            let pct = PHASE_DB_START + (i * (PHASE_DB_END - PHASE_DB_START) / track_count);
            emit_scan_progress(
                app_handle,
                pct,
                100,
                &format!("Saving to database ({}/{})", i + 1, track_count),
            );
        }
    }

    db::bulk_insert_track_artists(&tx, &track_artist_pairs)?;
    db::bulk_insert_track_albums(&tx, &track_album_entries)?;

    for (&album_id, album_artist_name) in &album_artists {
        db::set_album_artist_by_id(&tx, album_id, album_artist_name)?;
    }

    tx.commit().map_err(Error::Db)?;

    emit_scan_progress(app_handle, 100, 100, "Updates saved");
    let _ = app_handle.emit("library-updated", ());

    if !unique_artists_to_fetch.is_empty() {
        let fetch_pic = sync::get_setting(app_handle, "autoFetchArtistPic", true).unwrap_or(true);

        if fetch_pic {
            let ids: Vec<i64> = unique_artists_to_fetch.keys().copied().collect();
            if let Ok(needed) = db::get_artists_missing_images(conn, &ids) {
                unique_artists_to_fetch.retain(|id, _| needed.contains(id));
            }

            if !unique_artists_to_fetch.is_empty() {
                let pool = app_handle.state::<db::DbPool>();
                let app_handle_clone = app_handle.clone();
                let app_dir_clone = app_dir.clone();
                let pool_clone = pool.inner().clone();

                tokio::spawn(async move {
                    let _ = artist_pic_fetcher::fetch_artist_images(
                        &unique_artists_to_fetch,
                        &app_dir_clone,
                        pool_clone,
                        &app_handle_clone,
                    )
                    .await;
                });
            }
        }
    }

    // Blacklist failed paths so the scanner skips corrupted files on future runs
    if !failed_paths.is_empty() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        for (path, reason) in &failed_paths {
            let mtime = std::fs::metadata(path)
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(now);

            if let Err(e) = db::add_to_scan_blacklist(conn, path, mtime, reason) {
                tracing::warn!(error = %e, path = %path, "failed to blacklist");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_artists_single() {
        let result = split_artists("John Doe");
        assert_eq!(result, vec!["John Doe"]);
    }

    #[test]
    fn test_split_artists_comma_separated() {
        let result = split_artists("Artist A, Artist B, Artist C");
        assert_eq!(result, vec!["Artist A", "Artist B", "Artist C"]);
    }

    #[test]
    fn test_split_artists_feat() {
        let result = split_artists("Artist A feat. Artist B");
        assert_eq!(result, vec!["Artist A", "Artist B"]);
    }

    #[test]
    fn test_split_artists_ft() {
        let result = split_artists("Artist A ft. Artist B");
        assert_eq!(result, vec!["Artist A", "Artist B"]);
    }

    #[test]
    fn test_split_artists_featuring() {
        let result = split_artists("Artist A featuring Artist B");
        assert_eq!(result, vec!["Artist A", "Artist B"]);
    }

    #[test]
    fn test_split_artists_semicolon() {
        let result = split_artists("Artist A; Artist B");
        assert_eq!(result, vec!["Artist A", "Artist B"]);
    }

    #[test]
    fn test_split_artists_empty() {
        let result = split_artists("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_split_artists_only_separator_returns_empty() {
        // " feat. " is entirely replaced by ", " then split gives empty strings
        let result = split_artists(" feat. ");
        assert!(result.is_empty());
    }

    #[test]
    fn test_split_artists_trims_whitespace() {
        let result = split_artists("  Artist A  ,  Artist B  ");
        assert_eq!(result, vec!["Artist A", "Artist B"]);
    }

    #[test]
    fn test_split_artists_multiple_separators() {
        let result = split_artists("A, B feat. C; D");
        assert_eq!(result, vec!["A", "B", "C", "D"]);
    }

    #[test]
    fn test_split_artists_ampersand_no_split() {
        let result = split_artists("A & B");
        assert_eq!(result, vec!["A & B"]);
    }

    #[test]
    fn test_split_artists_feat_no_spaces() {
        let result = split_artists("A feat.B");
        assert_eq!(result, vec!["A feat.B"]);
    }

    #[test]
    fn test_split_artists_only_whitespace() {
        let result = split_artists("   ");
        assert!(result.is_empty());
    }

    #[test]
    fn test_split_artists_multiple_feat() {
        let result = split_artists("A feat. B feat. C");
        assert_eq!(result, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_extract_metadata_nonexistent() {
        let result = extract_metadata(Path::new("/nonexistent/path.flac"));
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_metadata_wav_no_tags() {
        let tmp = tempfile::TempDir::new().unwrap();
        let wav_path = tmp.path().join("test.wav");
        create_minimal_wav(&wav_path);

        let meta = extract_metadata(&wav_path).unwrap();
        assert_eq!(meta.title, "test");
        assert_eq!(meta.artists, vec!["Unknown Artist"]);
        assert_eq!(meta.album, "Unknown Album");
        assert!(meta.album_artist.is_none());
        assert!(meta.picture.is_none());
        assert!(meta.release_year.is_none());
        assert!(meta.track_number.is_none());
    }

    #[test]
    fn test_save_image_to_app_dir_valid() {
        let tmp = tempfile::TempDir::new().unwrap();
        let app_dir = tmp.path().join("app");
        let src = tmp.path().join("test.png");
        create_test_png(&src);

        let result = save_image_to_app_dir(&app_dir, src.to_str().unwrap(), "covers").unwrap();
        assert!(result.ends_with(".webp"));
        assert!(app_dir.join("covers").join(&result).exists());
    }

    #[test]
    fn test_save_image_to_app_dir_nonexistent() {
        let tmp = tempfile::TempDir::new().unwrap();
        let app_dir = tmp.path().join("app");
        let result = save_image_to_app_dir(&app_dir, "/nonexistent/image.png", "covers");
        assert!(result.is_err());
    }

    #[test]
    fn test_save_picture_valid() {
        let tmp = tempfile::TempDir::new().unwrap();
        let app_dir = tmp.path();

        let png_bytes = create_test_png_bytes();
        let picture = lofty::picture::Picture::new_unchecked(
            lofty::picture::PictureType::CoverFront,
            Some(lofty::picture::MimeType::Png),
            None,
            png_bytes,
        );

        let result = save_picture(app_dir, &picture).unwrap();
        assert!(result.ends_with(".webp"));
        assert!(app_dir.join("covers").join(&result).exists());
    }

    #[test]
    fn test_save_picture_skips_existing_without_reencode() {
        let tmp = tempfile::TempDir::new().unwrap();
        let app_dir = tmp.path();

        let png_bytes = create_test_png_bytes();
        let picture = lofty::picture::Picture::new_unchecked(
            lofty::picture::PictureType::CoverFront,
            Some(lofty::picture::MimeType::Png),
            None,
            png_bytes,
        );

        let filename = save_picture(app_dir, &picture).unwrap();
        let dest = app_dir.join("covers").join(&filename);
        let mtime_before = fs::metadata(&dest).unwrap().modified().unwrap();

        // Second save must reuse the existing file (no rewrite).
        std::thread::sleep(std::time::Duration::from_millis(20));
        let filename2 = save_picture(app_dir, &picture).unwrap();
        assert_eq!(filename, filename2);
        let mtime_after = fs::metadata(&dest).unwrap().modified().unwrap();
        assert_eq!(mtime_before, mtime_after);
    }

    #[test]
    fn test_picture_content_hash_stable() {
        let png_bytes = create_test_png_bytes();
        let picture = lofty::picture::Picture::new_unchecked(
            lofty::picture::PictureType::CoverFront,
            Some(lofty::picture::MimeType::Png),
            None,
            png_bytes,
        );
        let h1 = picture_content_hash(&picture);
        let h2 = picture_content_hash(&picture);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // sha256 hex
    }

    #[test]
    fn test_save_covers_dedupes_identical_art() {
        let tmp = tempfile::TempDir::new().unwrap();
        let app_dir = tmp.path();
        let covers_dir = app_dir.join("covers");
        fs::create_dir_all(&covers_dir).unwrap();

        let png_bytes = create_test_png_bytes();
        let picture = lofty::picture::Picture::new_unchecked(
            lofty::picture::PictureType::CoverFront,
            Some(lofty::picture::MimeType::Png),
            None,
            png_bytes,
        );
        let hash = picture_content_hash(&picture);

        // Simulate many tracks sharing one cover — only one encode should run.
        let mut unique: HashMap<String, Picture> = HashMap::new();
        for _ in 0..20 {
            unique
                .entry(hash.clone())
                .or_insert_with(|| picture.clone());
        }
        assert_eq!(unique.len(), 1);

        encode_and_save_cover(&covers_dir, &hash, unique.get(&hash).unwrap()).unwrap();
        assert!(covers_dir.join(format!("{hash}.webp")).exists());
    }

    #[test]
    fn test_ensure_track_in_db_with_wav() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        db::init_db(&mut conn).unwrap();

        let tmp = tempfile::TempDir::new().unwrap();
        let wav_path = tmp.path().join("test.wav");
        create_minimal_wav(&wav_path);

        let track_id = ensure_track_in_db(&conn, &wav_path, tmp.path()).unwrap();
        assert!(track_id > 0);

        let (title, artist_count): (String, i64) = conn
            .query_row(
                "SELECT t.title, (SELECT COUNT(*) FROM track_artist WHERE track_id = ?1) FROM track t WHERE t.id = ?1",
                rusqlite::params![track_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(title, "test");
        assert_eq!(artist_count, 1);
    }

    // --- Helpers ---

    fn create_minimal_wav(path: &Path) {
        let wav_data: Vec<u8> = vec![
            0x52, 0x49, 0x46, 0x46, // "RIFF"
            0x26, 0x00, 0x00, 0x00, // file size - 8 (38)
            0x57, 0x41, 0x56, 0x45, // "WAVE"
            0x66, 0x6d, 0x74, 0x20, // "fmt "
            0x10, 0x00, 0x00, 0x00, // fmt chunk size (16)
            0x01, 0x00, // PCM format
            0x01, 0x00, // 1 channel
            0x44, 0xac, 0x00, 0x00, // 44100 Hz
            0x88, 0x58, 0x01, 0x00, // byte rate (88200)
            0x02, 0x00, // block align (2)
            0x10, 0x00, // 16 bits per sample
            0x64, 0x61, 0x74, 0x61, // "data"
            0x02, 0x00, 0x00, 0x00, // data size (2)
            0x00, 0x00, // one silent 16-bit sample
        ];
        fs::write(path, wav_data).unwrap();
    }

    fn create_test_png(path: &Path) {
        let img = image::RgbaImage::new(1, 1);
        img.save(path).unwrap();
    }

    fn create_test_png_bytes() -> Vec<u8> {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("t.png");
        create_test_png(&path);
        fs::read(&path).unwrap()
    }
}
