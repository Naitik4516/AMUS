use crate::error::{Error, Result};
use crate::models::*;
use rusqlite::{Connection, OptionalExtension, Params, params, types::ToSql};
use rusqlite_migration::{M, Migrations};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub type DbPool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
pub enum Timeframe {
    Today,
    ThisWeek,
    ThisMonth,
    Last3Months,
    Last6Months,
    ThisYear,
    LastYear,
    Last5Years,
    #[default]
    AllTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TopSort {
    #[default]
    Plays,
    Time,
}


fn timeframe_window_days(timeframe: Timeframe) -> Option<i64> {
    match timeframe {
        Timeframe::Today => Some(1),
        Timeframe::ThisWeek => Some(7),
        Timeframe::ThisMonth => Some(30),
        Timeframe::Last3Months => Some(90),
        Timeframe::Last6Months => Some(180),
        Timeframe::ThisYear | Timeframe::LastYear => Some(365),
        Timeframe::Last5Years => Some(1825),
        Timeframe::AllTime => None,
    }
}

pub fn timeframe_where_clause(alias: &str, timeframe: Timeframe) -> String {
    match timeframe {
        Timeframe::Today => {
            format!("{} >= datetime('now', 'localtime', 'start of day')", alias)
        }
        Timeframe::ThisWeek => {
            format!("{} >= datetime('now', 'localtime', '-7 days')", alias)
        }
        Timeframe::ThisMonth => {
            format!("{} >= datetime('now', 'localtime', '-30 days')", alias)
        }
        Timeframe::Last3Months => {
            format!("{} >= datetime('now', 'localtime', '-90 days')", alias)
        }
        Timeframe::Last6Months => {
            format!("{} >= datetime('now', 'localtime', '-180 days')", alias)
        }
        Timeframe::ThisYear => {
            format!("{} >= datetime('now', 'localtime', 'start of year')", alias)
        }
        Timeframe::LastYear => {
            format!("{} >= datetime('now', 'localtime', '-1 year')", alias)
        }
        Timeframe::Last5Years => {
            format!("{} >= datetime('now', 'localtime', '-5 years')", alias)
        }
        Timeframe::AllTime => "1=1".to_string(),
    }
}

fn top_order_by(sort: TopSort, name_col: &str) -> String {
    match sort {
        TopSort::Plays => {
            format!("play_count DESC, total_time DESC, {name_col} COLLATE NOCASE ASC")
        }
        TopSort::Time => format!("total_time DESC, play_count DESC, {name_col} COLLATE NOCASE ASC"),
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    Title,
    Artist,
    Album,
    Duration,
    RecentlyAdded,
}

const MIGRATIONS_SLICE: &[M<'_>] = &[
    M::up(include_str!("../migrations/001_initial_schema.sql")),
    M::up(include_str!("../migrations/002_add_album_artist.sql")),
    M::up(include_str!("../migrations/003_add_fetch_tracking.sql")),
    M::up(include_str!("../migrations/004_add_playlist_cover_art.sql")),
    M::up(include_str!("../migrations/005_add_user_queue_index.sql")),
    M::up(include_str!(
        "../migrations/006_alter_playback_history_source_type.sql"
    )),
    M::up(include_str!("../migrations/007_scan_blacklist.sql")),
    M::up(include_str!("../migrations/008_extended_metadata.sql")),
    M::up(include_str!("../migrations/009_genre_thumbnail.sql")),
    M::up(include_str!("../migrations/010_lyrics_fetch_attempts.sql")),
    M::up(include_str!("../migrations/011_drop_musical_key.sql")),
];

const MIGRATIONS: Migrations<'_> = Migrations::from_slice(MIGRATIONS_SLICE);

pub fn init_db(conn: &mut Connection) -> Result<()> {
    MIGRATIONS
        .to_latest(conn)
        .map_err(|e| Error::Migration(e.to_string()))?;
    Ok(())
}

pub fn add_source_dir(conn: &Connection, path: &str) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO source_dirs (path) VALUES (?)",
        params![path],
    )
    .map_err(Error::Db)?;
    Ok(())
}

pub fn get_source_dirs(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn
        .prepare("SELECT path FROM source_dirs")
        .map_err(Error::Db)?;
    let dirs_iter = stmt.query_map([], |row| row.get(0)).map_err(Error::Db)?;

    dirs_iter
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)
}

pub fn remove_source_dir(conn: &mut Connection, path: &str) -> Result<()> {
    let tx = conn.transaction().map_err(Error::Db)?;


    let path = path.trim_end_matches(['/', '\\']);

    tx.execute(
        "DELETE FROM track WHERE path = ? OR path LIKE ? || '/%' OR path LIKE ? || '\\%'",
        params![path, path, path],
    )
    .map_err(Error::Db)?;
    tx.execute("DELETE FROM source_dirs WHERE path = ?", [path])
        .map_err(Error::Db)?;

    tx.execute(
        "DELETE FROM artist WHERE id NOT IN (SELECT DISTINCT artist_id FROM track_artist)",
        [],
    )
    .map_err(Error::Db)?;
    tx.execute(
        "DELETE FROM album WHERE id NOT IN (SELECT DISTINCT album_id FROM album_track)",
        [],
    )
    .map_err(Error::Db)?;

    tx.commit().map_err(Error::Db)?;
    Ok(())
}

pub fn report_fetch_success(conn: &Connection, artist_id: i64) -> Result<()> {
    conn.prepare_cached(
        "UPDATE artist SET fetch_attempts = 0, last_fetch_attempt = NULL WHERE id = ?",
    )
    .map_err(Error::Db)?
    .execute(params![artist_id])
    .map_err(Error::Db)?;
    Ok(())
}

pub fn report_fetch_failure(conn: &Connection, artist_id: i64) -> Result<()> {
    conn
        .prepare_cached(
            "UPDATE artist SET fetch_attempts = fetch_attempts + 1, last_fetch_attempt = datetime('now') WHERE id = ?",
        )
        .map_err(Error::Db)?
        .execute(params![artist_id])
        .map_err(Error::Db)?;
    Ok(())
}

pub fn get_all_track_paths_and_mtimes(conn: &Connection) -> Result<HashMap<String, i64>> {
    let mut stmt = conn
        .prepare("SELECT path, mtime FROM track")
        .map_err(Error::Db)?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(Error::Db)?;

    let mut map = HashMap::new();
    for row in rows {
        let (path, mtime) = row.map_err(Error::Db)?;
        map.insert(path, mtime);
    }
    Ok(map)
}

pub fn delete_tracks_by_paths(conn: &Connection, paths: &[String]) -> Result<usize> {
    if paths.is_empty() {
        return Ok(0);
    }

    let mut total_deleted = 0;
    for chunk in paths.chunks(900) {
        let placeholders = sql_placeholders(chunk.len());

        let sql = format!("DELETE FROM track WHERE path IN ({})", placeholders);
        let mut stmt = conn.prepare(&sql).map_err(Error::Db)?;
        let count = stmt
            .execute(rusqlite::params_from_iter(chunk))
            .map_err(Error::Db)?;
        total_deleted += count;
    }

    conn.execute(
        "DELETE FROM artist WHERE id NOT IN (SELECT DISTINCT artist_id FROM track_artist);",
        [],
    )
    .map_err(Error::Db)?;

    conn.execute(
        "DELETE FROM album WHERE id NOT IN (SELECT DISTINCT album_id FROM album_track);",
        [],
    )
    .map_err(Error::Db)?;

    Ok(total_deleted)
}

pub fn delete_track_by_id(conn: &Connection, id: i64) -> Result<String> {
    let path: String = conn
        .query_row("SELECT path FROM track WHERE id = ?1", params![id], |row| {
            row.get(0)
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Error::Unknown(format!("Track {id} not found")),
            e => Error::Db(e),
        })?;

    conn.execute("DELETE FROM track WHERE id = ?1", params![id])
        .map_err(Error::Db)?;

    conn.execute(
        "DELETE FROM artist WHERE id NOT IN (SELECT DISTINCT artist_id FROM track_artist);",
        [],
    )
    .map_err(Error::Db)?;

    conn.execute(
        "DELETE FROM album WHERE id NOT IN (SELECT DISTINCT album_id FROM album_track);",
        [],
    )
    .map_err(Error::Db)?;

    Ok(path)
}

pub fn add_to_scan_blacklist(
    conn: &Connection,
    path: &str,
    mtime: i64,
    reason: &str,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO scan_blacklist (path, mtime, reason) VALUES (?1, ?2, ?3)",
        params![path, mtime, reason],
    )
    .map_err(Error::Db)?;
    Ok(())
}

pub fn remove_from_scan_blacklist(conn: &Connection, path: &str) -> Result<()> {
    conn.execute("DELETE FROM scan_blacklist WHERE path = ?1", params![path])
        .map_err(Error::Db)?;
    Ok(())
}

pub fn get_scan_blacklist(conn: &Connection) -> Result<Vec<BlacklistedEntry>> {
    let mut stmt = conn
        .prepare(
            "SELECT path, mtime, reason, created_at FROM scan_blacklist ORDER BY created_at DESC",
        )
        .map_err(Error::Db)?;
    let rows = stmt
        .query_map([], |row| {
            Ok(BlacklistedEntry {
                path: row.get(0)?,
                mtime: row.get(1)?,
                reason: row.get(2)?,
                created_at: row.get::<_, String>(3)?,
            })
        })
        .map_err(Error::Db)?;
    let mut entries = Vec::new();
    for row in rows {
        entries.push(row.map_err(Error::Db)?);
    }
    Ok(entries)
}

pub fn get_or_create_artist(conn: &Connection, name: &str) -> Result<i64> {
    conn
        .prepare_cached(
            "INSERT INTO artist (name) VALUES (?1) ON CONFLICT(name) DO UPDATE SET name=?1 RETURNING id",
        )
        .map_err(Error::Db)?
        .query_row(params![name], |row| row.get(0))
        .map_err(Error::Db)
}

pub fn get_or_create_album(
    conn: &Connection,
    name: &str,
    cover_art: Option<&str>,
    year: Option<i32>,
) -> Result<i64> {
    conn.prepare_cached(
        "INSERT INTO album (name, cover_art, year) VALUES (?1, ?2, ?3)
             ON CONFLICT(name) DO UPDATE SET
               name = excluded.name,
               cover_art = COALESCE(excluded.cover_art, album.cover_art),
               year = COALESCE(excluded.year, album.year)
             RETURNING id",
    )
    .map_err(Error::Db)?
    .query_row(params![name, cover_art, year], |row| row.get(0))
    .map_err(Error::Db)
}

pub fn set_track_artists(conn: &Connection, track_id: i64, artist_ids: &[i64]) -> Result<()> {
    let existing: HashSet<i64> = conn
        .prepare_cached("SELECT artist_id FROM track_artist WHERE track_id = ?")
        .map_err(Error::Db)?
        .query_map(params![track_id], |row| row.get(0))
        .map_err(Error::Db)?
        .collect::<std::result::Result<HashSet<_>, _>>()
        .map_err(Error::Db)?;

    let new_set: HashSet<i64> = artist_ids.iter().copied().collect();

    if existing == new_set {
        return Ok(());
    }

    conn.prepare_cached("DELETE FROM track_artist WHERE track_id = ?")
        .map_err(Error::Db)?
        .execute(params![track_id])
        .map_err(Error::Db)?;

    if artist_ids.is_empty() {
        return Ok(());
    }

    let placeholders: Vec<String> = artist_ids.iter().map(|_| "(?, ?)".to_string()).collect();
    let sql = format!(
        "INSERT OR IGNORE INTO track_artist (track_id, artist_id) VALUES {}",
        placeholders.join(", ")
    );

    let mut param_values: Vec<Box<dyn ToSql>> = Vec::with_capacity(artist_ids.len() * 2);
    for &artist_id in artist_ids {
        param_values.push(Box::new(track_id));
        param_values.push(Box::new(artist_id));
    }
    let params_refs: Vec<&dyn ToSql> = param_values.iter().map(|p| p.as_ref()).collect();

    conn.prepare_cached(&sql)
        .map_err(Error::Db)?
        .execute(params_refs.as_slice())
        .map_err(Error::Db)?;
    Ok(())
}

pub fn get_artists_needing_fetch(conn: &Connection) -> Result<Vec<(i64, String)>> {
    let mut stmt = conn
        .prepare_cached(
            "SELECT id, name FROM artist
             WHERE (profile_image IS NULL OR banner_image IS NULL)
             AND fetch_attempts < 3
             AND name != 'Unknown Artist'",
        )
        .map_err(Error::Db)?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(Error::Db)?;
    let mut artists = Vec::new();
    for row in rows {
        artists.push(row.map_err(Error::Db)?);
    }
    Ok(artists)
}

/// Returns the subset of `ids` that still need an artist image fetched
/// (missing profile/banner and fetch attempts not exhausted).
pub fn get_artists_missing_images(
    conn: &Connection,
    ids: &[i64],
) -> Result<std::collections::HashSet<i64>> {
    if ids.is_empty() {
        return Ok(std::collections::HashSet::new());
    }
    let placeholders = vec!["?"; ids.len()].join(", ");
    let sql = format!(
        "SELECT id FROM artist WHERE id IN ({placeholders})
         AND (profile_image IS NULL OR banner_image IS NULL)
         AND fetch_attempts < 3"
    );
    let mut stmt = conn.prepare(&sql).map_err(Error::Db)?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(ids), |row| row.get::<_, i64>(0))
        .map_err(Error::Db)?;
    let mut result = std::collections::HashSet::new();
    for row in rows {
        result.insert(row.map_err(Error::Db)?);
    }
    Ok(result)
}

pub fn set_album_artist(conn: &Connection, name: &str, album_artist: &str) -> Result<()> {
    conn.prepare_cached(
        "UPDATE album SET album_artist = ? WHERE name = ? AND album_artist IS NULL",
    )
    .map_err(Error::Db)?
    .execute(params![album_artist, name])
    .map_err(Error::Db)?;
    Ok(())
}

pub fn set_album_artist_by_id(conn: &Connection, album_id: i64, album_artist: &str) -> Result<()> {
    conn.prepare_cached("UPDATE album SET album_artist = ? WHERE id = ? AND album_artist IS NULL")
        .map_err(Error::Db)?
        .execute(params![album_artist, album_id])
        .map_err(Error::Db)?;
    Ok(())
}

pub fn update_track(
    conn: &Connection,
    path: &str,
    title: &str,
    duration_sec: u32,
    year: Option<i32>,
    mtime: i64,
    file_size: i64,
    cover_art: Option<&str>,
    genre: Option<&str>,
    bitrate: Option<u32>,
    sample_rate: u32,
    bit_depth: Option<u8>,
    channels: u8,
    audio_format: &str,
    codec: Option<&str>,
    bpm: Option<f32>,
    replaygain_track_gain: Option<f32>,
    replaygain_track_peak: Option<f32>,
    replaygain_album_gain: Option<f32>,
    replaygain_album_peak: Option<f32>,
    encoder: Option<&str>,
) -> Result<i64> {
    conn.prepare_cached(
        "INSERT INTO track (path, title, duration_sec, year, mtime, file_size, cover_art,
            genre, bitrate, sample_rate, bit_depth, channels, audio_format, codec,
            bpm, replaygain_track_gain, replaygain_track_peak,
            replaygain_album_gain, replaygain_album_peak, encoder)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7,
             ?8, ?9, ?10, ?11, ?12, ?13, ?14,
             ?15, ?16, ?17, ?18, ?19, ?20)
            ON CONFLICT(path) DO UPDATE SET
            title = excluded.title,
            duration_sec = excluded.duration_sec,
            year = excluded.year,
            mtime = excluded.mtime,
            file_size = excluded.file_size,
            cover_art = excluded.cover_art,
            genre = excluded.genre,
            bitrate = excluded.bitrate,
            sample_rate = excluded.sample_rate,
            bit_depth = excluded.bit_depth,
            channels = excluded.channels,
            audio_format = excluded.audio_format,
            codec = excluded.codec,
            bpm = excluded.bpm,
            replaygain_track_gain = excluded.replaygain_track_gain,
            replaygain_track_peak = excluded.replaygain_track_peak,
            replaygain_album_gain = excluded.replaygain_album_gain,
            replaygain_album_peak = excluded.replaygain_album_peak,
            encoder = excluded.encoder
            RETURNING id",
    )
    .map_err(Error::Db)?
    .query_row(
        params![
            path,
            title,
            duration_sec,
            year,
            mtime,
            file_size,
            cover_art,
            genre,
            bitrate,
            sample_rate,
            bit_depth,
            channels,
            audio_format,
            codec,
            bpm,
            replaygain_track_gain,
            replaygain_track_peak,
            replaygain_album_gain,
            replaygain_album_peak,
            encoder
        ],
        |row| row.get(0),
    )
    .map_err(Error::Db)
}

pub fn set_track_album(
    conn: &Connection,
    track_id: i64,
    album_id: i64,
    track_number: i32,
) -> Result<()> {
    let unchanged = conn
        .prepare_cached(
            "SELECT 1 FROM album_track WHERE track_id = ? AND album_id = ? AND track_number = ?",
        )
        .map_err(Error::Db)?
        .query_row(params![track_id, album_id, track_number], |_| Ok(()))
        .optional()
        .map_err(Error::Db)?
        .is_some();

    if unchanged {
        return Ok(());
    }

    conn.prepare_cached("DELETE FROM album_track WHERE track_id = ?")
        .map_err(Error::Db)?
        .execute(params![track_id])
        .map_err(Error::Db)?;

    conn.prepare_cached(
        "INSERT OR IGNORE INTO album_track (album_id, track_id, track_number) VALUES (?, ?, ?)",
    )
    .map_err(Error::Db)?
    .execute(params![album_id, track_id, track_number])
    .map_err(Error::Db)?;
    Ok(())
}

pub fn clear_track_artists(conn: &Connection, track_id: i64) -> Result<()> {
    conn.prepare_cached("DELETE FROM track_artist WHERE track_id = ?")
        .map_err(Error::Db)?
        .execute(params![track_id])
        .map_err(Error::Db)?;
    Ok(())
}

pub fn clear_track_album(conn: &Connection, track_id: i64) -> Result<()> {
    conn.prepare_cached("DELETE FROM album_track WHERE track_id = ?")
        .map_err(Error::Db)?
        .execute(params![track_id])
        .map_err(Error::Db)?;
    Ok(())
}

pub fn bulk_insert_track_artists(conn: &Connection, pairs: &[(i64, i64)]) -> Result<()> {
    if pairs.is_empty() {
        return Ok(());
    }
    for chunk in pairs.chunks(400) {
        let placeholders: Vec<String> = chunk.iter().map(|_| "(?, ?)".to_string()).collect();
        let sql = format!(
            "INSERT OR IGNORE INTO track_artist (track_id, artist_id) VALUES {}",
            placeholders.join(", ")
        );
        let mut param_values: Vec<Box<dyn ToSql>> = Vec::with_capacity(chunk.len() * 2);
        for &(tid, aid) in chunk {
            param_values.push(Box::new(tid));
            param_values.push(Box::new(aid));
        }
        let params_refs: Vec<&dyn ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        conn.prepare_cached(&sql)
            .map_err(Error::Db)?
            .execute(params_refs.as_slice())
            .map_err(Error::Db)?;
    }
    Ok(())
}

pub fn clear_track_genres(conn: &Connection, track_id: i64) -> Result<()> {
    conn.prepare_cached("DELETE FROM track_genre WHERE track_id = ?")
        .map_err(Error::Db)?
        .execute(params![track_id])
        .map_err(Error::Db)?;
    Ok(())
}

pub fn get_or_create_genre(conn: &Connection, name: &str) -> Result<i64> {
    conn.prepare_cached(
        "INSERT INTO genre (name) VALUES (?1) ON CONFLICT(name) DO UPDATE SET name=?1 RETURNING id",
    )
    .map_err(Error::Db)?
    .query_row(params![name], |row| row.get(0))
    .map_err(Error::Db)
}

pub fn bulk_insert_track_genres(conn: &Connection, pairs: &[(i64, i64)]) -> Result<()> {
    if pairs.is_empty() {
        return Ok(());
    }
    for chunk in pairs.chunks(400) {
        let placeholders: Vec<String> = chunk.iter().map(|_| "(?, ?)".to_string()).collect();
        let sql = format!(
            "INSERT OR IGNORE INTO track_genre (track_id, genre_id) VALUES {}",
            placeholders.join(", ")
        );
        let mut param_values: Vec<Box<dyn ToSql>> = Vec::with_capacity(chunk.len() * 2);
        for &(tid, gid) in chunk {
            param_values.push(Box::new(tid));
            param_values.push(Box::new(gid));
        }
        let params_refs: Vec<&dyn ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        conn.prepare_cached(&sql)
            .map_err(Error::Db)?
            .execute(params_refs.as_slice())
            .map_err(Error::Db)?;
    }
    Ok(())
}

pub fn get_genres_for_tracks(
    conn: &Connection,
    track_ids: &[i64],
) -> Result<HashMap<i64, Vec<Genre>>> {
    let mut map = HashMap::new();
    if track_ids.is_empty() {
        return Ok(map);
    }

    for chunk in track_ids.chunks(900) {
        let placeholders = sql_placeholders(chunk.len());
        let sql = format!(
            "SELECT tg.track_id, g.id, g.name, g.thumbnail
             FROM track_genre tg
             JOIN genre g ON tg.genre_id = g.id
             WHERE tg.track_id IN ({})",
            placeholders
        );
        let mut stmt = conn.prepare(&sql).map_err(Error::Db)?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(chunk), |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    Genre {
                        id: row.get(1)?,
                        name: row.get(2)?,
                        thumbnail: row.get(3)?,
                        ..Default::default()
                    },
                ))
            })
            .map_err(Error::Db)?;

        for row in rows {
            let (track_id, genre) = row.map_err(Error::Db)?;
            map.entry(track_id).or_insert_with(Vec::new).push(genre);
        }
    }

    Ok(map)
}

pub fn set_track_lyrics(
    conn: &Connection,
    track_id: i64,
    plain_lyrics: Option<&str>,
    synced_lyrics: Option<&str>,
    source: &str,
) -> Result<()> {
    if plain_lyrics.is_none() && synced_lyrics.is_none() {
        return Ok(());
    }
    conn.prepare_cached(
        "INSERT INTO track_lyrics (track_id, plain_lyrics, synced_lyrics, source)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(track_id) DO UPDATE SET
           plain_lyrics = excluded.plain_lyrics,
           synced_lyrics = excluded.synced_lyrics,
           source = excluded.source",
    )
    .map_err(Error::Db)?
    .execute(params![track_id, plain_lyrics, synced_lyrics, source])
    .map_err(Error::Db)?;
    Ok(())
}

pub fn get_track_lyrics(conn: &Connection, track_id: i64) -> Result<Option<Lyrics>> {
    let result = conn
        .query_row(
            "SELECT plain_lyrics, synced_lyrics, source FROM track_lyrics WHERE track_id = ?",
            params![track_id],
            |row| {
                Ok(Lyrics {
                    plain_lyrics: row.get(0)?,
                    synced_lyrics: row.get(1)?,
                    source: row.get(2)?,
                })
            },
        )
        .optional()
        .map_err(Error::Db)?;
    Ok(result)
}

pub fn report_lyrics_fetch_success(conn: &Connection, track_id: i64) -> Result<()> {
    conn.prepare_cached(
        "INSERT INTO track_lyrics (track_id, plain_lyrics, synced_lyrics, source, fetch_attempts, last_fetch_attempt)
         VALUES (?1, NULL, NULL, 'lrclib', 0, datetime('now'))
         ON CONFLICT(track_id) DO UPDATE SET
           fetch_attempts = 0,
           last_fetch_attempt = datetime('now')",
    )
    .map_err(Error::Db)?
    .execute(params![track_id])
    .map_err(Error::Db)?;
    Ok(())
}

pub fn report_lyrics_fetch_failure(conn: &Connection, track_id: i64) -> Result<()> {
    conn.prepare_cached(
        "INSERT INTO track_lyrics (track_id, plain_lyrics, synced_lyrics, source, fetch_attempts, last_fetch_attempt)
         VALUES (?1, NULL, NULL, 'lrclib', 1, datetime('now'))
         ON CONFLICT(track_id) DO UPDATE SET
           fetch_attempts = fetch_attempts + 1,
           last_fetch_attempt = datetime('now')",
    )
    .map_err(Error::Db)?
    .execute(params![track_id])
    .map_err(Error::Db)?;
    Ok(())
}

pub fn set_track_genre(conn: &Connection, track_id: i64, genre_name: &str) -> Result<()> {
    let genre_id = get_or_create_genre(conn, genre_name)?;
    clear_track_genres(conn, track_id)?;
    bulk_insert_track_genres(conn, &[(track_id, genre_id)])?;
    Ok(())
}

pub fn get_tracks_by_genre(conn: &Connection, genre_id: i64) -> Result<Vec<Track>> {
    let sql =
        "SELECT t.id, t.title, al.id, al.name, al.cover_art, alt.track_number, al.album_artist, al.year, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        JOIN track_genre tg ON t.id = tg.track_id
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        WHERE tg.genre_id = ?
        ORDER BY t.title COLLATE NOCASE ASC";

    prepare_tracks_list(conn, sql, params![genre_id])
}

pub fn update_genre(
    conn: &Connection,
    id: i64,
    name: &str,
    thumbnail: Option<&str>,
) -> Result<Genre> {
    conn.prepare_cached(
        "UPDATE genre SET name = ?1, thumbnail = ?2 WHERE id = ?3 RETURNING id, name, thumbnail",
    )
    .map_err(Error::Db)?
    .query_row(params![name, thumbnail, id], |row| {
        Ok(Genre {
            id: row.get(0)?,
            name: row.get(1)?,
            thumbnail: row.get(2)?,
            ..Default::default()
        })
    })
    .map_err(Error::Db)
}

pub fn get_all_genres(conn: &Connection) -> Result<Vec<Genre>> {
    let sql = "SELECT g.id, g.name, g.thumbnail,
            COUNT(tg.track_id) AS track_count,
            IFNULL(SUM(s.play_count), 0) AS total_plays,
            MAX(s.last_played_at) AS last_played_at,
            MIN(t.added_at) AS added_at
        FROM genre g
        LEFT JOIN track_genre tg ON tg.genre_id = g.id
        LEFT JOIN track t ON t.id = tg.track_id
        LEFT JOIN track_stats s ON s.track_id = tg.track_id
        GROUP BY g.id
        ORDER BY g.name COLLATE NOCASE ASC";
    let mut stmt = conn.prepare(sql).map_err(Error::Db)?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Genre {
                id: row.get(0)?,
                name: row.get(1)?,
                thumbnail: row.get(2)?,
                track_count: row.get(3)?,
                total_plays: row.get(4)?,
                last_played_at: row.get(5)?,
                added_at: row.get(6)?,
                ..Default::default()
            })
        })
        .map_err(Error::Db)?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)
}

pub fn delete_genre(conn: &Connection, genre_id: i64) -> Result<()> {
    let tx = conn.unchecked_transaction().map_err(Error::Db)?;
    tx.execute(
        "DELETE FROM track_genre WHERE genre_id = ?",
        params![genre_id],
    )
    .map_err(Error::Db)?;
    tx.execute("DELETE FROM genre WHERE id = ?", params![genre_id])
        .map_err(Error::Db)?;
    tx.commit().map_err(Error::Db)?;
    Ok(())
}

/// Track number of a track within its album (album_track.track_number).
pub fn get_track_number(conn: &Connection, track_id: i64) -> Result<Option<i32>> {
    conn.query_row(
        "SELECT track_number FROM album_track WHERE track_id = ?",
        params![track_id],
        |row| row.get(0),
    )
    .optional()
    .map_err(Error::Db)
}

pub fn set_track_cover_art(
    conn: &Connection,
    track_id: i64,
    cover_art: Option<&str>,
) -> Result<()> {
    conn.prepare_cached("UPDATE track SET cover_art = ?1 WHERE id = ?2")
        .map_err(Error::Db)?
        .execute(params![cover_art, track_id])
        .map_err(Error::Db)?;
    Ok(())
}

pub fn bulk_insert_track_albums(conn: &Connection, entries: &[(i64, i64, i32)]) -> Result<()> {
    if entries.is_empty() {
        return Ok(());
    }
    for chunk in entries.chunks(250) {
        let placeholders: Vec<String> = chunk.iter().map(|_| "(?, ?, ?)".to_string()).collect();
        let sql = format!(
            "INSERT OR IGNORE INTO album_track (album_id, track_id, track_number) VALUES {}",
            placeholders.join(", ")
        );
        let mut param_values: Vec<Box<dyn ToSql>> = Vec::with_capacity(chunk.len() * 3);
        for &(aid, tid, tn) in chunk {
            param_values.push(Box::new(aid));
            param_values.push(Box::new(tid));
            param_values.push(Box::new(tn));
        }
        let params_refs: Vec<&dyn ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        conn.prepare_cached(&sql)
            .map_err(Error::Db)?
            .execute(params_refs.as_slice())
            .map_err(Error::Db)?;
    }
    Ok(())
}

pub fn get_track_id_by_path(conn: &Connection, path: &str) -> Result<i64> {
    conn.query_row(
        "SELECT id FROM track WHERE path = ?",
        params![path],
        |row| row.get(0),
    )
    .map_err(Error::Db)
}

pub fn get_track_by_id(conn: &Connection, id: i64) -> Result<Track> {
    let sql =
        "SELECT t.id, t.title, al.id, al.name, al.cover_art, alt.track_number, al.album_artist, al.year, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        WHERE t.id = ?";
    let tracks = prepare_tracks_list(conn, sql, params![id])?;
    tracks
        .into_iter()
        .next()
        .ok_or_else(|| Error::Db(rusqlite::Error::QueryReturnedNoRows))
}

pub fn get_track_path_by_id(conn: &Connection, id: i64) -> Result<String> {
    conn.query_row("SELECT path FROM track WHERE id = ?", params![id], |row| {
        row.get(0)
    })
    .map_err(Error::Db)
}

pub fn toggle_favorite(conn: &Connection, track_id: i64) -> Result<Track> {
    conn.execute(
        "UPDATE track SET is_favorite = NOT is_favorite WHERE id = ?",
        params![track_id],
    )
    .map_err(Error::Db)?;
    get_track_by_id(conn, track_id)
}

pub fn get_all_albums(conn: &Connection) -> Result<Vec<Album>> {
    let sql = "SELECT al.id, al.name, al.cover_art, al.album_artist, al.year,
            COUNT(DISTINCT alt.track_id) AS track_count,
            IFNULL(SUM(s.play_count), 0) AS total_plays,
            MAX(s.last_played_at) AS last_played_at,
            MIN(t.added_at) AS added_at
        FROM album al
        LEFT JOIN album_track alt ON alt.album_id = al.id
        LEFT JOIN track t ON t.id = alt.track_id
        LEFT JOIN track_stats s ON s.track_id = alt.track_id
        GROUP BY al.id
        ORDER BY al.name COLLATE NOCASE ASC";

    let mut stmt = conn.prepare(sql).map_err(Error::Db)?;
    let raw: Vec<(Album, Option<String>)> = stmt
        .query_map([], |row| {
            let album = Album {
                id: row.get(0)?,
                name: row.get(1)?,
                cover_art: row.get(2)?,
                album_artist: None,
                year: row.get::<_, Option<i32>>(4)?.map(|y| y as u32),
                track_count: row.get(5)?,
                total_plays: row.get(6)?,
                last_played_at: row.get(7)?,
                added_at: row.get(8)?,
                ..Default::default()
            };
            let aa: Option<String> = row.get(3)?;
            Ok((album, aa))
        })
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?;

    let mut albums: Vec<Album> = Vec::with_capacity(raw.len());
    for (mut album, aa) in raw {
        album.album_artist = resolve_album_artist(conn, aa)?;
        albums.push(album);
    }
    Ok(albums)
}

pub fn get_all_playlists(conn: &Connection) -> Result<Vec<Playlist>> {
    let sql = "SELECT p.id, p.name, p.cover_art,
            COUNT(pt.track_id) AS track_count,
            IFNULL(SUM(s.play_count), 0) AS total_plays,
            MAX(s.last_played_at) AS last_played_at,
            MIN(t.added_at) AS added_at
        FROM playlist p
        LEFT JOIN playlist_track pt ON pt.playlist_id = p.id
        LEFT JOIN track t ON t.id = pt.track_id
        LEFT JOIN track_stats s ON s.track_id = pt.track_id
        GROUP BY p.id";

    let mut stmt = conn.prepare(sql).map_err(Error::Db)?;
    let playlist_iter = stmt
        .query_map([], |row| {
            Ok(Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
                cover_art: row.get(2)?,
                track_count: row.get(3)?,
                total_plays: row.get(4)?,
                last_played_at: row.get(5)?,
                added_at: row.get(6)?,
                ..Default::default()
            })
        })
        .map_err(Error::Db)?;

    playlist_iter
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)
}

pub fn get_all_artists(conn: &Connection) -> Result<Vec<Artist>> {
    let sql = "SELECT ar.id, ar.name, ar.profile_image, ar.banner_image,
            COUNT(DISTINCT ta.track_id) AS track_count,
            IFNULL(SUM(s.play_count), 0) AS total_plays,
            MAX(s.last_played_at) AS last_played_at,
            MIN(t.added_at) AS added_at
        FROM artist ar
        LEFT JOIN track_artist ta ON ta.artist_id = ar.id
        LEFT JOIN track t ON t.id = ta.track_id
        LEFT JOIN track_stats s ON s.track_id = ta.track_id
        GROUP BY ar.id";

    let mut stmt = conn.prepare(sql).map_err(Error::Db)?;
    let artist_iter = stmt
        .query_map([], |row| {
            Ok(Artist {
                id: row.get(0)?,
                name: row.get(1)?,
                profile_image: row.get(2)?,
                banner_image: row.get(3)?,
                track_count: row.get(4)?,
                total_plays: row.get(5)?,
                last_played_at: row.get(6)?,
                added_at: row.get(7)?,
                ..Default::default()
            })
        })
        .map_err(Error::Db)?;

    artist_iter
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)
}

pub fn create_playlist(conn: &Connection, name: &str) -> Result<Playlist> {
    conn.execute("INSERT INTO playlist (name) VALUES (?)", params![name])
        .map_err(Error::Db)?;
    let id = conn.last_insert_rowid();
    get_playlist(conn, id)
}

pub fn delete_playlist(conn: &Connection, playlist_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM playlist_track WHERE playlist_id = ?",
        params![playlist_id],
    )
    .map_err(Error::Db)?;

    conn.execute("DELETE FROM playlist WHERE id = ?", params![playlist_id])
        .map_err(Error::Db)?;
    Ok(())
}

pub fn add_track_to_playlist(conn: &Connection, playlist_id: i64, track_id: i64) -> Result<()> {
    let position: i32 = conn
        .query_row(
            "SELECT IFNULL(MAX(position), -1) + 1 FROM playlist_track WHERE playlist_id = ?",
            params![playlist_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    conn.execute(
        "INSERT OR IGNORE INTO playlist_track (playlist_id, track_id, position) VALUES (?, ?, ?)",
        params![playlist_id, track_id, position],
    )
    .map_err(Error::Db)?;
    Ok(())
}

pub fn update_playlist(conn: &Connection, playlist: Playlist) -> Result<Playlist> {
    conn.execute(
        "UPDATE playlist SET name = ?, cover_art = ? WHERE id = ?",
        params![playlist.name, playlist.cover_art, playlist.id],
    )
    .map_err(Error::Db)?;
    get_playlist(conn, playlist.id)
}

pub fn update_artist(conn: &Connection, artist: Artist) -> Result<Artist> {
    conn.execute(
        "UPDATE artist SET name = ?, profile_image = ?, banner_image = ? WHERE id = ?",
        params![
            artist.name,
            artist.profile_image,
            artist.banner_image,
            artist.id
        ],
    )
    .map_err(Error::Db)?;
    get_artist(conn, artist.id)
}

pub fn update_album(
    conn: &Connection,
    id: i64,
    name: Option<&str>,
    cover_art: Option<&str>,
) -> Result<Album> {
    if let Some(name) = name {
        conn.execute("UPDATE album SET name = ? WHERE id = ?", params![name, id])
            .map_err(Error::Db)?;
    }
    if let Some(cover_art) = cover_art {
        conn.execute(
            "UPDATE album SET cover_art = ? WHERE id = ?",
            params![cover_art, id],
        )
        .map_err(Error::Db)?;
    }
    get_album(conn, id)
}

pub fn update_track_partial(
    conn: &Connection,
    id: i64,
    title: String,
    year: Option<i32>,
    genre: Option<String>,
) -> Result<TrackDetails> {
    conn.execute(
        "UPDATE track SET title = ?, year = ?, genre = COALESCE(?, genre) WHERE id = ?",
        params![title, year, genre, id],
    )
    .map_err(Error::Db)?;
    get_track_details(conn, id)
}

pub fn get_tracks_by_album(conn: &Connection, album_id: i64) -> Result<Vec<Track>> {
    let sql =
        "SELECT t.id, t.title, al.id, al.name, al.cover_art, alt.track_number, al.album_artist, al.year, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        JOIN album_track alt ON t.id = alt.track_id
        JOIN album al ON alt.album_id = al.id
        WHERE alt.album_id = ?
        ORDER BY alt.track_number ASC";

    prepare_tracks_list(conn, sql, params![album_id])
}

pub fn get_tracks_by_artist(conn: &Connection, artist_id: i64) -> Result<Vec<Track>> {
    let sql =
        "SELECT t.id, t.title, al.id, al.name, al.cover_art, alt.track_number, al.album_artist, al.year, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        JOIN track_artist ta ON t.id = ta.track_id
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        WHERE ta.artist_id = ?
        ORDER BY al.name COLLATE NOCASE ASC, alt.track_number ASC";

    prepare_tracks_list(conn, sql, params![artist_id])
}

pub fn get_tracks_in_playlist(
    conn: &Connection,
    playlist_id: i64,
    sort_by: Option<SortBy>,
) -> Result<Vec<Track>> {
    let sql =
            "SELECT t.id, t.title, al.id, al.name, al.cover_art, alt.track_number, al.album_artist, al.year, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        JOIN playlist_track pt ON t.id = pt.track_id
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        WHERE pt.playlist_id = ?";

    if let Some(sort_by) = sort_by {
        return prepare_sorted_tracks_list(conn, sql, params![playlist_id], Some(sort_by));
    }

    // Default order is playlist insertion order (position is MAX+1 on insert),
    // so the most recently added track is last.
    let ordered = format!("{} ORDER BY pt.position ASC", sql);
    prepare_tracks_list(conn, &ordered, params![playlist_id])
}

pub fn get_playlist(conn: &Connection, playlist_id: i64) -> Result<Playlist> {
    conn.query_row(
        "SELECT id, name, cover_art FROM playlist WHERE id = ?",
        params![playlist_id],
        |row| {
            Ok(Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
                cover_art: row.get(2)?,
                ..Default::default()
            })
        },
    )
    .map_err(Error::Db)
}

pub fn get_artist(conn: &Connection, artist_id: i64) -> Result<Artist> {
    conn.query_row(
        "SELECT id, name, profile_image, banner_image FROM artist WHERE id = ?",
        params![artist_id],
        |row| {
            Ok(Artist {
                id: row.get(0)?,
                name: row.get(1)?,
                profile_image: row.get(2)?,
                banner_image: row.get(3)?,
                ..Default::default()
            })
        },
    )
    .map_err(Error::Db)
}

pub fn get_album(conn: &Connection, album_id: i64) -> Result<Album> {
    let (mut album, aa): (Album, Option<String>) = conn
        .query_row(
            "SELECT id, name, cover_art, album_artist, year FROM album WHERE id = ?",
            params![album_id],
            |row| {
                let album = Album {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    cover_art: row.get(2)?,
                    album_artist: None,
                    year: row.get::<_, Option<i32>>(4)?.map(|y| y as u32),
                    ..Default::default()
                };
                Ok((album, row.get::<_, Option<String>>(3)?))
            },
        )
        .map_err(Error::Db)?;
    album.album_artist = resolve_album_artist(conn, aa)?;
    Ok(album)
}

pub fn get_favorite_tracks(conn: &Connection) -> Result<Vec<Track>> {
    let sql =
        "SELECT t.id, t.title, al.id, al.name, al.cover_art, alt.track_number, al.album_artist, al.year, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        WHERE t.is_favorite = 1
        ORDER BY t.added_at DESC";

    prepare_tracks_list(conn, sql, [])
}

pub fn get_track_details(conn: &Connection, track_id: i64) -> Result<TrackDetails> {
    let sql = "SELECT t.id, t.path, t.title, al.id, al.name, al.cover_art, alt.track_number, al.album_artist, al.year,
        t.duration_sec, t.is_favorite, t.mtime,
        IFNULL(s.play_count, 0), s.last_played_at, IFNULL(s.skip_count, 0), s.last_skipped_at, t.cover_art, t.added_at,
        t.genre, t.bitrate, t.sample_rate, t.bit_depth, t.channels, t.audio_format, t.codec,
        t.bpm, t.replaygain_track_gain, t.replaygain_track_peak,
        t.replaygain_album_gain, t.replaygain_album_peak, t.encoder
        FROM track t
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        LEFT JOIN track_stats s ON t.id = s.track_id
        WHERE t.id = ?";

    let (result, album_artist_name): (TrackDetails, Option<String>) = conn
        .query_row(sql, params![track_id], |row| {
            let album_id: Option<i64> = row.get(3)?;
            let album_title: Option<String> = row.get(4)?;
            let album_art: Option<String> = row.get(5)?;
            let album = Album {
                id: album_id.unwrap_or(0),
                name: album_title.unwrap_or_else(|| "Unknown Album".to_string()),
                cover_art: album_art,
                album_artist: None,
                year: row.get::<_, Option<i32>>(8)?.map(|y| y as u32),
                ..Default::default()
            };

            Ok((
                TrackDetails {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    title: row.get(2)?,
                    artists: vec![],
                    album,
                    duration_seconds: row.get(9)?,
                    is_favorite: row.get(10)?,
                    mtime: row.get(11)?,
                    play_count: row.get(12)?,
                    last_played_at: row.get(13)?,
                    skipped_count: row.get(14)?,
                    last_skipped_at: row.get(15)?,
                    cover_art: row.get(16)?,
                    added_at: row.get(17)?,
                    track_number: row.get::<_, Option<i32>>(6)?.map(|n| n as u32),
                    year: row.get::<_, Option<i32>>(8)?.map(|y| y as u32),
                    playlist_ids: vec![],
                    genres: vec![],
                    bitrate: row.get(19)?,
                    sample_rate: row.get::<_, u32>(20)?,
                    bit_depth: row.get(21)?,
                    channels: row.get::<_, u8>(22)?,
                    audio_format: row.get(23)?,
                    codec: row.get(24)?,
                    bpm: row.get(25)?,
                    replaygain_track_gain: row.get(26)?,
                    replaygain_track_peak: row.get(27)?,
                    replaygain_album_gain: row.get(28)?,
                    replaygain_album_peak: row.get(29)?,
                    encoder: row.get(30)?,
                    lyrics: None,
                },
                row.get::<_, Option<String>>(7)?,
            ))
        })
        .map_err(Error::Db)?;

    let mut details = result;
    if let Some(aa) = album_artist_name {
        details.album.album_artist = resolve_album_artist(conn, Some(aa))?;
    }

    let artists_map = get_artists_for_tracks(conn, &[track_id])?;
    if let Some(artists) = artists_map.get(&track_id) {
        details.artists = artists.clone();
    }

    let playlist_ids_map = get_playlist_ids_for_tracks(conn, &[track_id])?;
    if let Some(ids) = playlist_ids_map.get(&track_id) {
        details.playlist_ids = ids.clone();
    }

    let genres_map = get_genres_for_tracks(conn, &[track_id])?;
    if let Some(genres) = genres_map.get(&track_id) {
        details.genres = genres.clone();
    }

    let lyrics = get_track_lyrics(conn, track_id)?;
    details.lyrics = lyrics;

    Ok(details)
}

pub fn get_similar_tracks(
    conn: &Connection,
    current_track_id: i64,
    limit: usize,
) -> Result<Vec<Track>> {
    let sql = r#"
    SELECT t.id, t.title, al.id, al.name, al.cover_art, alt.track_number, al.album_artist, al.year, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
    FROM track t
    JOIN track current ON current.id = ?
    LEFT JOIN album_track alt ON t.id = alt.track_id
    LEFT JOIN album al ON al.id = alt.album_id
    LEFT JOIN track_stats s ON s.track_id = t.id
    WHERE t.id != current.id
    GROUP BY t.id
    ORDER BY (
            (CASE WHEN EXISTS (
                SELECT 1 FROM track_artist ta1 JOIN track_artist ta2 ON ta1.artist_id = ta2.artist_id
                WHERE ta1.track_id = t.id AND ta2.track_id = current.id
            ) THEN 50 ELSE 0 END) +
            (CASE WHEN EXISTS (
                SELECT 1 FROM album_track alt1 JOIN album_track alt2 ON alt1.album_id = alt2.album_id
                WHERE alt1.track_id = t.id AND alt2.track_id = current.id
            ) THEN 20 ELSE 0 END) +
            (CASE WHEN t.is_favorite = 1 THEN 25 ELSE 0 END) +

            IFNULL(s.play_count, 0) * 2 -
            IFNULL(s.skip_count, 0) * 5 +

            -- If skipped in the last 24 hours, drop score by 150 points
            (CASE
                WHEN s.last_skipped_at IS NOT NULL
                     AND (strftime('%s', 'now') - strftime('%s', s.last_skipped_at)) < 86400
                THEN -150
                ELSE 0
            END) -

            -- 2-Hour Cool Down so that recently played track don't dominate the list
            (CASE
                WHEN s.last_played_at IS NOT NULL
                     AND (strftime('%s', 'now') - strftime('%s', s.last_played_at)) < 7200
                THEN -100
                ELSE 0
            END) +

            -- Randomness factor to add some variety
            (ABS(RANDOM() % 11))
        ) DESC
    LIMIT ?
    "#;
    prepare_tracks_list(conn, sql, params![current_track_id, limit as i64])
}

pub fn get_all_tracks(conn: &Connection, sort_by: Option<SortBy>) -> Result<Vec<Track>> {
    let sql =
        "SELECT t.id, t.title, al.id, al.name, al.cover_art, alt.track_number, al.album_artist, al.year, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id";

    prepare_sorted_tracks_list(conn, sql, [], sort_by)
}

pub fn get_track_playlist_ids(conn: &Connection, track_id: i64) -> Result<Vec<i64>> {
    let mut stmt = conn
        .prepare("SELECT playlist_id FROM playlist_track WHERE track_id = ?")
        .map_err(Error::Db)?;

    let rows = stmt
        .query_map(params![track_id], |row| row.get(0))
        .map_err(Error::Db)?;

    let mut ids = Vec::new();
    for row in rows {
        ids.push(row.map_err(Error::Db)?);
    }
    Ok(ids)
}

pub fn remove_track_from_playlist(
    conn: &Connection,
    playlist_id: i64,
    track_id: i64,
) -> Result<()> {
    conn.execute(
        "DELETE FROM playlist_track WHERE playlist_id = ? AND track_id = ?",
        params![playlist_id, track_id],
    )
    .map_err(Error::Db)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Search & name-resolution helpers (used by CLI)
// ---------------------------------------------------------------------------

pub fn search_tracks(conn: &Connection, query: &str, limit: usize) -> Result<Vec<Track>> {
    let sql =
        "SELECT t.id, t.title, al.id, al.name, al.cover_art, alt.track_number, al.album_artist, al.year, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        WHERE t.title LIKE ?1
           OR al.name LIKE ?1
           OR EXISTS (SELECT 1 FROM track_artist ta JOIN artist ar ON ta.artist_id = ar.id WHERE ta.track_id = t.id AND ar.name LIKE ?1)
        ORDER BY
            CASE WHEN t.title LIKE ?2 THEN 0 ELSE 1 END,
            t.title COLLATE NOCASE ASC
        LIMIT ?3";
    let pattern = format!("%{query}%");
    let exact_start = format!("{query}%");
    prepare_tracks_list(conn, sql, params![pattern, exact_start, limit as i64])
}

pub fn search_artists(conn: &Connection, query: &str, limit: usize) -> Result<Vec<Artist>> {
    let pattern = format!("%{query}%");
    let mut stmt = conn
        .prepare(
            "SELECT id, name, profile_image, banner_image
             FROM artist
             WHERE name LIKE ?1
             ORDER BY
                CASE WHEN name LIKE ?2 THEN 0 ELSE 1 END,
                name COLLATE NOCASE ASC
             LIMIT ?3",
        )
        .map_err(Error::Db)?;
    let exact_start = format!("{query}%");
    let rows = stmt
        .query_map(params![pattern, exact_start, limit as i64], |row| {
            Ok(Artist {
                id: row.get(0)?,
                name: row.get(1)?,
                profile_image: row.get(2)?,
                banner_image: row.get(3)?,
                ..Default::default()
            })
        })
        .map_err(Error::Db)?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)
}

pub fn search_albums(conn: &Connection, query: &str, limit: usize) -> Result<Vec<Album>> {
    let pattern = format!("%{query}%");
    let exact_start = format!("{query}%");
    let mut stmt = conn
        .prepare(
            "SELECT id, name, cover_art, album_artist, year
             FROM album
             WHERE name LIKE ?1
             ORDER BY
                CASE WHEN name LIKE ?2 THEN 0 ELSE 1 END,
                name COLLATE NOCASE ASC
             LIMIT ?3",
        )
        .map_err(Error::Db)?;
    let raw: Vec<(Album, Option<String>)> = stmt
        .query_map(params![pattern, exact_start, limit as i64], |row| {
            let album = Album {
                id: row.get(0)?,
                name: row.get(1)?,
                cover_art: row.get(2)?,
                album_artist: None,
                year: row.get::<_, Option<i32>>(4)?.map(|y| y as u32),
                ..Default::default()
            };
            Ok((album, row.get::<_, Option<String>>(3)?))
        })
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?;

    let mut albums: Vec<Album> = Vec::with_capacity(raw.len());
    for (mut album, aa) in raw {
        album.album_artist = resolve_album_artist(conn, aa)?;
        albums.push(album);
    }
    Ok(albums)
}

pub fn get_playlist_by_name(conn: &Connection, name: &str) -> Result<Playlist> {
    conn.query_row(
        "SELECT id, name, cover_art FROM playlist WHERE name = ?",
        params![name],
        |row| {
            Ok(Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
                cover_art: row.get(2)?,
                ..Default::default()
            })
        },
    )
    .map_err(Error::Db)
}

pub fn get_artist_by_name(conn: &Connection, name: &str) -> Result<Artist> {
    conn.query_row(
        "SELECT id, name, profile_image, banner_image FROM artist WHERE name = ?",
        params![name],
        |row| {
            Ok(Artist {
                id: row.get(0)?,
                name: row.get(1)?,
                profile_image: row.get(2)?,
                banner_image: row.get(3)?,
                ..Default::default()
            })
        },
    )
    .map_err(Error::Db)
}

pub fn get_album_by_name(conn: &Connection, name: &str) -> Result<Album> {
    let (mut album, aa): (Album, Option<String>) = conn
        .query_row(
            "SELECT id, name, cover_art, album_artist, year FROM album WHERE name = ?",
            params![name],
            |row| {
                let album = Album {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    cover_art: row.get(2)?,
                    album_artist: None,
                    year: row.get::<_, Option<i32>>(4)?.map(|y| y as u32),
                    ..Default::default()
                };
                Ok((album, row.get::<_, Option<String>>(3)?))
            },
        )
        .map_err(Error::Db)?;
    album.album_artist = resolve_album_artist(conn, aa)?;
    Ok(album)
}

// Utils

fn sql_placeholders(count: usize) -> String {
    let mut s = String::with_capacity(count.saturating_sub(1) * 2 + 1);
    for i in 0..count {
        if i > 0 {
            s.push(',');
        }
        s.push('?');
    }
    s
}

fn resolve_album_artist(
    conn: &Connection,
    album_artist_name: Option<String>,
) -> Result<Option<Vec<Artist>>> {
    match album_artist_name {
        None => Ok(None),
        Some(name) => {
            let mut stmt = conn
                .prepare("SELECT id, name, profile_image, banner_image FROM artist WHERE name = ?")
                .map_err(Error::Db)?;
            let artists: Vec<Artist> = stmt
                .query_map(params![name], |row| {
                    Ok(Artist {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        profile_image: row.get(2)?,
                        banner_image: row.get(3)?,
                        ..Default::default()
                    })
                })
                .map_err(Error::Db)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(Error::Db)?;
            if artists.is_empty() {
                Ok(None)
            } else {
                Ok(Some(artists))
            }
        }
    }
}

fn batch_resolve_album_artists(
    conn: &Connection,
    names: &[&str],
) -> Result<HashMap<String, Vec<Artist>>> {
    let mut map = HashMap::new();
    if names.is_empty() {
        return Ok(map);
    }

    let mut unique_names: Vec<&str> = names.to_vec();
    unique_names.sort();
    unique_names.dedup();

    let placeholders: Vec<String> = unique_names.iter().map(|_| "?".to_string()).collect();
    let sql = format!(
        "SELECT ar.name, ar.id, ar.name, ar.profile_image, ar.banner_image
         FROM artist ar
         WHERE ar.name IN ({})",
        placeholders.join(",")
    );

    if let Ok(mut stmt) = conn.prepare(&sql) {
        if let Ok(rows) = stmt.query_map(rusqlite::params_from_iter(&unique_names), |row| {
            let name: String = row.get(0)?;
            Ok((
                name,
                Artist {
                    id: row.get(1)?,
                    name: row.get(2)?,
                    profile_image: row.get(3)?,
                    banner_image: row.get(4)?,
                    ..Default::default()
                },
            ))
        }) {
            for row in rows {
                if let Ok((name, artist)) = row {
                    map.entry(name).or_default().push(artist);
                }
            }
        }
    }

    Ok(map)
}

pub fn prepare_tracks_list<P: Params>(
    conn: &Connection,
    sql: &str,
    params: P,
) -> Result<Vec<Track>> {
    let mut stmt = conn.prepare(sql).map_err(Error::Db)?;

    struct RawTrack {
        track: Track,
        album_artist_name: Option<String>,
    }

    let rows = stmt
        .query_map(params, |row| {
            let album_id: Option<i64> = row.get(2)?;
            let album_title: Option<String> = row.get(3)?;
            let album_art: Option<String> = row.get(4)?;
            let album_artist_name: Option<String> = row.get(6)?;
            let album_year: Option<i32> = row.get(7)?;
            let album = Album {
                id: album_id.unwrap_or(0),
                name: album_title.unwrap_or_else(|| "Unknown Album".to_string()),
                cover_art: album_art,
                album_artist: None,
                year: album_year.map(|y| y as u32),
                ..Default::default()
            };

            Ok(RawTrack {
                track: Track {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    artists: vec![],
                    album,
                    duration_seconds: row.get(8)?,
                    is_favorite: row.get(9)?,
                    cover_art: row.get(10)?,
                    added_at: row.get(11)?,
                    track_number: row.get::<_, Option<i32>>(5)?.map(|n| n as u32),
                    playlist_ids: vec![],
                    genre_ids: None,
                    queue_id: None,
                },
                album_artist_name,
            })
        })
        .map_err(Error::Db)?;

    let mut raw_tracks: Vec<RawTrack> = rows
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?;

    if !raw_tracks.is_empty() {
        let names: Vec<&str> = raw_tracks
            .iter()
            .filter_map(|r| r.album_artist_name.as_deref())
            .collect();
        if !names.is_empty() {
            let artist_map = batch_resolve_album_artists(conn, &names)?;
            for rt in &mut raw_tracks {
                if let Some(ref name) = rt.album_artist_name {
                    if let Some(artists) = artist_map.get(name) {
                        rt.track.album.album_artist = Some(artists.clone());
                    }
                }
            }
        }

        let track_ids: Vec<i64> = raw_tracks.iter().map(|r| r.track.id).collect();
        let artists_map = get_artists_for_tracks(conn, &track_ids)?;
        for rt in &mut raw_tracks {
            if let Some(artists) = artists_map.get(&rt.track.id) {
                rt.track.artists = artists.clone();
            }
        }

        let playlist_ids_map = get_playlist_ids_for_tracks(conn, &track_ids)?;
        for rt in &mut raw_tracks {
            if let Some(ids) = playlist_ids_map.get(&rt.track.id) {
                rt.track.playlist_ids = ids.clone();
            }
        }

        let genre_ids_map = get_genre_ids_for_tracks(conn, &track_ids)?;
        for rt in &mut raw_tracks {
            if let Some(ids) = genre_ids_map.get(&rt.track.id) {
                rt.track.genre_ids = Some(ids.clone());
            }
        }
    }

    Ok(raw_tracks.into_iter().map(|r| r.track).collect())
}

fn get_artists_for_tracks(
    conn: &Connection,
    track_ids: &[i64],
) -> Result<HashMap<i64, Vec<Artist>>> {
    let mut map = HashMap::new();
    if track_ids.is_empty() {
        return Ok(map);
    }

    for chunk in track_ids.chunks(900) {
        let placeholders = sql_placeholders(chunk.len());
        let sql = format!(
            "SELECT ta.track_id, ar.id, ar.name, ar.profile_image, ar.banner_image
             FROM track_artist ta
             JOIN artist ar ON ta.artist_id = ar.id
             WHERE ta.track_id IN ({})",
            placeholders
        );
        let mut stmt = conn.prepare(&sql).map_err(Error::Db)?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(chunk), |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    Artist {
                        id: row.get(1)?,
                        name: row.get(2)?,
                        profile_image: row.get(3)?,
                        banner_image: row.get(4)?,
                        ..Default::default()
                    },
                ))
            })
            .map_err(Error::Db)?;

        for row in rows {
            let (track_id, artist) = row.map_err(Error::Db)?;
            map.entry(track_id).or_insert_with(Vec::new).push(artist);
        }
    }

    Ok(map)
}

fn get_playlist_ids_for_tracks(
    conn: &Connection,
    track_ids: &[i64],
) -> Result<HashMap<i64, Vec<i64>>> {
    let mut map = HashMap::new();
    if track_ids.is_empty() {
        return Ok(map);
    }

    for chunk in track_ids.chunks(900) {
        let placeholders = sql_placeholders(chunk.len());
        let sql = format!(
            "SELECT track_id, playlist_id FROM playlist_track WHERE track_id IN ({})",
            placeholders
        );
        let mut stmt = conn.prepare(&sql).map_err(Error::Db)?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(chunk), |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(Error::Db)?;

        for row in rows {
            let (track_id, playlist_id) = row.map_err(Error::Db)?;
            map.entry(track_id)
                .or_insert_with(Vec::new)
                .push(playlist_id);
        }
    }

    Ok(map)
}

fn get_genre_ids_for_tracks(
    conn: &Connection,
    track_ids: &[i64],
) -> Result<HashMap<i64, Vec<i64>>> {
    let mut map = HashMap::new();
    if track_ids.is_empty() {
        return Ok(map);
    }

    for chunk in track_ids.chunks(900) {
        let placeholders = sql_placeholders(chunk.len());
        let sql = format!(
            "SELECT track_id, genre_id FROM track_genre WHERE track_id IN ({})",
            placeholders
        );
        let mut stmt = conn.prepare(&sql).map_err(Error::Db)?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(chunk), |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(Error::Db)?;

        for row in rows {
            let (track_id, genre_id) = row.map_err(Error::Db)?;
            map.entry(track_id).or_insert_with(Vec::new).push(genre_id);
        }
    }

    Ok(map)
}

fn prepare_sorted_tracks_list<P: Params>(
    conn: &Connection,
    sql: &str,
    params: P,
    sort_by: Option<SortBy>,
) -> Result<Vec<Track>> {
    let sql = match sort_by {
        Some(SortBy::Title) => format!("{} ORDER BY t.title COLLATE NOCASE ASC", sql),
        Some(SortBy::Artist) => format!(
            "{} ORDER BY (SELECT MIN(ar2.name) FROM track_artist ta2 JOIN artist ar2 ON ta2.artist_id = ar2.id WHERE ta2.track_id = t.id) COLLATE NOCASE ASC",
            sql
        ),
        Some(SortBy::Album) => format!("{} ORDER BY al.name COLLATE NOCASE ASC", sql),
        Some(SortBy::Duration) => format!("{} ORDER BY t.duration_sec ASC", sql),
        Some(SortBy::RecentlyAdded) => format!("{} ORDER BY t.added_at DESC", sql),
        None => format!("{} ORDER BY t.added_at ASC", sql),
    };

    prepare_tracks_list(conn, &sql, params)
}

/////////////////////////////////////
// Stats queries
/////////////////////////////////////

pub fn get_recently_played_tracks(conn: &Connection, limit: usize) -> Result<Vec<Track>> {
    let sql =
        "SELECT t.id, t.title, al.id, al.name, al.cover_art, alt.track_number, al.album_artist, al.year, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        JOIN track_stats s ON t.id = s.track_id
        WHERE s.last_played_at IS NOT NULL
        ORDER BY s.last_played_at DESC
        LIMIT ?";

    prepare_tracks_list(conn, sql, params![limit as i64])
}

pub fn get_most_played_tracks(
    conn: &Connection,
    limit: usize,
    timeframe: Timeframe,
) -> Result<Vec<Track>> {
    let time_filter = timeframe_where_clause("s.last_played_at", timeframe);

    let sql = format!(
        "SELECT t.id, t.title, al.id, al.name, al.cover_art, alt.track_number, al.album_artist, al.year, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        JOIN track_stats s ON t.id = s.track_id
        WHERE s.play_count > 0 AND {}
        ORDER BY s.play_count DESC
        LIMIT ?",
        time_filter
    );

    prepare_tracks_list(conn, &sql, params![limit as i64])
}

pub fn get_top_artists(conn: &Connection, limit: usize) -> Result<Vec<Artist>> {
    let sql = "SELECT ar.id, ar.name, ar.profile_image, ar.banner_image, SUM(IFNULL(s.play_count, 0)) as total_plays
        FROM artist ar
        JOIN track_artist ta ON ta.artist_id = ar.id
        JOIN track t ON t.id = ta.track_id
        LEFT JOIN track_stats s ON t.id = s.track_id
        GROUP BY ar.id
        ORDER BY total_plays DESC
        LIMIT ?";

    let mut stmt = conn.prepare(&sql).map_err(Error::Db)?;
    let artist_iter = stmt
        .query_map(params![limit as i64], |row| {
            Ok(Artist {
                id: row.get(0)?,
                name: row.get(1)?,
                profile_image: row.get(2)?,
                banner_image: row.get(3)?,
                ..Default::default()
            })
        })
        .map_err(Error::Db)?;

    artist_iter
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)
}

pub fn get_top_albums(conn: &Connection, limit: usize) -> Result<Vec<Album>> {
    let sql = "SELECT al.id, al.name, al.cover_art, al.album_artist, al.year, SUM(IFNULL(s.play_count, 0)) as total_plays
        FROM album al
        JOIN album_track alt ON alt.album_id = al.id
        JOIN track t ON t.id = alt.track_id
        LEFT JOIN track_stats s ON t.id = s.track_id
        GROUP BY al.id
        ORDER BY total_plays DESC
        LIMIT ?";

    let mut stmt = conn.prepare(&sql).map_err(Error::Db)?;
    let raw: Vec<(Album, Option<String>)> = stmt
        .query_map(params![limit as i64], |row| {
            let album = Album {
                id: row.get(0)?,
                name: row.get(1)?,
                cover_art: row.get(2)?,
                album_artist: None,
                year: row.get::<_, Option<i32>>(4)?.map(|y| y as u32),
                ..Default::default()
            };
            Ok((album, row.get::<_, Option<String>>(3)?))
        })
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?;

    let mut albums: Vec<Album> = Vec::with_capacity(raw.len());
    for (mut album, aa) in raw {
        album.album_artist = resolve_album_artist(conn, aa)?;
        albums.push(album);
    }
    Ok(albums)
}

pub fn get_forgotten_tracks(conn: &Connection, limit: usize) -> Result<Vec<Track>> {
    let sql =
        "SELECT t.id, t.title, al.id, al.name, al.cover_art, alt.track_number, al.album_artist, al.year, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        JOIN track_stats s ON t.id = s.track_id
        WHERE s.play_count > 0 AND s.last_played_at <= datetime('now', '-180 days')
        ORDER BY s.last_played_at ASC
        LIMIT ?";

    prepare_tracks_list(conn, sql, params![limit as i64])
}

pub fn get_unplayed_tracks(conn: &Connection, limit: usize) -> Result<Vec<Track>> {
    let sql =
        "SELECT t.id, t.title, al.id, al.name, al.cover_art, alt.track_number, al.album_artist, al.year, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        LEFT JOIN track_stats s ON t.id = s.track_id
        WHERE s.play_count IS NULL OR s.play_count = 0
        ORDER BY t.added_at DESC
        LIMIT ?";

    prepare_tracks_list(conn, sql, params![limit as i64])
}

pub fn get_recently_added_tracks(conn: &Connection, limit: usize) -> Result<Vec<Track>> {
    let sql =
        "SELECT t.id, t.title, al.id, al.name, al.cover_art, alt.track_number, al.album_artist, al.year, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        ORDER BY t.added_at DESC
        LIMIT ?";

    prepare_tracks_list(conn, sql, params![limit as i64])
}

pub fn get_stats_overview(conn: &Connection, timeframe: Timeframe) -> Result<StatsOverview> {
    let time_filter = timeframe_where_clause("ph.played_at", timeframe);

    let (total_tracks, total_artists, total_albums, total_file_size, largest_file_size, played_tracks, unplayed): (i64, i64, i64, i64, i64, i64, i64) = conn
        .query_row(
            "SELECT
                (SELECT COUNT(*) FROM track),
                (SELECT COUNT(*) FROM artist),
                (SELECT COUNT(*) FROM album),
                (SELECT COALESCE(SUM(file_size), 0) FROM track),
                (SELECT COALESCE(MAX(file_size), 0) FROM track),
                (SELECT COUNT(DISTINCT ph.track_id) FROM playback_history ph),
                (SELECT COUNT(*) FROM track t LEFT JOIN track_stats s ON t.id = s.track_id WHERE s.play_count IS NULL OR s.play_count = 0)",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?)),
        )
        .map_err(Error::Db)?;

    let (total_plays, total_time, days_span, avg_completion, skip_rate): (i64, f64, f64, Option<f64>, Option<f64>) = conn
        .query_row(
            &format!(
                "SELECT
                    COALESCE(COUNT(ph.id), 0),
                    COALESCE(SUM(t.duration_sec * ph.completion_percent / 100.0), 0),
                    COALESCE(CAST(JULIANDAY('now', 'localtime') - JULIANDAY(MIN(ph.played_at), 'localtime') AS REAL), 1),
                    AVG(ph.completion_percent),
                    AVG(CASE WHEN ph.completion_percent < 50 THEN 1.0 ELSE 0.0 END)
                 FROM playback_history ph
                 JOIN track t ON t.id = ph.track_id
                 WHERE {}",
                time_filter
            ),
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .map_err(Error::Db)?;

    let total_time_sec = total_time as i64;
    // For rolling windows divide by the window length; AllTime uses the span
    // since the first recorded play so a long-ago first play doesn't flatten
    // recent activity.
    let days = match timeframe_window_days(timeframe) {
        Some(w) => w as f64,
        None => days_span.max(1.0),
    };
    let avg_daily = if total_time > 0.0 {
        total_time / 60.0 / days
    } else {
        0.0
    };
    let largest_file_mb = largest_file_size as f64 / 1048576.0;
    let avg_file_size_mb = if total_tracks > 0 {
        total_file_size as f64 / total_tracks as f64 / 1048576.0
    } else {
        0.0
    };
    let pct_library_played = if total_tracks > 0 {
        played_tracks as f64 / total_tracks as f64 * 100.0
    } else {
        0.0
    };

    let format_dist = get_format_distribution(conn)?;

    let (avg_bitrate_kbps, avg_sample_rate, avg_bit_depth): (
        Option<f64>,
        Option<f64>,
        Option<f64>,
    ) = conn
        .query_row(
            "SELECT
                AVG(CASE WHEN bitrate IS NOT NULL AND bitrate > 0 THEN bitrate / 1000.0 END),
                AVG(CASE WHEN sample_rate > 0 THEN sample_rate END),
                AVG(CASE WHEN bit_depth IS NOT NULL AND bit_depth > 0 THEN bit_depth END)
             FROM track",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(Error::Db)?;

    Ok(StatsOverview {
        total_tracks,
        total_artists,
        total_albums,
        total_plays,
        total_listening_time_sec: total_time_sec,
        avg_daily_listening_min: avg_daily,
        total_file_size_bytes: total_file_size,
        avg_file_size_mb,
        largest_file_mb,
        format_distribution: format_dist,
        pct_library_played,
        unplayed_tracks: unplayed,
        avg_bitrate_kbps,
        avg_sample_rate,
        avg_bit_depth,
        avg_completion_pct: avg_completion,
        skip_rate,
    })
}

pub fn get_format_distribution(conn: &Connection) -> Result<Vec<FormatStat>> {
    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM track", [], |row| row.get(0))
        .map_err(Error::Db)?;

    let mut stmt = conn
        .prepare(
            "SELECT
                LOWER(audio_format) as fmt,
                COUNT(*) as cnt,
                COALESCE(SUM(file_size), 0) as total_bytes,
                AVG(CASE WHEN bitrate IS NOT NULL AND bitrate > 0 THEN bitrate / 1000.0 END),
                AVG(CASE WHEN sample_rate > 0 THEN sample_rate END),
                AVG(CASE WHEN bit_depth IS NOT NULL AND bit_depth > 0 THEN bit_depth END)
             FROM track
             GROUP BY fmt
             ORDER BY cnt DESC",
        )
        .map_err(Error::Db)?;

    let stats = stmt
        .query_map([], |row| {
            Ok(FormatStat {
                format: row.get(0)?,
                count: row.get(1)?,
                percentage: if total > 0 {
                    row.get::<_, i64>(1)? as f64 / total as f64 * 100.0
                } else {
                    0.0
                },
                total_bytes: row.get(2)?,
                avg_bitrate_kbps: row.get(3)?,
                avg_sample_rate: row.get(4)?,
                avg_bit_depth: row.get(5)?,
            })
        })
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?;

    Ok(stats)
}

/// First album per track, with its track number and album-artist name.
struct AlbumInfo {
    album: Album,
    track_number: Option<u32>,
    album_artist_name: Option<String>,
}

fn batch_resolve_albums_for_tracks(
    conn: &Connection,
    track_ids: &[i64],
) -> Result<HashMap<i64, AlbumInfo>> {
    let mut map = HashMap::new();
    if track_ids.is_empty() {
        return Ok(map);
    }
    let mut unique: Vec<i64> = track_ids.to_vec();
    unique.sort();
    unique.dedup();

    for chunk in unique.chunks(400) {
        let placeholders: Vec<String> = chunk.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "SELECT a2.track_id, al.id, al.name, al.cover_art, alt2.track_number, al.album_artist, al.year
             FROM (SELECT track_id, MIN(album_id) as album_id
                   FROM album_track
                   WHERE track_id IN ({})
                   GROUP BY track_id) a2
             JOIN album_track alt2 ON alt2.album_id = a2.album_id AND alt2.track_id = a2.track_id
             JOIN album al ON al.id = a2.album_id",
            placeholders.join(",")
        );
        let mut stmt = conn.prepare(&sql).map_err(Error::Db)?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(chunk.iter()), |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    AlbumInfo {
                        album: Album {
                            id: row.get(1)?,
                            name: row.get(2)?,
                            cover_art: row.get(3)?,
                            album_artist: None,
                            year: row.get::<_, Option<i32>>(6)?.map(|y| y as u32),
                            ..Default::default()
                        },
                        track_number: row.get::<_, Option<i64>>(4)?.map(|n| n as u32),
                        album_artist_name: row.get(5)?,
                    },
                ))
            })
            .map_err(Error::Db)?;
        for row in rows {
            if let Ok((track_id, info)) = row {
                map.entry(track_id).or_insert(info);
            }
        }
    }
    Ok(map)
}

pub fn get_top_tracks_with_stats(
    conn: &Connection,
    timeframe: Timeframe,
    limit: usize,
    sort_by: TopSort,
) -> Result<Vec<TopTrack>> {
    let time_filter = timeframe_where_clause("ph.played_at", timeframe);

    // Aggregate playback per track first, then attach album info in a second
    // pass. Joining album_track up-front would multiply plays/listening time
    // for tracks that belong to several albums.
    let mut stmt = conn
        .prepare(&format!(
            "SELECT t.id, t.title, t.duration_sec, t.is_favorite, t.cover_art, t.added_at,
                    COUNT(ph.id) as play_count,
                    COALESCE(SUM(t.duration_sec * ph.completion_percent / 100.0), 0) as total_time,
                    MAX(ph.played_at) as last_played
             FROM playback_history ph
             JOIN track t ON t.id = ph.track_id
             WHERE {}
             GROUP BY t.id
             ORDER BY {}
             LIMIT ?",
            time_filter,
            top_order_by(sort_by, "t.title")
        ))
        .map_err(Error::Db)?;

    let mut items: Vec<(i64, TopTrack)> = stmt
        .query_map(params![limit as i64], |row| {
            let track = Track {
                id: row.get(0)?,
                title: row.get(1)?,
                artists: vec![],
                album: Album {
                    id: 0,
                    name: "Unknown Album".to_string(),
                    cover_art: None,
                    album_artist: None,
                    year: None,
                    ..Default::default()
                },
                duration_seconds: row.get(2)?,
                is_favorite: row.get(3)?,
                cover_art: row.get(4)?,
                added_at: row.get(5)?,
                track_number: None,
                playlist_ids: vec![],
                genre_ids: None,
                queue_id: None,
            };
            Ok((
                row.get::<_, i64>(0)?,
                TopTrack {
                    track,
                    play_count: row.get(6)?,
                    total_listening_time_sec: row.get::<_, f64>(7)? as i64,
                    last_played_at: row.get(8)?,
                },
            ))
        })
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?;

    if items.is_empty() {
        return Ok(Vec::new());
    }

    let track_ids: Vec<i64> = items.iter().map(|(id, _)| *id).collect();
    let album_map = batch_resolve_albums_for_tracks(conn, &track_ids)?;
    for (track_id, top) in &mut items {
        if let Some(info) = album_map.get(track_id) {
            top.track.album = info.album.clone();
            top.track.track_number = info.track_number;
        }
    }

    let aa_names: Vec<&str> = album_map
        .values()
        .filter_map(|info| info.album_artist_name.as_deref())
        .collect();
    if !aa_names.is_empty() {
        let album_artist_map = batch_resolve_album_artists(conn, &aa_names)?;
        for (track_id, top) in &mut items {
            if let Some(name) = album_map
                .get(track_id)
                .and_then(|i| i.album_artist_name.as_deref())
            {
                if let Some(artists) = album_artist_map.get(name) {
                    top.track.album.album_artist = Some(artists.clone());
                }
            }
        }
    }

    let artists_map = get_artists_for_tracks(conn, &track_ids)?;
    for (track_id, top) in &mut items {
        if let Some(artists) = artists_map.get(track_id) {
            top.track.artists = artists.clone();
        }
    }

    Ok(items.into_iter().map(|(_, top)| top).collect())
}

pub fn get_top_artists_with_stats(
    conn: &Connection,
    timeframe: Timeframe,
    limit: usize,
    sort_by: TopSort,
) -> Result<Vec<TopArtist>> {
    let time_filter = timeframe_where_clause("ph.played_at", timeframe);

    let mut stmt = conn
        .prepare(&format!(
            "SELECT ar.id, ar.name, ar.profile_image, ar.banner_image,
                    COUNT(DISTINCT ph.id) as play_count,
                    COALESCE(SUM(t.duration_sec * ph.completion_percent / 100.0), 0) as total_time,
                    COUNT(DISTINCT ph.track_id) as tracks_played
             FROM playback_history ph
             JOIN track t ON t.id = ph.track_id
             JOIN track_artist ta ON ta.track_id = t.id
             JOIN artist ar ON ar.id = ta.artist_id
             WHERE {}
             GROUP BY ar.id
             ORDER BY {}
             LIMIT ?",
            time_filter,
            top_order_by(sort_by, "ar.name")
        ))
        .map_err(Error::Db)?;

    let rows = stmt
        .query_map(params![limit as i64], |row| {
            Ok(TopArtist {
                artist: Artist {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    profile_image: row.get(2)?,
                    banner_image: row.get(3)?,
                    ..Default::default()
                },
                play_count: row.get(4)?,
                total_listening_time_sec: row.get::<_, f64>(5)? as i64,
                tracks_played: row.get(6)?,
            })
        })
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?;

    Ok(rows)
}

pub fn get_top_albums_with_stats(
    conn: &Connection,
    timeframe: Timeframe,
    limit: usize,
    sort_by: TopSort,
) -> Result<Vec<TopAlbum>> {
    let time_filter = timeframe_where_clause("ph.played_at", timeframe);

    let mut stmt = conn
        .prepare(&format!(
            "SELECT al.id, al.name, al.cover_art, al.album_artist, al.year,
                    COUNT(DISTINCT ph.id) as play_count,
                    COALESCE(SUM(t.duration_sec * ph.completion_percent / 100.0), 0) as total_time,
                    COUNT(DISTINCT ph.track_id) as tracks_played
             FROM playback_history ph
             JOIN track t ON t.id = ph.track_id
             JOIN album_track alt ON alt.track_id = t.id
             JOIN album al ON al.id = alt.album_id
             WHERE {}
             GROUP BY al.id
             ORDER BY {}
             LIMIT ?",
            time_filter,
            top_order_by(sort_by, "al.name")
        ))
        .map_err(Error::Db)?;

    let raw: Vec<(TopAlbum, Option<String>)> = stmt
        .query_map(params![limit as i64], |row| {
            Ok((
                TopAlbum {
                    album: Album {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        cover_art: row.get(2)?,
                        album_artist: None,
                        year: row.get::<_, Option<i32>>(4)?.map(|y| y as u32),
                        ..Default::default()
                    },
                    play_count: row.get(5)?,
                    total_listening_time_sec: row.get::<_, f64>(6)? as i64,
                    tracks_played: row.get(7)?,
                },
                row.get::<_, Option<String>>(3)?,
            ))
        })
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?;

    let mut top_albums: Vec<TopAlbum> = Vec::with_capacity(raw.len());
    for (mut ta, aa) in raw {
        ta.album.album_artist = resolve_album_artist(conn, aa)?;
        top_albums.push(ta);
    }
    Ok(top_albums)
}

pub fn get_top_genres_with_stats(
    conn: &Connection,
    timeframe: Timeframe,
    limit: usize,
    sort_by: TopSort,
) -> Result<Vec<TopGenre>> {
    let time_filter = timeframe_where_clause("ph.played_at", timeframe);

    let mut stmt = conn
        .prepare(&format!(
            "SELECT g.id, g.name, g.thumbnail,
                    COUNT(DISTINCT ph.id) as play_count,
                    COALESCE(SUM(t.duration_sec * ph.completion_percent / 100.0), 0) as total_time,
                    COUNT(DISTINCT ph.track_id) as tracks_played
             FROM playback_history ph
             JOIN track t ON t.id = ph.track_id
             JOIN track_genre tg ON tg.track_id = t.id
             JOIN genre g ON g.id = tg.genre_id
             WHERE {}
             GROUP BY g.id
             ORDER BY {}
             LIMIT ?",
            time_filter,
            top_order_by(sort_by, "g.name")
        ))
        .map_err(Error::Db)?;

    let rows = stmt
        .query_map(params![limit as i64], |row| {
            Ok(TopGenre {
                genre: Genre {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    thumbnail: row.get(2)?,
                    ..Default::default()
                },
                play_count: row.get(3)?,
                total_listening_time_sec: row.get::<_, f64>(4)? as i64,
                tracks_played: row.get(5)?,
            })
        })
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?;

    Ok(rows)
}

pub fn get_listening_time_trend(
    conn: &Connection,
    timeframe: Timeframe,
) -> Result<Vec<TimeSeriesPoint>> {
    let time_filter = timeframe_where_clause("ph.played_at", timeframe);

    let mut stmt = conn
        .prepare(&format!(
            "SELECT date(ph.played_at, 'localtime') as day,
                    COALESCE(SUM(t.duration_sec * ph.completion_percent / 100.0 / 60.0), 0) as minutes
             FROM playback_history ph
             JOIN track t ON t.id = ph.track_id
             WHERE {}
             GROUP BY date(ph.played_at, 'localtime')
             ORDER BY day ASC",
            time_filter
        ))
        .map_err(Error::Db)?;

    let rows = stmt
        .query_map([], |row| {
            Ok(TimeSeriesPoint {
                date: row.get(0)?,
                value: row.get(1)?,
            })
        })
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?;

    Ok(rows)
}

pub fn get_streak_data(conn: &Connection, timeframe: Timeframe) -> Result<StreakData> {
    let time_filter = timeframe_where_clause("played_at", timeframe);

    let mut stmt = conn
        .prepare(&format!(
            "SELECT DISTINCT date(played_at, 'localtime') as play_date,
                    (SELECT COUNT(*) FROM playback_history ph2 WHERE date(ph2.played_at, 'localtime') = date(ph.played_at, 'localtime')) as count
             FROM playback_history ph
             WHERE {}
             ORDER BY play_date ASC",
            time_filter
        ))
        .map_err(Error::Db)?;

    let date_counts: Vec<(String, i64)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?;

    // Streaks are computed from the same time-filtered set the calendar shows,
    // so the cards stay consistent with the selected timeframe.
    let dates: Vec<String> = date_counts.iter().map(|(d, _)| d.clone()).collect();
    let longest = compute_longest_streak(&dates);

    let today = chrono::Local::now().date_naive();
    let current = compute_current_streak(&dates, today);

    let streak_dates: Vec<String> = date_counts.iter().map(|(d, _)| d.clone()).collect();
    let daily_counts: HashMap<String, i64> = date_counts.into_iter().collect();

    Ok(StreakData {
        current_streak: current,
        longest_streak: longest,
        streak_dates,
        daily_counts,
    })
}

fn compute_longest_streak(dates: &[String]) -> i32 {
    if dates.is_empty() {
        return 0;
    }
    let mut max_streak = 1;
    let mut current_streak = 1;
    for i in 1..dates.len() {
        let prev = parse_date(&dates[i - 1]);
        let curr = parse_date(&dates[i]);
        if let (Some(p), Some(c)) = (prev, curr) {
            if (c - p).num_days() == 1 {
                current_streak += 1;
                max_streak = max_streak.max(current_streak);
            } else {
                current_streak = 1;
            }
        }
    }
    max_streak
}

fn compute_current_streak(dates: &[String], today: chrono::NaiveDate) -> i32 {
    if dates.is_empty() {
        return 0;
    }
    let last = parse_date(dates.last().unwrap());
    if let Some(l) = last {
        let diff = (today - l).num_days();
        if diff > 1 {
            return 0;
        }
    }

    let mut streak = 0;
    let mut expected = today;
    for d in dates.iter().rev() {
        if let Some(date) = parse_date(d) {
            if date == expected {
                streak += 1;
                expected = date - chrono::Duration::days(1);
            } else if streak == 0 && date == expected - chrono::Duration::days(1) {
                // Today hasn't been played yet, but yesterday was: the streak
                // is still alive. Only the first iteration may skip a day so a
                // gap in the middle of the streak never counts.
                streak += 1;
                expected = date - chrono::Duration::days(1);
            } else {
                break;
            }
        }
    }
    streak
}

fn parse_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

pub fn get_library_growth(conn: &Connection) -> Result<Vec<GrowthPoint>> {
    let period_fmt: String = conn
        .query_row(
            "SELECT
                CASE
                    WHEN JULIANDAY(MAX(added_at)) - JULIANDAY(MIN(added_at)) < 31 THEN '%Y-%m-%d'
                    WHEN JULIANDAY(MAX(added_at)) - JULIANDAY(MIN(added_at)) < 365 THEN '%Y-%W'
                    ELSE '%Y-%m'
                END
             FROM track",
            [],
            |row| row.get(0),
        )
        .map_err(Error::Db)?;

    let pe = format!("strftime('{}', t.added_at)", period_fmt);

    let mut stmt = conn
        .prepare(&format!(
            "SELECT {pe} as period, COUNT(*) as tracks
             FROM track t
             GROUP BY period
             ORDER BY period ASC",
            pe = pe
        ))
        .map_err(Error::Db)?;

    let track_growth: HashMap<String, i64> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?
        .into_iter()
        .collect();

    // Artist growth: count artists by period of their first track appearance
    let pe_artist = format!("strftime('{}', first.first_added)", period_fmt);
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {pe} as period, COUNT(*) as artists
             FROM (
                 SELECT MIN(t.added_at) as first_added
                 FROM artist ar
                 JOIN track_artist ta ON ta.artist_id = ar.id
                 JOIN track t ON t.id = ta.track_id
                 GROUP BY ar.id
             ) first
             GROUP BY period
             ORDER BY period ASC",
            pe = pe_artist
        ))
        .map_err(Error::Db)?;

    let artist_growth: HashMap<String, i64> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?
        .into_iter()
        .collect();

    // Album growth: count albums by period of their first track appearance
    let pe_album = format!("strftime('{}', first.first_added)", period_fmt);
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {pe} as period, COUNT(*) as albums
             FROM (
                 SELECT MIN(t.added_at) as first_added
                 FROM album al
                 JOIN album_track alt ON alt.album_id = al.id
                 JOIN track t ON t.id = alt.track_id
                 GROUP BY al.id
             ) first
             GROUP BY period
             ORDER BY period ASC",
            pe = pe_album
        ))
        .map_err(Error::Db)?;

    let album_growth: HashMap<String, i64> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?
        .into_iter()
        .collect();

    // Collect all unique periods sorted
    let mut periods: Vec<String> = track_growth.keys().cloned().collect();
    for key in artist_growth.keys() {
        if !periods.contains(key) {
            periods.push(key.clone());
        }
    }
    for key in album_growth.keys() {
        if !periods.contains(key) {
            periods.push(key.clone());
        }
    }
    periods.sort();

    let mut result = Vec::new();
    let mut cum_tracks: i64 = 0;
    let mut cum_artists: i64 = 0;
    let mut cum_albums: i64 = 0;

    for period in &periods {
        cum_tracks += track_growth.get(period).copied().unwrap_or(0);
        cum_artists += artist_growth.get(period).copied().unwrap_or(0);
        cum_albums += album_growth.get(period).copied().unwrap_or(0);
        result.push(GrowthPoint {
            period: period.clone(),
            tracks_added: cum_tracks,
            artists_added: cum_artists,
            albums_added: cum_albums,
        });
    }

    Ok(result)
}

pub fn get_heatmap_hourly(conn: &Connection, timeframe: Timeframe) -> Result<Vec<HeatmapCell>> {
    let time_filter = timeframe_where_clause("played_at", timeframe);

    let day_names = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];

    let mut stmt = conn
        .prepare(&format!(
            "SELECT CAST(strftime('%w', played_at, 'localtime') AS INTEGER) as weekday,
                    CAST(strftime('%H', played_at, 'localtime') AS INTEGER) as hour,
                    COUNT(*) as count
             FROM playback_history ph
             WHERE {}
             GROUP BY weekday, hour
             ORDER BY weekday, hour",
            time_filter
        ))
        .map_err(Error::Db)?;

    let raw: Vec<(i64, i64, i64)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?;

    let mut cells = Vec::new();
    for wd in 0..7 {
        for h in 0..24 {
            let count = raw
                .iter()
                .find(|&&(w, hh, _)| w == wd && hh == h)
                .map(|&(_, _, c)| c)
                .unwrap_or(0);
            cells.push(HeatmapCell {
                label: format!("{}, {}:00", day_names[wd as usize], h),
                value: count,
            });
        }
    }

    Ok(cells)
}

pub fn get_heatmap_weekday(conn: &Connection, timeframe: Timeframe) -> Result<Vec<HeatmapCell>> {
    let time_filter = timeframe_where_clause("played_at", timeframe);

    let day_names = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];

    let mut stmt = conn
        .prepare(&format!(
            "SELECT CAST(strftime('%w', played_at, 'localtime') AS INTEGER) as weekday,
                    COUNT(*) as count
             FROM playback_history ph
             WHERE {}
             GROUP BY weekday
             ORDER BY weekday",
            time_filter
        ))
        .map_err(Error::Db)?;

    let raw: Vec<(i64, i64)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?;

    let cells: Vec<HeatmapCell> = (0..7)
        .map(|wd| {
            let count = raw
                .iter()
                .find(|&&(w, _)| w == wd)
                .map(|&(_, c)| c)
                .unwrap_or(0);
            HeatmapCell {
                label: day_names[wd as usize].to_string(),
                value: count,
            }
        })
        .collect();

    Ok(cells)
}

pub fn get_favorite_trends(conn: &Connection, timeframe: Timeframe) -> Result<Vec<FavoriteTrend>> {
    let time_filter = timeframe_where_clause("ph.played_at", timeframe);
    let period_expr = "strftime('%Y-%m', ph.played_at, 'localtime')";

    let mut stmt = conn
        .prepare(&format!(
            "SELECT {pe} as period,
                    t.id as track_id, t.title as track_name,
                    COUNT(ph.id) as track_plays
             FROM playback_history ph
             JOIN track t ON t.id = ph.track_id
             WHERE {}
             GROUP BY period, t.id
             ORDER BY period ASC, track_plays DESC
            ",
            time_filter,
            pe = period_expr
        ))
        .map_err(Error::Db)?;

    let mut rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?;

    // Group by period and take top track per period
    rows.sort_by(|a, b| a.0.cmp(&b.0).then(b.3.cmp(&a.3)));

    let mut periods: Vec<String> = Vec::new();
    let mut top_track_by_period: HashMap<String, (i64, String)> = HashMap::new();
    for (period, track_id, track_name, _) in &rows {
        if !top_track_by_period.contains_key(period) {
            top_track_by_period.insert(period.clone(), (*track_id, track_name.clone()));
            periods.push(period.clone());
        }
    }

    // Top artist per period: count plays per artist within the timeframe.
    let top_artist_by_period = collect_top_per_period(
        conn,
        &format!(
            "SELECT {pe} as period, ar.id as id, ar.name as name, COUNT(ph.id) as plays
             FROM playback_history ph
             JOIN track t ON t.id = ph.track_id
             JOIN track_artist ta ON ta.track_id = t.id
             JOIN artist ar ON ar.id = ta.artist_id
             WHERE {}
             GROUP BY period, ar.id
             ORDER BY period ASC, plays DESC",
            time_filter,
            pe = period_expr
        ),
    )?;

    // Top album per period: count plays per album within the timeframe.
    let top_album_by_period = collect_top_per_period(
        conn,
        &format!(
            "SELECT {pe} as period, al.id as id, al.name as name, COUNT(ph.id) as plays
             FROM playback_history ph
             JOIN track t ON t.id = ph.track_id
             JOIN album_track alt ON alt.track_id = t.id
             JOIN album al ON al.id = alt.album_id
             WHERE {}
             GROUP BY period, al.id
             ORDER BY period ASC, plays DESC",
            time_filter,
            pe = period_expr
        ),
    )?;

    let mut result = Vec::new();
    for period in &periods {
        let (track_id_opt, track_name_opt) = top_track_by_period
            .get(period)
            .map(|(id, n)| (*id, n.clone()))
            .unzip();
        let (artist_id_opt, artist_name_opt) = top_artist_by_period
            .get(period)
            .map(|(id, n)| (*id, n.clone()))
            .unzip();
        let (album_id_opt, album_name_opt) = top_album_by_period
            .get(period)
            .map(|(id, n)| (*id, n.clone()))
            .unzip();

        result.push(FavoriteTrend {
            period: period.clone(),
            top_track_id: track_id_opt,
            top_track_name: track_name_opt,
            top_artist_id: artist_id_opt,
            top_artist_name: artist_name_opt,
            top_album_id: album_id_opt,
            top_album_name: album_name_opt,
        });
    }

    Ok(result)
}

/// Runs a `(period, id, name, plays)` query ordered by plays desc and keeps
/// the top row (highest plays) per period.
fn collect_top_per_period(conn: &Connection, sql: &str) -> Result<HashMap<String, (i64, String)>> {
    let mut stmt = conn.prepare(sql).map_err(Error::Db)?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?;

    let mut top: HashMap<String, (i64, String)> = HashMap::new();
    for (period, id, name) in rows {
        top.entry(period).or_insert((id, name));
    }
    Ok(top)
}

pub fn get_playback_history_timeline(
    conn: &Connection,
    timeframe: Timeframe,
    limit: usize,
) -> Result<Vec<PlaybackEvent>> {
    let time_filter = timeframe_where_clause("ph.played_at", timeframe);

    let mut stmt = conn
        .prepare(&format!(
            "SELECT ph.played_at, t.id, t.title, t.duration_sec, t.is_favorite, t.cover_art, t.added_at,
                    ph.completion_percent, ph.source_type
             FROM playback_history ph
             JOIN track t ON t.id = ph.track_id
             WHERE {}
             ORDER BY ph.played_at DESC
             LIMIT ?",
            time_filter
        ))
        .map_err(Error::Db)?;

    let mut raw_events: Vec<(i64, PlaybackEvent)> = stmt
        .query_map(params![limit as i64], |row| {
            let track = Track {
                id: row.get(1)?,
                title: row.get(2)?,
                artists: vec![],
                album: Album {
                    id: 0,
                    name: "Unknown Album".to_string(),
                    cover_art: None,
                    album_artist: None,
                    year: None,
                    ..Default::default()
                },
                duration_seconds: row.get(3)?,
                is_favorite: row.get(4)?,
                cover_art: row.get(5)?,
                added_at: row.get(6)?,
                track_number: None,
                playlist_ids: vec![],
                genre_ids: None,
                queue_id: None,
            };

            Ok((
                row.get::<_, i64>(1)?,
                PlaybackEvent {
                    played_at: row.get(0)?,
                    track,
                    completion_percent: row.get(7)?,
                    source_type: row.get::<_, String>(8)?.to_lowercase(),
                },
            ))
        })
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?;

    if !raw_events.is_empty() {
        let track_ids: Vec<i64> = raw_events.iter().map(|(id, _)| *id).collect();
        let album_map = batch_resolve_albums_for_tracks(conn, &track_ids)?;
        for (track_id, event) in &mut raw_events {
            if let Some(info) = album_map.get(track_id) {
                event.track.album = info.album.clone();
                event.track.track_number = info.track_number;
            }
        }

        let aa_names: Vec<&str> = album_map
            .values()
            .filter_map(|info| info.album_artist_name.as_deref())
            .collect();
        if !aa_names.is_empty() {
            let album_artist_map = batch_resolve_album_artists(conn, &aa_names)?;
            for (track_id, event) in &mut raw_events {
                if let Some(name) = album_map
                    .get(track_id)
                    .and_then(|i| i.album_artist_name.as_deref())
                {
                    if let Some(artists) = album_artist_map.get(name) {
                        event.track.album.album_artist = Some(artists.clone());
                    }
                }
            }
        }

        let artists_map = get_artists_for_tracks(conn, &track_ids)?;
        for (track_id, event) in &mut raw_events {
            if let Some(artists) = artists_map.get(track_id) {
                event.track.artists = artists.clone();
            }
        }
    }

    Ok(raw_events.into_iter().map(|(_, event)| event).collect())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataAge {
    pub min_track_added_at: Option<String>,
    pub min_played_at: Option<String>,
    pub data_age_days: i64,
}

pub fn get_data_age(conn: &Connection) -> Result<DataAge> {
    let min_track: Option<String> = conn
        .query_row("SELECT MIN(added_at) FROM track", [], |row| row.get(0))
        .ok()
        .flatten();

    let min_play: Option<String> = conn
        .query_row("SELECT MIN(played_at) FROM playback_history", [], |row| {
            row.get(0)
        })
        .ok()
        .flatten();

    let now = chrono::Utc::now().naive_utc();
    let max_age = |s: &str| -> i64 {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
            (now - dt).num_days().max(0)
        } else {
            0
        }
    };
    let data_age_days = match (&min_track, &min_play) {
        (Some(t), Some(p)) => max_age(t).max(max_age(p)),
        (Some(t), None) => max_age(t),
        (None, Some(p)) => max_age(p),
        (None, None) => 0,
    };

    Ok(DataAge {
        min_track_added_at: min_track,
        min_played_at: min_play,
        data_age_days,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_album_artist_by_id() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        init_db(&mut conn).unwrap();

        let album_id = get_or_create_album(&conn, "Test Album", None, Some(2024)).unwrap();
        set_album_artist_by_id(&conn, album_id, "Test Album Artist").unwrap();

        let artist: String = conn
            .query_row(
                "SELECT album_artist FROM album WHERE id = ?",
                params![album_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(artist, "Test Album Artist");
    }

    #[test]
    fn test_large_bulk_inserts_chunking() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        init_db(&mut conn).unwrap();

        let artist_id = get_or_create_artist(&conn, "Artist 1").unwrap();
        let album_id = get_or_create_album(&conn, "Album 1", None, None).unwrap();
        let genre_id = get_or_create_genre(&conn, "Genre 1").unwrap();

        let mut tracks = Vec::new();
        for idx in 0..600 {
            let path = format!("/tmp/track_{idx}.flac");
            let tid = update_track(
                &conn, &path, "Title", 180, None, 1000, 1000, None, None, None, 44100, None, 2,
                "flac", None, None, None, None, None, None, None,
            )
            .unwrap();
            tracks.push(tid);
        }

        // Test bulk artist insert with 600 pairs (1200 params > 999 limit)
        let artist_pairs: Vec<(i64, i64)> = tracks.iter().map(|&tid| (tid, artist_id)).collect();
        assert!(bulk_insert_track_artists(&conn, &artist_pairs).is_ok());

        // Test bulk album insert with 600 entries (1800 params > 999 limit)
        let album_entries: Vec<(i64, i64, i32)> =
            tracks.iter().map(|&tid| (album_id, tid, 1)).collect();
        assert!(bulk_insert_track_albums(&conn, &album_entries).is_ok());

        // Test bulk genre insert with 600 pairs (1200 params > 999 limit)
        let genre_pairs: Vec<(i64, i64)> = tracks.iter().map(|&tid| (tid, genre_id)).collect();
        assert!(bulk_insert_track_genres(&conn, &genre_pairs).is_ok());
    }

    #[test]
    fn migrations_validate() {
        assert!(MIGRATIONS.validate().is_ok());
    }

    #[test]
    fn test_timeframe_where_clause_today() {
        let result = timeframe_where_clause("t.col", Timeframe::Today);
        assert_eq!(
            result,
            "t.col >= datetime('now', 'localtime', 'start of day')"
        );
    }

    #[test]
    fn test_timeframe_where_clause_this_week() {
        let result = timeframe_where_clause("t.col", Timeframe::ThisWeek);
        assert_eq!(result, "t.col >= datetime('now', 'localtime', '-7 days')");
    }

    #[test]
    fn test_timeframe_where_clause_this_month() {
        let result = timeframe_where_clause("t.col", Timeframe::ThisMonth);
        assert_eq!(result, "t.col >= datetime('now', 'localtime', '-30 days')");
    }

    #[test]
    fn test_timeframe_where_clause_last_3_months() {
        let result = timeframe_where_clause("t.col", Timeframe::Last3Months);
        assert_eq!(result, "t.col >= datetime('now', 'localtime', '-90 days')");
    }

    #[test]
    fn test_timeframe_where_clause_last_year() {
        let result = timeframe_where_clause("t.col", Timeframe::LastYear);
        assert_eq!(result, "t.col >= datetime('now', 'localtime', '-1 year')");
    }

    #[test]
    fn test_timeframe_where_clause_all_time() {
        let result = timeframe_where_clause("t.col", Timeframe::AllTime);
        assert_eq!(result, "1=1");
    }

    #[test]
    fn test_sql_placeholders_single() {
        assert_eq!(sql_placeholders(1), "?");
    }

    #[test]
    fn test_sql_placeholders_multiple() {
        assert_eq!(sql_placeholders(3), "?,?,?");
    }

    #[test]
    fn test_sql_placeholders_zero() {
        assert_eq!(sql_placeholders(0), "");
    }

    #[test]
    fn test_sql_placeholders_large() {
        let result = sql_placeholders(5);
        assert_eq!(result, "?,?,?,?,?");
    }

    #[test]
    fn test_in_memory_db_init() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        init_db(&mut conn).unwrap();
        assert!(
            conn.query_row("SELECT COUNT(*) FROM track", [], |r| r.get::<_, i64>(0))
                .is_ok()
        );
    }

    #[test]
    fn test_add_and_get_source_dirs() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        init_db(&mut conn).unwrap();

        add_source_dir(&conn, "/music/rock").unwrap();
        add_source_dir(&conn, "/music/jazz").unwrap();
        let dirs = get_source_dirs(&conn).unwrap();
        assert_eq!(dirs.len(), 2);
        assert!(dirs.contains(&"/music/rock".to_string()));
        assert!(dirs.contains(&"/music/jazz".to_string()));
    }

    #[test]
    fn test_add_source_dir_ignores_duplicates() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        init_db(&mut conn).unwrap();

        add_source_dir(&conn, "/music/rock").unwrap();
        add_source_dir(&conn, "/music/rock").unwrap();
        let dirs = get_source_dirs(&conn).unwrap();
        assert_eq!(dirs.len(), 1);
    }

    #[test]
    fn test_get_or_create_artist_new() {
        let conn = setup_memory_db();
        let id = get_or_create_artist(&conn, "Test Artist").unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_get_or_create_artist_existing() {
        let conn = setup_memory_db();
        let id1 = get_or_create_artist(&conn, "Test Artist").unwrap();
        let id2 = get_or_create_artist(&conn, "Test Artist").unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_get_or_create_album_new() {
        let conn = setup_memory_db();
        let id = get_or_create_album(&conn, "Test Album", None, None).unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_get_or_create_album_existing() {
        let conn = setup_memory_db();
        let id1 = get_or_create_album(&conn, "Test Album", None, None).unwrap();
        let id2 = get_or_create_album(&conn, "Test Album", Some("cover.webp"), Some(2024)).unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_update_and_get_track() {
        let conn = setup_memory_db();
        let track_id = update_track(
            &conn,
            "/music/test.mp3",
            "Test Song",
            200,
            Some(2024),
            1000,
            5000,
            None,
            None,
            None,
            0,
            None,
            0,
            "",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert!(track_id > 0);

        let artist_id = get_or_create_artist(&conn, "Test Artist").unwrap();
        set_track_artists(&conn, track_id, &[artist_id]).unwrap();

        let album_id = get_or_create_album(&conn, "Test Album", None, None).unwrap();
        set_track_album(&conn, track_id, album_id, 1).unwrap();

        let track = get_track_by_id(&conn, track_id).unwrap();
        assert_eq!(track.title, "Test Song");
        assert_eq!(track.duration_seconds, 200);
        assert_eq!(track.album.name, "Test Album");
    }

    #[test]
    fn test_toggle_favorite() {
        let conn = setup_memory_db();
        let track_id = update_track(
            &conn,
            "/music/test.mp3",
            "Test",
            100,
            None,
            0,
            100,
            None,
            None,
            None,
            0,
            None,
            0,
            "",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let artist_id = get_or_create_artist(&conn, "Artist").unwrap();
        set_track_artists(&conn, track_id, &[artist_id]).unwrap();
        let album_id = get_or_create_album(&conn, "Album", None, None).unwrap();
        set_track_album(&conn, track_id, album_id, 1).unwrap();

        let track = toggle_favorite(&conn, track_id).unwrap();
        assert!(track.is_favorite);

        let track = toggle_favorite(&conn, track_id).unwrap();
        assert!(!track.is_favorite);
    }

    #[test]
    fn test_create_and_get_playlist() {
        let conn = setup_memory_db();
        let playlist = create_playlist(&conn, "My Favorites").unwrap();
        assert_eq!(playlist.name, "My Favorites");

        let playlists = get_all_playlists(&conn).unwrap();
        assert_eq!(playlists.len(), 1);
        assert_eq!(playlists[0].name, "My Favorites");
    }

    #[test]
    fn test_add_track_to_playlist() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        let playlist = create_playlist(&conn, "Test Playlist").unwrap();

        add_track_to_playlist(&conn, playlist.id, track_id).unwrap();
        let tracks = get_tracks_in_playlist(&conn, playlist.id, None).unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].id, track_id);
    }

    #[test]
    fn test_remove_track_from_playlist() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        let playlist = create_playlist(&conn, "Test").unwrap();

        add_track_to_playlist(&conn, playlist.id, track_id).unwrap();
        remove_track_from_playlist(&conn, playlist.id, track_id).unwrap();
        let tracks = get_tracks_in_playlist(&conn, playlist.id, None).unwrap();
        assert_eq!(tracks.len(), 0);
    }

    #[test]
    fn test_delete_playlist() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        let playlist = create_playlist(&conn, "Test").unwrap();

        add_track_to_playlist(&conn, playlist.id, track_id).unwrap();
        delete_playlist(&conn, playlist.id).unwrap();

        let playlists = get_all_playlists(&conn).unwrap();
        assert_eq!(playlists.len(), 0);
    }

    #[test]
    fn test_get_all_tracks() {
        let conn = setup_memory_db();
        insert_basic_track(&conn);
        insert_basic_track_with_path(&conn, "/music/song2.mp3", 2);

        let tracks = get_all_tracks(&conn, None).unwrap();
        assert_eq!(tracks.len(), 2);
    }

    #[test]
    fn test_search_tracks() {
        let conn = setup_memory_db();
        insert_basic_track(&conn);
        let track_id = update_track(
            &conn,
            "/music/other.mp3",
            "Other Song",
            200,
            Some(2024),
            2000,
            6000,
            None,
            None,
            None,
            0,
            None,
            0,
            "",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let artist_id = get_or_create_artist(&conn, "Other Artist").unwrap();
        set_track_artists(&conn, track_id, &[artist_id]).unwrap();
        let album_id = get_or_create_album(&conn, "Other Album", None, None).unwrap();
        set_track_album(&conn, track_id, album_id, 2).unwrap();

        let results = search_tracks(&conn, "Test", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Test Song");
    }

    #[test]
    fn test_get_track_path_by_id() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        let path = get_track_path_by_id(&conn, track_id).unwrap();
        assert_eq!(path, "/music/test.mp3");
    }

    #[test]
    fn test_remove_source_dir() {
        let mut conn = setup_memory_db();
        add_source_dir(&conn, "/music/rock").unwrap();
        add_source_dir(&conn, "/music/jazz").unwrap();

        let track_id = update_track(
            &conn,
            "/music/rock/song.mp3",
            "Song",
            100,
            None,
            0,
            100,
            None,
            None,
            None,
            0,
            None,
            0,
            "",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let artist_id = get_or_create_artist(&conn, "Artist").unwrap();
        set_track_artists(&conn, track_id, &[artist_id]).unwrap();

        let album_id = get_or_create_album(&conn, "Album", None, None).unwrap();
        set_track_album(&conn, track_id, album_id, 1).unwrap();

        remove_source_dir(&mut conn, "/music/rock").unwrap();
        let dirs = get_source_dirs(&conn).unwrap();
        assert_eq!(dirs.len(), 1);
    }

    #[test]
    fn test_get_track_details() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        let details = get_track_details(&conn, track_id).unwrap();
        assert_eq!(details.title, "Test Song");
        assert_eq!(details.path, "/music/test.mp3");
    }

    #[test]
    fn test_get_similar_tracks() {
        let conn = setup_memory_db();
        let track1 = insert_basic_track(&conn);
        let track2 = insert_basic_track_with_path(&conn, "/music/song2.mp3", 2);

        let similar = get_similar_tracks(&conn, track1, 5).unwrap();
        assert_eq!(similar.len(), 1);
        assert_eq!(similar[0].id, track2);
    }

    #[test]
    fn test_fetch_tracking() {
        let conn = setup_memory_db();
        let id = get_or_create_artist(&conn, "Fetch Artist").unwrap();

        report_fetch_success(&conn, id).unwrap();
        let needs = get_artists_needing_fetch(&conn).unwrap();
        assert!(needs.iter().any(|(aid, _)| *aid == id));

        report_fetch_failure(&conn, id).unwrap();
        report_fetch_failure(&conn, id).unwrap();
        report_fetch_failure(&conn, id).unwrap();
        let needs = get_artists_needing_fetch(&conn).unwrap();
        assert!(!needs.iter().any(|(aid, _)| *aid == id));
    }

    #[test]
    fn test_get_all_tracks_with_sort() {
        let conn = setup_memory_db();
        insert_basic_track(&conn); // "Test Song"
        insert_basic_track_with_path(&conn, "/music/a-song.mp3", 2);

        let by_title = get_all_tracks(&conn, Some(SortBy::Title)).unwrap();
        assert_eq!(by_title[0].title, "Other Song");
        assert_eq!(by_title[1].title, "Test Song");

        let by_recent = get_all_tracks(&conn, Some(SortBy::RecentlyAdded)).unwrap();
        assert!(by_recent[0].added_at >= by_recent[1].added_at);
    }

    #[test]
    fn test_toggle_favorite_nonexistent_track() {
        let conn = setup_memory_db();
        assert!(toggle_favorite(&conn, 9999).is_err());
    }

    #[test]
    fn test_get_playlist_by_name() {
        let conn = setup_memory_db();
        create_playlist(&conn, "Unique Playlist").unwrap();
        let found = get_playlist_by_name(&conn, "Unique Playlist").unwrap();
        assert_eq!(found.name, "Unique Playlist");
        assert!(get_playlist_by_name(&conn, "Does Not Exist").is_err());
    }

    fn setup_memory_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        init_db(&mut conn).unwrap();
        conn
    }

    fn make_track(conn: &Connection, path: &str, title: &str, duration: u32, size: i64) -> i64 {
        update_track(
            conn,
            path,
            title,
            duration,
            None,
            0u64 as i64,
            size,
            None,
            None,
            None,
            0,
            None,
            0,
            "",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap()
    }

    fn insert_basic_track(conn: &Connection) -> i64 {
        insert_basic_track_with_path(conn, "/music/test.mp3", 1)
    }

    fn insert_basic_track_with_path(conn: &Connection, path: &str, id_suffix: i64) -> i64 {
        let track_id = make_track(
            conn,
            path,
            &format!("{} Song", if id_suffix == 1 { "Test" } else { "Other" }),
            200,
            5000,
        );
        let artist_id = get_or_create_artist(conn, "Test Artist").unwrap();
        set_track_artists(conn, track_id, &[artist_id]).unwrap();
        let album_id = get_or_create_album(conn, "Test Album", None, None).unwrap();
        set_track_album(conn, track_id, album_id, id_suffix as i32).unwrap();
        track_id
    }

    // -----------------------------------------------------------------------
    // Stats helpers
    // -----------------------------------------------------------------------

    fn insert_play_at(
        conn: &Connection,
        track_id: i64,
        played_at: &str,
        source_type: &str,
        completion_percent: f64,
    ) {
        conn.execute(
            "INSERT INTO playback_history (track_id, played_at, source_type, completion_percent) VALUES (?1, ?2, ?3, ?4)",
            params![track_id, played_at, source_type, completion_percent],
        ).unwrap();
    }

    /// Local weekday (0=Sunday) + hour for a stored UTC timestamp. Mirrors
    /// SQLite's `'localtime'` modifier so heatmap tests are TZ-agnostic.
    fn local_hour_weekday(utc_str: &str) -> (i64, i64) {
        use chrono::{Datelike, Timelike};
        let dt = chrono::NaiveDateTime::parse_from_str(utc_str, "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc()
            .with_timezone(&chrono::Local);
        (dt.weekday().num_days_from_sunday() as i64, dt.hour() as i64)
    }

    // -----------------------------------------------------------------------
    // Stats overview tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_stats_overview_empty() {
        let conn = setup_memory_db();
        let overview = get_stats_overview(&conn, Timeframe::AllTime).unwrap();
        assert_eq!(overview.total_tracks, 0);
        assert_eq!(overview.total_artists, 0);
        assert_eq!(overview.total_albums, 0);
        assert_eq!(overview.total_plays, 0);
        assert_eq!(overview.total_listening_time_sec, 0);
        assert_eq!(overview.avg_daily_listening_min, 0.0);
        assert_eq!(overview.unplayed_tracks, 0);
    }

    #[test]
    fn test_stats_overview_with_tracks_no_plays() {
        let conn = setup_memory_db();
        insert_basic_track(&conn);
        let overview = get_stats_overview(&conn, Timeframe::AllTime).unwrap();
        assert_eq!(overview.total_tracks, 1);
        assert_eq!(overview.total_artists, 1);
        assert_eq!(overview.total_albums, 1);
        assert_eq!(overview.total_plays, 0);
        assert_eq!(overview.total_listening_time_sec, 0);
        assert_eq!(overview.unplayed_tracks, 1);
    }

    #[test]
    fn test_stats_overview_with_plays() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 100.0);
        let overview = get_stats_overview(&conn, Timeframe::AllTime).unwrap();
        assert_eq!(overview.total_plays, 1);
        assert_eq!(overview.total_listening_time_sec, 200);
        assert_eq!(overview.unplayed_tracks, 0);
    }

    #[test]
    fn test_stats_overview_with_partial_play() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 50.0);
        let overview = get_stats_overview(&conn, Timeframe::AllTime).unwrap();
        assert_eq!(overview.total_plays, 1);
        assert_eq!(overview.total_listening_time_sec, 100);
    }

    #[test]
    fn test_stats_overview_timeframe_filters() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 100.0);
        // Today's timeframe should not include 2024 plays
        let overview = get_stats_overview(&conn, Timeframe::Today).unwrap();
        assert_eq!(overview.total_plays, 0);
    }

    #[test]
    fn test_stats_overview_file_size() {
        let conn = setup_memory_db();
        update_track(
            &conn,
            "/music/small.mp3",
            "Small",
            100,
            None,
            0,
            1048576,
            None,
            None,
            None,
            0,
            None,
            0,
            "",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        update_track(
            &conn,
            "/music/large.flac",
            "Large",
            200,
            None,
            0,
            5242880,
            None,
            None,
            None,
            0,
            None,
            0,
            "",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let overview = get_stats_overview(&conn, Timeframe::AllTime).unwrap();
        assert_eq!(overview.total_tracks, 2);
        assert_eq!(overview.total_file_size_bytes, 6291456);
        assert!((overview.avg_file_size_mb - 3.0).abs() < 0.001);
        assert!((overview.largest_file_mb - 5.0).abs() < 0.001);
    }

    // -----------------------------------------------------------------------
    // get_top_tracks_with_stats tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_top_tracks_with_stats_empty() {
        let conn = setup_memory_db();
        let result =
            get_top_tracks_with_stats(&conn, Timeframe::AllTime, 10, TopSort::default()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_top_tracks_with_stats_ordering() {
        let conn = setup_memory_db();
        let t1 = insert_basic_track(&conn);
        let t2 = insert_basic_track_with_path(&conn, "/music/b.mp3", 2);
        insert_play_at(&conn, t1, "2024-01-15 10:00:00", "ALBUM", 100.0);
        insert_play_at(&conn, t1, "2024-01-16 10:00:00", "ALBUM", 100.0);
        insert_play_at(&conn, t2, "2024-01-15 10:00:00", "ALBUM", 100.0);
        let result =
            get_top_tracks_with_stats(&conn, Timeframe::AllTime, 10, TopSort::default()).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].play_count, 2);
        assert_eq!(result[1].play_count, 1);
        assert_eq!(result[0].total_listening_time_sec, 400);
        assert_eq!(result[1].total_listening_time_sec, 200);
    }

    #[test]
    fn test_top_tracks_with_stats_partial_completion() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 50.0);
        let result =
            get_top_tracks_with_stats(&conn, Timeframe::AllTime, 10, TopSort::default()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].total_listening_time_sec, 100);
    }

    #[test]
    fn test_top_tracks_with_stats_limit() {
        let conn = setup_memory_db();
        let t1 = insert_basic_track(&conn);
        let t2 = insert_basic_track_with_path(&conn, "/music/b.mp3", 2);
        insert_play_at(&conn, t1, "2024-01-15 10:00:00", "ALBUM", 100.0);
        insert_play_at(&conn, t2, "2024-01-15 10:00:00", "ALBUM", 100.0);
        let result =
            get_top_tracks_with_stats(&conn, Timeframe::AllTime, 1, TopSort::default()).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_top_tracks_with_stats_last_played() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 100.0);
        insert_play_at(&conn, track_id, "2024-06-20 15:30:00", "ALBUM", 100.0);
        let result =
            get_top_tracks_with_stats(&conn, Timeframe::AllTime, 10, TopSort::default()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].play_count, 2);
        assert!(result[0].last_played_at.is_some());
    }

    #[test]
    fn test_top_tracks_with_stats_multi_album_no_inflation() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        let album2 = get_or_create_album(&conn, "Compilation", None, Some(2024)).unwrap();
        conn.execute(
            "INSERT INTO album_track (album_id, track_id, track_number) VALUES (?, ?, ?)",
            params![album2, track_id, 5],
        )
        .unwrap();
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 100.0);
        insert_play_at(&conn, track_id, "2024-01-16 10:00:00", "ALBUM", 50.0);
        let result =
            get_top_tracks_with_stats(&conn, Timeframe::AllTime, 10, TopSort::default()).unwrap();
        assert_eq!(result.len(), 1);
        // One play per history row regardless of how many albums the track is in.
        assert_eq!(result[0].play_count, 2);
        assert_eq!(result[0].total_listening_time_sec, 300);
        // Album is resolved deterministically (the first album row by id).
        assert!(result[0].track.album.id == 1 || result[0].track.album.id == album2);
    }

    #[test]
    fn test_top_tracks_with_stats_sort_by_time() {
        let conn = setup_memory_db();
        // t1: 1 play of a 200s track at 100% => 200s
        let t1 = insert_basic_track(&conn);
        // t2: 2 plays of a 100s track at 50% => 100s total
        let t2 = update_track(
            &conn,
            "/music/other.mp3",
            "Other Song",
            100,
            Some(2024),
            2000,
            6000,
            None,
            None,
            None,
            0,
            None,
            0,
            "",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        insert_play_at(&conn, t1, "2024-01-15 10:00:00", "ALBUM", 100.0);
        insert_play_at(&conn, t2, "2024-01-15 10:00:00", "ALBUM", 50.0);
        insert_play_at(&conn, t2, "2024-01-16 10:00:00", "ALBUM", 50.0);
        let by_plays =
            get_top_tracks_with_stats(&conn, Timeframe::AllTime, 10, TopSort::Plays).unwrap();
        let by_time =
            get_top_tracks_with_stats(&conn, Timeframe::AllTime, 10, TopSort::Time).unwrap();
        assert_eq!(by_plays[0].track.id, t2);
        assert_eq!(by_plays[0].play_count, 2);
        assert_eq!(by_time[0].track.id, t1);
        assert_eq!(by_time[0].total_listening_time_sec, 200);
    }

    // -----------------------------------------------------------------------
    // get_top_artists_with_stats tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_top_artists_with_stats_empty() {
        let conn = setup_memory_db();
        let result =
            get_top_artists_with_stats(&conn, Timeframe::AllTime, 10, TopSort::default()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_top_artists_with_stats_single() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 100.0);
        let result =
            get_top_artists_with_stats(&conn, Timeframe::AllTime, 10, TopSort::default()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].artist.name, "Test Artist");
        assert_eq!(result[0].play_count, 1);
        assert_eq!(result[0].tracks_played, 1);
        assert_eq!(result[0].total_listening_time_sec, 200);
    }

    #[test]
    fn test_top_artists_with_stats_multiple_artists() {
        let conn = setup_memory_db();
        // Track 1 with "Test Artist"
        let t1 = insert_basic_track(&conn);
        // Track 2 with a different artist
        let t2_path = "/music/other.mp3";
        let t2 = update_track(
            &conn,
            t2_path,
            "Other Song",
            200,
            Some(2024),
            2000,
            6000,
            None,
            None,
            None,
            0,
            None,
            0,
            "",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let artist2_id = get_or_create_artist(&conn, "Second Artist").unwrap();
        set_track_artists(&conn, t2, &[artist2_id]).unwrap();
        let album_id = get_or_create_album(&conn, "Other Album", None, None).unwrap();
        set_track_album(&conn, t2, album_id, 1).unwrap();

        insert_play_at(&conn, t1, "2024-01-15 10:00:00", "ALBUM", 100.0);
        insert_play_at(&conn, t2, "2024-01-15 10:00:00", "ALBUM", 100.0);
        let result =
            get_top_artists_with_stats(&conn, Timeframe::AllTime, 10, TopSort::default()).unwrap();
        assert_eq!(result.len(), 2);
        // Equal play counts tie-break by listening time, then name.
        assert_eq!(result[0].artist.name, "Second Artist");
        assert_eq!(result[1].artist.name, "Test Artist");
    }

    // -----------------------------------------------------------------------
    // get_top_albums_with_stats tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_top_albums_with_stats_empty() {
        let conn = setup_memory_db();
        let result =
            get_top_albums_with_stats(&conn, Timeframe::AllTime, 10, TopSort::default()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_top_albums_with_stats_single() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 50.0);
        let result =
            get_top_albums_with_stats(&conn, Timeframe::AllTime, 10, TopSort::default()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].album.name, "Test Album");
        assert_eq!(result[0].play_count, 1);
        assert_eq!(result[0].total_listening_time_sec, 100);
        assert_eq!(result[0].tracks_played, 1);
    }

    #[test]
    fn test_top_albums_with_stats_multiple_albums() {
        let conn = setup_memory_db();
        let t1 = insert_basic_track(&conn); // "Test Album"
        let t2_path = "/music/other.mp3";
        let t2 = update_track(
            &conn,
            t2_path,
            "Other Song",
            200,
            Some(2024),
            2000,
            6000,
            None,
            None,
            None,
            0,
            None,
            0,
            "",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let artist_id = get_or_create_artist(&conn, "Artist").unwrap();
        set_track_artists(&conn, t2, &[artist_id]).unwrap();
        let album2_id = get_or_create_album(&conn, "Other Album", None, None).unwrap();
        set_track_album(&conn, t2, album2_id, 1).unwrap();

        insert_play_at(&conn, t1, "2024-01-15 10:00:00", "ALBUM", 100.0);
        insert_play_at(&conn, t2, "2024-01-15 10:00:00", "ALBUM", 100.0);
        let result =
            get_top_albums_with_stats(&conn, Timeframe::AllTime, 10, TopSort::default()).unwrap();
        assert_eq!(result.len(), 2);
    }

    // -----------------------------------------------------------------------
    // get_top_genres_with_stats tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_top_genres_with_stats_empty() {
        let conn = setup_memory_db();
        let result =
            get_top_genres_with_stats(&conn, Timeframe::AllTime, 10, TopSort::default()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_top_genres_with_stats_basic() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        let gid = get_or_create_genre(&conn, "Rock").unwrap();
        bulk_insert_track_genres(&conn, &[(track_id, gid)]).unwrap();
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 100.0);
        let result =
            get_top_genres_with_stats(&conn, Timeframe::AllTime, 10, TopSort::default()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].genre.name, "Rock");
        assert_eq!(result[0].play_count, 1);
        assert_eq!(result[0].tracks_played, 1);
        assert_eq!(result[0].total_listening_time_sec, 200);
    }

    #[test]
    fn test_top_genres_with_stats_multi_genre_play_counts_once_per_genre() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        let gid1 = get_or_create_genre(&conn, "Rock").unwrap();
        let gid2 = get_or_create_genre(&conn, "Indie").unwrap();
        bulk_insert_track_genres(&conn, &[(track_id, gid1), (track_id, gid2)]).unwrap();
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 100.0);
        let result =
            get_top_genres_with_stats(&conn, Timeframe::AllTime, 10, TopSort::default()).unwrap();
        assert_eq!(result.len(), 2);
        // A play counts once per genre, matching the overview's per-row count.
        assert_eq!(result[0].play_count, 1);
        assert_eq!(result[1].play_count, 1);
    }

    #[test]
    fn test_top_genres_with_stats_sort_by_time() {
        let conn = setup_memory_db();
        let t1 = insert_basic_track(&conn); // 200s
        let t2 = update_track(
            &conn,
            "/music/other.mp3",
            "Other Song",
            100,
            Some(2024),
            2000,
            6000,
            None,
            None,
            None,
            0,
            None,
            0,
            "",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let rock = get_or_create_genre(&conn, "Rock").unwrap();
        let pop = get_or_create_genre(&conn, "Pop").unwrap();
        bulk_insert_track_genres(&conn, &[(t1, pop)]).unwrap();
        bulk_insert_track_genres(&conn, &[(t2, rock)]).unwrap();
        insert_play_at(&conn, t1, "2024-01-15 10:00:00", "ALBUM", 100.0); // 200s
        insert_play_at(&conn, t2, "2024-01-15 10:00:00", "ALBUM", 50.0); // 50s
        insert_play_at(&conn, t2, "2024-01-16 10:00:00", "ALBUM", 50.0); // 50s
        let by_plays =
            get_top_genres_with_stats(&conn, Timeframe::AllTime, 10, TopSort::Plays).unwrap();
        let by_time =
            get_top_genres_with_stats(&conn, Timeframe::AllTime, 10, TopSort::Time).unwrap();
        assert_eq!(by_plays[0].genre.name, "Rock");
        assert_eq!(by_plays[0].play_count, 2);
        assert_eq!(by_time[0].genre.name, "Pop");
        assert_eq!(by_time[0].total_listening_time_sec, 200);
    }

    // -----------------------------------------------------------------------
    // get_listening_time_trend tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_listening_time_trend_empty() {
        let conn = setup_memory_db();
        let result = get_listening_time_trend(&conn, Timeframe::AllTime).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_listening_time_trend_single_day() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 100.0);
        let result = get_listening_time_trend(&conn, Timeframe::AllTime).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].date, "2024-01-15");
        assert!((result[0].value - 200.0_f64 / 60.0).abs() < 0.001);
    }

    #[test]
    fn test_listening_time_trend_multiple_days() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 100.0);
        insert_play_at(&conn, track_id, "2024-01-16 10:00:00", "ALBUM", 100.0);
        let result = get_listening_time_trend(&conn, Timeframe::AllTime).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].date, "2024-01-15");
        assert_eq!(result[1].date, "2024-01-16");
    }

    // -----------------------------------------------------------------------
    // get_streak_data tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_streak_data_no_plays() {
        let conn = setup_memory_db();
        let result = get_streak_data(&conn, Timeframe::AllTime).unwrap();
        assert_eq!(result.current_streak, 0);
        assert_eq!(result.longest_streak, 0);
        assert!(result.streak_dates.is_empty());
        assert!(result.daily_counts.is_empty());
    }

    #[test]
    fn test_streak_data_single_day() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 100.0);
        let result = get_streak_data(&conn, Timeframe::AllTime).unwrap();
        assert_eq!(result.streak_dates.len(), 1);
        assert_eq!(result.streak_dates[0], "2024-01-15");
        assert_eq!(*result.daily_counts.get("2024-01-15").unwrap(), 1);
        assert_eq!(result.longest_streak, 1);
    }

    #[test]
    fn test_streak_data_consecutive_days() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 100.0);
        insert_play_at(&conn, track_id, "2024-01-16 10:00:00", "ALBUM", 100.0);
        insert_play_at(&conn, track_id, "2024-01-17 10:00:00", "ALBUM", 100.0);
        let result = get_streak_data(&conn, Timeframe::AllTime).unwrap();
        assert_eq!(result.streak_dates.len(), 3);
        assert_eq!(result.longest_streak, 3);
    }

    #[test]
    fn test_streak_data_gap_resets_streak() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 100.0);
        insert_play_at(&conn, track_id, "2024-01-16 10:00:00", "ALBUM", 100.0);
        insert_play_at(&conn, track_id, "2024-01-20 10:00:00", "ALBUM", 100.0);
        let result = get_streak_data(&conn, Timeframe::AllTime).unwrap();
        // Longest streak is 2 (Jan 15-16), not 3 (gap on 17-19)
        assert_eq!(result.longest_streak, 2);
    }

    #[test]
    fn test_streak_data_multiple_plays_same_day() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 100.0);
        insert_play_at(&conn, track_id, "2024-01-15 14:00:00", "ALBUM", 100.0);
        let result = get_streak_data(&conn, Timeframe::AllTime).unwrap();
        assert_eq!(result.streak_dates.len(), 1);
        assert_eq!(*result.daily_counts.get("2024-01-15").unwrap(), 2);
    }

    #[test]
    fn test_current_streak_allows_unplayed_today() {
        // Listened yesterday, not today: streak is still alive.
        let dates = vec!["2026-08-01".to_string(), "2026-08-02".to_string()];
        let today = chrono::NaiveDate::from_ymd_opt(2026, 8, 3).unwrap();
        assert_eq!(compute_current_streak(&dates, today), 2);
    }

    #[test]
    fn test_current_streak_does_not_skip_mid_streak_gap() {
        // Mon/Wed/Thu with today=Thu: Tuesday is missing, so the streak is
        // only Thu+Wed (2), not 3.
        let dates = vec![
            "2026-07-27".to_string(),
            "2026-07-29".to_string(),
            "2026-07-30".to_string(),
        ];
        let today = chrono::NaiveDate::from_ymd_opt(2026, 7, 30).unwrap();
        assert_eq!(compute_current_streak(&dates, today), 2);
    }

    #[test]
    fn test_current_streak_breaks_after_two_missed_days() {
        let dates = vec![
            "2026-07-20".to_string(),
            "2026-07-21".to_string(),
            "2026-07-22".to_string(),
        ];
        let today = chrono::NaiveDate::from_ymd_opt(2026, 7, 25).unwrap();
        assert_eq!(compute_current_streak(&dates, today), 0);
    }

    #[test]
    fn test_streak_data_uses_timeframe_dates() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        // One play inside the week, two consecutive just outside it.
        let now = chrono::Local::now().naive_local();
        let day = chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap();
        insert_play_at(
            &conn,
            track_id,
            &(now.date() - chrono::Duration::days(1))
                .and_time(day)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            "ALBUM",
            100.0,
        );
        insert_play_at(
            &conn,
            track_id,
            &(now.date() - chrono::Duration::days(10))
                .and_time(day)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            "ALBUM",
            100.0,
        );
        insert_play_at(
            &conn,
            track_id,
            &(now.date() - chrono::Duration::days(11))
                .and_time(day)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            "ALBUM",
            100.0,
        );
        // All-time: longest streak is 2 (the two consecutive older plays).
        let all = get_streak_data(&conn, Timeframe::AllTime).unwrap();
        assert_eq!(all.longest_streak, 2);
        // This week window: only the recent play counts; streaks come from the
        // same filtered set the calendar displays.
        let week = get_streak_data(&conn, Timeframe::ThisWeek).unwrap();
        assert_eq!(week.streak_dates.len(), 1);
        assert_eq!(week.longest_streak, 1);
    }

    // -----------------------------------------------------------------------
    // get_library_growth tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_library_growth_empty() {
        let conn = setup_memory_db();
        let result = get_library_growth(&conn).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_library_growth_with_tracks() {
        let conn = setup_memory_db();
        insert_basic_track(&conn);
        let result = get_library_growth(&conn).unwrap();
        assert!(!result.is_empty());
        assert!(result[0].tracks_added >= 1);
    }

    #[test]
    fn test_library_growth_multiple_tracks() {
        let conn = setup_memory_db();
        let t1 = insert_basic_track(&conn);
        conn.execute(
            "UPDATE track SET added_at = '2024-01-01 10:00:00' WHERE id = ?",
            params![t1],
        )
        .unwrap();
        let t2 = insert_basic_track_with_path(&conn, "/music/other.mp3", 2);
        conn.execute(
            "UPDATE track SET added_at = '2024-01-15 10:00:00' WHERE id = ?",
            params![t2],
        )
        .unwrap();
        let result = get_library_growth(&conn).unwrap();
        assert!(!result.is_empty());
        // The last entry should have cumulative tracks >= 2
        assert!(result.last().unwrap().tracks_added >= 2);
    }

    // -----------------------------------------------------------------------
    // get_heatmap_hourly tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_heatmap_hourly_empty() {
        let conn = setup_memory_db();
        let result = get_heatmap_hourly(&conn, Timeframe::AllTime).unwrap();
        assert_eq!(result.len(), 168);
        for cell in &result {
            assert_eq!(cell.value, 0);
        }
    }

    #[test]
    fn test_heatmap_hourly_with_plays() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 100.0);
        let result = get_heatmap_hourly(&conn, Timeframe::AllTime).unwrap();
        assert_eq!(result.len(), 168);
        let (wd, h) = local_hour_weekday("2024-01-15 10:00:00");
        assert_eq!(result[(wd * 24 + h) as usize].value, 1);
    }

    #[test]
    fn test_heatmap_hourly_multiple_hours() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        insert_play_at(&conn, track_id, "2024-01-15 08:00:00", "ALBUM", 100.0);
        insert_play_at(&conn, track_id, "2024-01-15 22:00:00", "ALBUM", 100.0);
        let result = get_heatmap_hourly(&conn, Timeframe::AllTime).unwrap();
        assert_eq!(result.len(), 168);
        let (wd1, h1) = local_hour_weekday("2024-01-15 08:00:00");
        let (wd2, h2) = local_hour_weekday("2024-01-15 22:00:00");
        assert_eq!(result[(wd1 * 24 + h1) as usize].value, 1);
        assert_eq!(result[(wd2 * 24 + h2) as usize].value, 1);
    }

    // -----------------------------------------------------------------------
    // get_heatmap_weekday tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_heatmap_weekday_empty() {
        let conn = setup_memory_db();
        let result = get_heatmap_weekday(&conn, Timeframe::AllTime).unwrap();
        assert_eq!(result.len(), 7);
        for cell in &result {
            assert_eq!(cell.value, 0);
        }
    }

    #[test]
    fn test_heatmap_weekday_with_plays() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 100.0);
        insert_play_at(&conn, track_id, "2024-01-17 10:00:00", "ALBUM", 100.0);
        let result = get_heatmap_weekday(&conn, Timeframe::AllTime).unwrap();
        assert_eq!(result.len(), 7);
        let (wd1, _) = local_hour_weekday("2024-01-15 10:00:00");
        let (wd2, _) = local_hour_weekday("2024-01-17 10:00:00");
        assert_eq!(result[wd1 as usize].value, 1);
        assert_eq!(result[wd2 as usize].value, 1);
    }

    // -----------------------------------------------------------------------
    // get_favorite_trends tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_favorite_trends_empty() {
        let conn = setup_memory_db();
        let result = get_favorite_trends(&conn, Timeframe::AllTime).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_favorite_trends_single_period() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 100.0);
        let result = get_favorite_trends(&conn, Timeframe::AllTime).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].period, "2024-01");
        assert_eq!(result[0].top_track_id, Some(track_id));
        assert_eq!(result[0].top_track_name.as_deref(), Some("Test Song"));
        assert_eq!(result[0].top_artist_name.as_deref(), Some("Test Artist"));
        assert_eq!(result[0].top_album_name.as_deref(), Some("Test Album"));
    }

    #[test]
    fn test_favorite_trends_two_months() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 100.0);
        insert_play_at(&conn, track_id, "2024-02-10 10:00:00", "ALBUM", 100.0);
        let result = get_favorite_trends(&conn, Timeframe::AllTime).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].period, "2024-01");
        assert_eq!(result[1].period, "2024-02");
    }

    #[test]
    fn test_favorite_trends_top_artist_and_album_are_true_tops() {
        let conn = setup_memory_db();
        // Top track (5 plays) belongs to ArtistX / AlbumX.
        let a = insert_basic_track(&conn);
        // Two tracks from ArtistY / AlbumY with 4 + 3 plays = 7 total.
        let b = update_track(
            &conn,
            "/music/b.mp3",
            "Song B",
            200,
            None,
            0,
            1000,
            None,
            None,
            None,
            0,
            None,
            0,
            "",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let c = update_track(
            &conn,
            "/music/c.mp3",
            "Song C",
            200,
            None,
            0,
            1000,
            None,
            None,
            None,
            0,
            None,
            0,
            "",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let artist_x = get_or_create_artist(&conn, "ArtistX").unwrap();
        let artist_y = get_or_create_artist(&conn, "ArtistY").unwrap();
        set_track_artists(&conn, a, &[artist_x]).unwrap();
        set_track_artists(&conn, b, &[artist_y]).unwrap();
        set_track_artists(&conn, c, &[artist_y]).unwrap();
        let album_x = get_or_create_album(&conn, "AlbumX", None, None).unwrap();
        let album_y = get_or_create_album(&conn, "AlbumY", None, None).unwrap();
        set_track_album(&conn, a, album_x, 1).unwrap();
        set_track_album(&conn, b, album_y, 1).unwrap();
        set_track_album(&conn, c, album_y, 2).unwrap();
        for _ in 0..5 {
            insert_play_at(&conn, a, "2024-01-15 10:00:00", "ALBUM", 100.0);
        }
        for _ in 0..4 {
            insert_play_at(&conn, b, "2024-01-16 10:00:00", "ALBUM", 100.0);
        }
        for _ in 0..3 {
            insert_play_at(&conn, c, "2024-01-17 10:00:00", "ALBUM", 100.0);
        }

        let result = get_favorite_trends(&conn, Timeframe::AllTime).unwrap();
        assert_eq!(result.len(), 1);
        // Top track is the 5-play track...
        assert_eq!(result[0].top_track_id, Some(a));
        // ...but top artist/album are the 7-play artist/album.
        assert_eq!(result[0].top_artist_name.as_deref(), Some("ArtistY"));
        assert_eq!(result[0].top_album_name.as_deref(), Some("AlbumY"));
    }

    // -----------------------------------------------------------------------
    // get_playback_history_timeline tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_playback_history_timeline_empty() {
        let conn = setup_memory_db();
        let result = get_playback_history_timeline(&conn, Timeframe::AllTime, 10).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_playback_history_timeline_ordering() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 100.0);
        insert_play_at(&conn, track_id, "2024-01-20 10:00:00", "ALBUM", 50.0);
        let result = get_playback_history_timeline(&conn, Timeframe::AllTime, 10).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result[0].played_at > result[1].played_at);
        assert!((result[0].completion_percent - 50.0).abs() < 0.001);
        assert!((result[1].completion_percent - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_playback_history_timeline_limit() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 100.0);
        insert_play_at(&conn, track_id, "2024-01-16 10:00:00", "ALBUM", 100.0);
        let result = get_playback_history_timeline(&conn, Timeframe::AllTime, 1).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_playback_history_timeline_source_type() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "PLAYLIST", 100.0);
        let result = get_playback_history_timeline(&conn, Timeframe::AllTime, 10).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].source_type, "playlist");
    }

    #[test]
    fn test_playback_history_timeline_multi_album_no_duplicates() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn);
        let album2 = get_or_create_album(&conn, "Compilation", None, None).unwrap();
        conn.execute(
            "INSERT INTO album_track (album_id, track_id, track_number) VALUES (?, ?, ?)",
            params![album2, track_id, 5],
        )
        .unwrap();
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 100.0);
        let result = get_playback_history_timeline(&conn, Timeframe::AllTime, 10).unwrap();
        assert_eq!(result.len(), 1);
    }

    // -----------------------------------------------------------------------
    // avg daily + quality aggregate tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_stats_overview_avg_daily_uses_window() {
        let conn = setup_memory_db();
        let track_id = insert_basic_track(&conn); // 200s
        // 5 plays over the last 5 days, each 100% = 1000s ≈ 16.67 min
        for day in 1..=5 {
            let date = chrono::Utc::now()
                .checked_sub_days(chrono::Days::new(day))
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            insert_play_at(&conn, track_id, &date, "ALBUM", 100.0);
        }
        // this_week window = 7 days => 1000 / 60 / 7 ≈ 2.38 min/day
        let week = get_stats_overview(&conn, Timeframe::ThisWeek).unwrap();
        assert_eq!(week.total_plays, 5);
        assert!((week.avg_daily_listening_min - 1000.0 / 60.0 / 7.0).abs() < 0.001);
        // today window = 1 day; all plays are from previous days.
        let today = get_stats_overview(&conn, Timeframe::Today).unwrap();
        assert_eq!(today.total_plays, 0);
    }

    #[test]
    fn test_stats_overview_quality_and_completion() {
        let conn = setup_memory_db();
        update_track(
            &conn,
            "/music/song.flac",
            "Song",
            200,
            None,
            0,
            5000,
            None,
            None,
            Some(1400),
            44100,
            Some(16),
            2,
            "flac",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let track_id = get_track_id_by_path(&conn, "/music/song.flac").unwrap();
        insert_play_at(&conn, track_id, "2024-01-15 10:00:00", "ALBUM", 100.0);
        insert_play_at(&conn, track_id, "2024-01-16 10:00:00", "ALBUM", 20.0);
        let overview = get_stats_overview(&conn, Timeframe::AllTime).unwrap();
        assert!((overview.avg_bitrate_kbps.unwrap() - 1.4).abs() < 0.001);
        assert_eq!(overview.avg_sample_rate.unwrap() as i64, 44100);
        assert_eq!(overview.avg_bit_depth.unwrap() as i64, 16);
        assert!((overview.avg_completion_pct.unwrap() - 60.0).abs() < 0.001);
        assert!((overview.skip_rate.unwrap() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_format_distribution_quality_aggregates() {
        let conn = setup_memory_db();
        update_track(
            &conn,
            "/music/a.flac",
            "A",
            100,
            None,
            0,
            1000,
            None,
            None,
            Some(1200),
            48000,
            Some(24),
            2,
            "flac",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        update_track(
            &conn,
            "/music/b.flac",
            "B",
            100,
            None,
            0,
            1000,
            None,
            None,
            Some(800),
            44100,
            Some(16),
            2,
            "flac",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let result = get_format_distribution(&conn).unwrap();
        assert_eq!(result.len(), 1);
        assert!((result[0].avg_bitrate_kbps.unwrap() - 1.0).abs() < 0.001);
        assert_eq!(result[0].avg_sample_rate.unwrap() as i64, 46050);
        assert_eq!(result[0].avg_bit_depth.unwrap() as i64, 20);
    }

    // -----------------------------------------------------------------------
    // get_format_distribution tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_format_distribution_empty() {
        let conn = setup_memory_db();
        let result = get_format_distribution(&conn).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_format_distribution_single_format() {
        let conn = setup_memory_db();
        update_track(
            &conn,
            "/music/song1.mp3",
            "Song 1",
            100,
            None,
            0,
            1000,
            None,
            None,
            None,
            0,
            None,
            0,
            "mp3",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        update_track(
            &conn,
            "/music/song2.mp3",
            "Song 2",
            200,
            None,
            0,
            2000,
            None,
            None,
            None,
            0,
            None,
            0,
            "mp3",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let result = get_format_distribution(&conn).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].format, "mp3");
        assert_eq!(result[0].count, 2);
        assert!((result[0].percentage - 100.0).abs() < 0.001);
        assert_eq!(result[0].total_bytes, 3000);
    }

    #[test]
    fn test_format_distribution_multiple_formats() {
        let conn = setup_memory_db();
        update_track(
            &conn,
            "/music/song1.mp3",
            "Song 1",
            100,
            None,
            0,
            1000,
            None,
            None,
            None,
            0,
            None,
            0,
            "mp3",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        update_track(
            &conn,
            "/music/song2.flac",
            "Song 2",
            200,
            None,
            0,
            2000,
            None,
            None,
            None,
            0,
            None,
            0,
            "flac",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let result = get_format_distribution(&conn).unwrap();
        assert_eq!(result.len(), 2);
        for stat in &result {
            assert_eq!(stat.count, 1);
            assert!((stat.percentage - 50.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_format_distribution_uppercase_extension() {
        let conn = setup_memory_db();
        update_track(
            &conn,
            "/music/song.MP3",
            "Song",
            100,
            None,
            0,
            1000,
            None,
            None,
            None,
            0,
            None,
            0,
            "mp3",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let result = get_format_distribution(&conn).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].format, "mp3");
    }
}
