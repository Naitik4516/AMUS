use chrono::{DateTime, Utc};
use crate::error::{Error, Result};
use crate::models::*;
use rusqlite::{Connection, OptionalExtension, Params, params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type DbPool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
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
    AllTime,
}

pub fn timeframe_where_clause(alias: &str, timeframe: Timeframe) -> String {
    match timeframe {
        Timeframe::Today => format!("{} >= datetime('now', 'start of day')", alias),
        Timeframe::ThisWeek => format!("{} >= datetime('now', '-7 days')", alias),
        Timeframe::ThisMonth => format!("{} >= datetime('now', '-30 days')", alias),
        Timeframe::Last3Months => format!("{} >= datetime('now', '-90 days')", alias),
        Timeframe::Last6Months => format!("{} >= datetime('now', '-180 days')", alias),
        Timeframe::ThisYear => format!("{} >= datetime('now', 'start of year')", alias),
        Timeframe::LastYear => format!("{} >= datetime('now', '-1 year')", alias),
        Timeframe::Last5Years => format!("{} >= datetime('now', '-5 years')", alias),
        Timeframe::AllTime => "1=1".to_string(),
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

pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "BEGIN;
        CREATE TABLE IF NOT EXISTS track (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            title TEXT NOT NULL,
            duration_sec INTEGER NOT NULL,
            cover_art TEXT,
            year INTEGER,
            is_favorite BOOLEAN DEFAULT FALSE,
            added_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            mtime INTEGER NOT NULL,
            file_size INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS artist (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            profile_image TEXT,
            banner_image TEXT
        );

        CREATE TABLE IF NOT EXISTS album (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            cover_art TEXT,
            year INTEGER
        );

        CREATE TABLE IF NOT EXISTS playlist (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Junction Tables
        CREATE TABLE IF NOT EXISTS track_artist (
            track_id INTEGER,
            artist_id INTEGER,
            PRIMARY KEY (track_id, artist_id),
            FOREIGN KEY (track_id) REFERENCES track(id) ON DELETE CASCADE,
            FOREIGN KEY (artist_id) REFERENCES artist(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS album_track (
            album_id INTEGER,
            track_id INTEGER,
            track_number INTEGER NOT NULL, -- <-- Added for Album Sequence
            PRIMARY KEY (album_id, track_id),
            FOREIGN KEY (album_id) REFERENCES album(id) ON DELETE CASCADE,
            FOREIGN KEY (track_id) REFERENCES track(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS playlist_track (
            playlist_id INTEGER,
            track_id INTEGER,
            position INTEGER NOT NULL, -- <-- Added for Custom Drag & Drop Order
            PRIMARY KEY (playlist_id, track_id),
            FOREIGN KEY (playlist_id) REFERENCES playlist(id) ON DELETE CASCADE,
            FOREIGN KEY (track_id) REFERENCES track(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS source_dirs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE
        );

        CREATE TABLE IF NOT EXISTS playback_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            track_id INTEGER NOT NULL,
            played_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            source_type TEXT CHECK(source_type IN ('ALBUM', 'PLAYLIST', 'ARTIST', 'FAVORITES', 'SEARCH', 'RECOMMENDATION', 'OTHER')),
            source_id INTEGER,
            completion_percent REAL CHECK(completion_percent >= 0 AND completion_percent <= 100),
            FOREIGN KEY(track_id) REFERENCES track(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS user_queue (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            track_id INTEGER NOT NULL,
            position INTEGER NOT NULL,
            added_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(track_id) REFERENCES track(id) ON DELETE CASCADE
        );

        CREATE VIEW IF NOT EXISTS track_stats AS
        SELECT
            t.id AS track_id,
            COUNT(ph.id) AS play_count,
            MAX(ph.played_at) AS last_played_at,
            SUM(CASE WHEN ph.completion_percent < 50 THEN 1 ELSE 0 END) AS skip_count,
            MAX(CASE WHEN ph.completion_percent < 50 THEN ph.played_at ELSE NULL END) AS last_skipped_at
        FROM track t
        LEFT JOIN playback_history ph ON t.id = ph.track_id
        GROUP BY t.id;

        COMMIT;",
    )
    .map_err(Error::Db)?;

    conn.execute_batch(
        "BEGIN;
        -- Search & Sorting
        CREATE INDEX IF NOT EXISTS idx_track_title ON track(title);
        CREATE INDEX IF NOT EXISTS idx_track_added_at ON track(added_at);
        CREATE INDEX IF NOT EXISTS idx_artist_name ON artist(name);
        CREATE INDEX IF NOT EXISTS idx_album_name ON album(name);

        -- Junction/Foreign Key Lookups
        CREATE INDEX IF NOT EXISTS idx_track_artist_artist_id ON track_artist(artist_id);
        CREATE INDEX IF NOT EXISTS idx_album_track_album_id ON album_track(album_id);
        CREATE INDEX IF NOT EXISTS idx_playlist_track_playlist_id ON playlist_track(playlist_id);

        -- Metrics & Timeline
        CREATE INDEX IF NOT EXISTS idx_playback_history_played_at ON playback_history(played_at);
        CREATE INDEX IF NOT EXISTS idx_playback_history_track_id ON playback_history(track_id);
        COMMIT;",
    )
    .map_err(Error::Db)?;

    // Migration: add file_size column if missing (pre-v2 databases)
    let _ = conn.execute_batch(
        "ALTER TABLE track ADD COLUMN file_size INTEGER DEFAULT 0;",
    );

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

    let path_pattern = format!("{}%", path);

    tx.execute("DELETE FROM track WHERE path LIKE ?", [path_pattern])
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
        let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");

        // Delete the tracks. Cascading foreign keys will clean up track_artist, album_track,
        // playlist_track, and playback_history.
        let sql = format!("DELETE FROM track WHERE path IN ({})", placeholders);
        let mut stmt = conn.prepare(&sql).map_err(Error::Db)?;
        let count = stmt
            .execute(rusqlite::params_from_iter(chunk))
            .map_err(Error::Db)?;
        total_deleted += count;
    }

    // Clean up orphan artists and albums
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

pub fn get_or_create_artist(conn: &Connection, name: &str) -> Result<i64> {
    conn.execute(
        "INSERT OR IGNORE INTO artist (name) VALUES (?)",
        params![name],
    )
    .map_err(Error::Db)?;

    conn.query_row(
        "SELECT id FROM artist WHERE name = ?",
        params![name],
        |row| row.get(0),
    )
    .map_err(Error::Db)
}

pub fn set_track_artists(conn: &Connection, track_id: i64, artist_ids: &[i64]) -> Result<()> {
    conn.execute(
        "DELETE FROM track_artist WHERE track_id = ?",
        params![track_id],
    )
    .map_err(Error::Db)?;

    for artist_id in artist_ids {
        conn.execute(
            "INSERT OR IGNORE INTO track_artist (track_id, artist_id) VALUES (?, ?)",
            params![track_id, artist_id],
        )
        .map_err(Error::Db)?;
    }
    Ok(())
}

pub fn update_artist_profile_image(conn: &Connection, artist_id: i64, path: &str) -> Result<()> {
    conn.execute(
        "UPDATE artist SET profile_image = ? WHERE id = ? AND profile_image IS NULL",
        params![path, artist_id],
    )
    .map_err(Error::Db)?;
    Ok(())
}

pub fn update_artist_banner_image(conn: &Connection, artist_id: i64, path: &str) -> Result<()> {
    conn.execute(
        "UPDATE artist SET banner_image = ? WHERE id = ? AND banner_image IS NULL",
        params![path, artist_id],
    )
    .map_err(Error::Db)?;
    Ok(())
}

pub fn artist_has_photo(conn: &Connection, artist_id: i64) -> Result<bool> {
    let res: Option<String> = conn
        .query_row(
            "SELECT profile_image FROM artist WHERE id = ?",
            params![artist_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(Error::Db)?;
    Ok(res.is_some())
}

pub fn artist_has_banner(conn: &Connection, artist_id: i64) -> Result<bool> {
    let res: Option<String> = conn
        .query_row(
            "SELECT banner_image FROM artist WHERE id = ?",
            params![artist_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(Error::Db)?;
    Ok(res.is_some())
}

pub fn get_or_create_album(
    conn: &Connection,
    name: &str,
    cover_art: Option<&str>,
    year: Option<i32>,
) -> Result<i64> {
    conn.execute(
        "INSERT OR IGNORE INTO album (name, cover_art, year) VALUES (?, ?, ?)",
        params![name, cover_art, year],
    )
    .map_err(Error::Db)?;

    // if let Some(art) = cover_art {
    //     conn.execute(
    //         "UPDATE album SET cover_art = ? WHERE name = ? AND cover_art IS NULL",
    //         params![art, name],
    //     )
    //     .map_err(Error::Db)?;
    // }

    let album_id: i64 = conn
        .query_row(
            "SELECT id FROM album WHERE name = ?",
            params![name],
            |row| row.get(0),
        )
        .map_err(Error::Db)?;

    Ok(album_id)
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
) -> Result<i64> {
    conn.execute(
        "INSERT INTO track (path, title, duration_sec, year, mtime, file_size, cover_art)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(path) DO UPDATE SET
        title = excluded.title,
        duration_sec = excluded.duration_sec,
        year = excluded.year,
        mtime = excluded.mtime,
        file_size = excluded.file_size,
        cover_art = excluded.cover_art",
        params![path, title, duration_sec, year, mtime, file_size, cover_art],
    )
    .map_err(Error::Db)?;

    conn.query_row(
        "SELECT id FROM track WHERE path = ?",
        params![path],
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
    conn.execute(
        "DELETE FROM album_track WHERE track_id = ?",
        params![track_id],
    )
    .map_err(Error::Db)?;

    conn.execute(
        "INSERT OR IGNORE INTO album_track (album_id, track_id, track_number) VALUES (?, ?, ?)",
        params![album_id, track_id, track_number],
    )
    .map_err(Error::Db)?;
    Ok(())
}

pub fn get_track_mtime(conn: &Connection, path: &str) -> Result<Option<i64>> {
    conn.query_row(
        "SELECT mtime FROM track WHERE path = ?",
        params![path],
        |row| row.get(0),
    )
    .optional()
    .map_err(Error::Db)
}

pub fn get_track_id_by_path(conn: &Connection, path: &str) -> Result<i64> {
    conn.query_row(
        "SELECT id FROM track WHERE path = ?",
        params![path],
        |row| row.get(0),
    )
    .map_err(Error::Db)
}

pub fn toggle_favorite(conn: &Connection, track_id: i64) -> Result<bool> {
    conn.query_row(
        "UPDATE track SET is_favorite = NOT is_favorite WHERE id = ? RETURNING is_favorite",
        params![track_id],
        |row| row.get(0),
    )
    .map_err(Error::Db)
}

pub fn record_playback(
    conn: &Connection,
    track_id: i64,
    source_type: &str,
    completion_percent: f64,
) -> Result<()> {
    let clamped = completion_percent.clamp(0.0, 100.0);
    conn.execute(
        "INSERT INTO playback_history (track_id, source_type, completion_percent) VALUES (?, ?, ?)",
        params![track_id, source_type, clamped],
    )
    .map_err(Error::Db)?;
    Ok(())
}

pub fn get_all_albums(conn: &Connection) -> Result<Vec<Album>> {
    let sql = "SELECT id, name, cover_art
        FROM album
        ORDER BY name COLLATE NOCASE ASC";

    let mut stmt = conn.prepare(sql).map_err(Error::Db)?;
    let album_iter = stmt
        .query_map([], |row| {
            Ok(Album {
                id: row.get(0)?,
                name: row.get(1)?,
                cover_art: row.get(2)?,
            })
        })
        .map_err(Error::Db)?;

    album_iter
        .collect::<rusqlite::Result<Vec<_>>>()
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
            })
        },
    )
    .map_err(Error::Db)
}

pub fn get_album(conn: &Connection, album_id: i64) -> Result<Album> {
    conn.query_row(
        "SELECT id, name, cover_art FROM album WHERE id = ?",
        params![album_id],
        |row| {
            Ok(Album {
                id: row.get(0)?,
                name: row.get(1)?,
                cover_art: row.get(2)?,
            })
        },
    )
    .map_err(Error::Db)
}

pub fn get_all_playlists(conn: &Connection) -> Result<Vec<Playlist>> {
    let mut stmt = conn
        .prepare("SELECT id, name FROM playlist")
        .map_err(Error::Db)?;
    let playlist_iter = stmt
        .query_map([], |row| {
            Ok(Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .map_err(Error::Db)?;

    playlist_iter
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)
}

pub fn get_all_artists(conn: &Connection) -> Result<Vec<Artist>> {
    let mut stmt = conn
        .prepare("SELECT id, name, profile_image FROM artist")
        .map_err(Error::Db)?;
    let artist_iter = stmt
        .query_map([], |row| {
            Ok(Artist {
                id: row.get(0)?,
                name: row.get(1)?,
                profile_image: row.get(2)?,
                banner_image: None,
            })
        })
        .map_err(Error::Db)?;

    artist_iter
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)
}

pub fn get_albums_by_artist(conn: &Connection, artist_id: i64) -> Result<Vec<Album>> {
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT al.id, al.name, al.cover_art
             FROM album al
             JOIN album_track at2 ON al.id = at2.album_id
             JOIN track_artist ta ON at2.track_id = ta.track_id
             WHERE ta.artist_id = ?",
        )
        .map_err(Error::Db)?;
    let album_iter = stmt
        .query_map(params![artist_id], |row| {
            Ok(Album {
                id: row.get(0)?,
                name: row.get(1)?,
                cover_art: row.get(2)?,
            })
        })
        .map_err(Error::Db)?;

    album_iter
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)
}

pub fn create_playlist(conn: &Connection, name: &str) -> Result<()> {
    conn.execute("INSERT INTO playlist (name) VALUES (?)", params![name])
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

pub fn get_tracks_in_playlist(
    conn: &Connection,
    playlist_id: i64,
    sort_by: Option<SortBy>,
) -> Result<Vec<Track>> {
    let sql =
            "SELECT t.id, t.title, al.id, al.name, al.cover_art, NULL, NULL, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        JOIN playlist_track pt ON t.id = pt.track_id
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        WHERE pt.playlist_id = ?";

    prepare_sorted_tracks_list(conn, sql, params![playlist_id], sort_by)
}

pub fn get_tracks_by_artist(
    conn: &Connection,
    artist_id: i64,
    sort_by: Option<SortBy>,
) -> Result<Vec<Track>> {
    let sql =
            "SELECT t.id, t.title, al.id, al.name, al.cover_art, NULL, NULL, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        JOIN track_artist ta ON t.id = ta.track_id
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        WHERE ta.artist_id = ?";

    prepare_sorted_tracks_list(conn, sql, params![artist_id], sort_by)
}

pub fn get_tracks_by_album(
    conn: &Connection,
    album_id: i64,
    sort_by: Option<SortBy>,
) -> Result<Vec<Track>> {
    let sql =
            "SELECT t.id, t.title, al.id, al.name, al.cover_art, NULL, NULL, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        WHERE alt.album_id = ?";

    prepare_sorted_tracks_list(conn, sql, params![album_id], sort_by)
}

pub fn get_favorite_tracks(conn: &Connection, sort_by: Option<SortBy>) -> Result<Vec<Track>> {
    let sql =
        "SELECT t.id, t.title, al.id, al.name, al.cover_art, NULL, NULL, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        WHERE t.is_favorite = 1";

    prepare_sorted_tracks_list(conn, sql, [], sort_by)
}

pub fn get_recently_played_tracks(conn: &Connection, limit: usize) -> Result<Vec<Track>> {
    let sql =
        "SELECT t.id, t.title, al.id, al.name, al.cover_art, NULL, NULL, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        JOIN track_stats s ON t.id = s.track_id
        WHERE s.last_played_at IS NOT NULL
        ORDER BY s.last_played_at DESC
        LIMIT ?";

    prepare_tracks_list(conn, sql, params![limit])
}

pub fn get_most_played_tracks(
    conn: &Connection,
    limit: usize,
    timeframe: Timeframe,
) -> Result<Vec<Track>> {
    let time_filter = timeframe_where_clause("s.last_played_at", timeframe);

    let sql = format!(
        "SELECT t.id, t.title, al.id, al.name, al.cover_art, NULL, NULL, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        JOIN track_stats s ON t.id = s.track_id
        WHERE s.play_count > 0 AND {}
        ORDER BY s.play_count DESC
        LIMIT ?",
        time_filter
    );

    prepare_tracks_list(conn, &sql, params![limit])
}

pub fn get_top_artists(conn: &Connection, limit: usize) -> Result<Vec<Artist>> {
    let sql = "SELECT ar.id, ar.name, ar.profile_image, SUM(IFNULL(s.play_count, 0)) as total_plays
        FROM artist ar
        JOIN track_artist ta ON ta.artist_id = ar.id
        JOIN track t ON t.id = ta.track_id
        LEFT JOIN track_stats s ON t.id = s.track_id
        GROUP BY ar.id
        ORDER BY total_plays DESC
        LIMIT ?";

    let mut stmt = conn.prepare(&sql).map_err(Error::Db)?;
    let artist_iter = stmt
        .query_map(params![limit], |row| {
            Ok(Artist {
                id: row.get(0)?,
                name: row.get(1)?,
                profile_image: row.get(2)?,
                banner_image: None,
            })
        })
        .map_err(Error::Db)?;

    artist_iter
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)
}

pub fn get_top_albums(conn: &Connection, limit: usize) -> Result<Vec<Album>> {
    let sql = "SELECT al.id, al.name, al.cover_art, SUM(IFNULL(s.play_count, 0)) as total_plays
        FROM album al
        JOIN album_track alt ON alt.album_id = al.id
        JOIN track t ON t.id = alt.track_id
        LEFT JOIN track_stats s ON t.id = s.track_id
        GROUP BY al.id
        ORDER BY total_plays DESC
        LIMIT ?";

    let mut stmt = conn.prepare(&sql).map_err(Error::Db)?;
    let album_iter = stmt
        .query_map(params![limit], |row| {
            Ok(Album {
                id: row.get(0)?,
                name: row.get(1)?,
                cover_art: row.get(2)?,
            })
        })
        .map_err(Error::Db)?;

    album_iter
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)
}

pub fn get_forgotten_tracks(conn: &Connection, limit: usize) -> Result<Vec<Track>> {
    let sql =
        "SELECT t.id, t.title, al.id, al.name, al.cover_art, NULL, NULL, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        JOIN track_stats s ON t.id = s.track_id
        WHERE s.play_count > 0 AND s.last_played_at <= datetime('now', '-180 days')
        ORDER BY s.last_played_at ASC
        LIMIT ?";

    prepare_tracks_list(conn, sql, params![limit])
}

pub fn get_unplayed_tracks(conn: &Connection, limit: usize) -> Result<Vec<Track>> {
    let sql =
        "SELECT t.id, t.title, al.id, al.name, al.cover_art, NULL, NULL, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        LEFT JOIN track_stats s ON t.id = s.track_id
        WHERE s.play_count IS NULL OR s.play_count = 0
        ORDER BY t.added_at DESC
        LIMIT ?";

    prepare_tracks_list(conn, sql, params![limit])
}

pub fn get_recently_added_tracks(conn: &Connection, limit: usize) -> Result<Vec<Track>> {
    let sql =
        "SELECT t.id, t.title, al.id, al.name, al.cover_art, NULL, NULL, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        ORDER BY t.added_at DESC
        LIMIT ?";

    prepare_tracks_list(conn, sql, params![limit])
}

pub fn get_track_details(conn: &Connection, track_id: i64) -> Result<TrackDetails> {
    let sql = "SELECT t.id, t.path, t.title, al.id, al.name, al.cover_art, NULL, NULL, t.duration_sec, t.is_favorite, t.mtime,
        IFNULL(s.play_count, 0), s.last_played_at, IFNULL(s.skip_count, 0), s.last_skipped_at, t.cover_art, t.added_at
        FROM track t
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        LEFT JOIN track_stats s ON t.id = s.track_id
        WHERE t.id = ?";

    let result = conn
        .query_row(sql, params![track_id], |row| {
            let album_id: Option<i64> = row.get(3)?;
            let album_title: Option<String> = row.get(4)?;
            let album_art: Option<String> = row.get(5)?;
            let album = Album {
                id: album_id.unwrap_or(0),
                name: album_title.unwrap_or_else(|| "Unknown Album".to_string()),
                cover_art: album_art,
            };

            Ok(TrackDetails {
                id: row.get(0)?,
                path: row.get(1)?,
                title: row.get(2)?,
                artists: vec![],
                album,
                duration_seconds: row.get(8)?,
                is_favorite: row.get(9)?,
                mtime: row.get(10)?,
                play_count: row.get(11)?,
                last_played_at: row.get(12)?,
                skipped_count: row.get(13)?,
                last_skipped_at: row.get(14)?,
                cover_art: row.get(15)?,
                added_at: row.get(16)?,
            })
        })
        .map_err(Error::Db)?;

    let mut details = result;
    let artists_map = get_artists_for_tracks(conn, &[track_id])?;
    if let Some(artists) = artists_map.get(&track_id) {
        details.artists = artists.clone();
    }

    Ok(details)
}

pub fn search_tracks(conn: &Connection, query: &str, limit: usize) -> Result<Vec<Track>> {
    let pattern = format!("%{}%", query);

    let sql = r#"
        SELECT
            t.id,
            t.title,
            al.id,
            al.name,
            al.cover_art,
            NULL,
            NULL,
            t.duration_sec,
            t.is_favorite,
            t.cover_art,
            t.added_at
        FROM track t
        LEFT JOIN track_artist ta ON t.id = ta.track_id
        LEFT JOIN artist ar ON ta.artist_id = ar.id
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON al.id = alt.album_id
        LEFT JOIN track_stats s ON s.track_id = t.id
        WHERE t.title LIKE ? OR ar.name LIKE ? OR al.name LIKE ?
        GROUP BY t.id
        ORDER BY
            (CASE WHEN t.title LIKE ? THEN 3 ELSE 0 END) +
            (CASE WHEN ar.name LIKE ? THEN 2 ELSE 0 END) +
            (CASE WHEN al.name LIKE ? THEN 2 ELSE 0 END) DESC,
            IFNULL(s.play_count, 0) DESC
        LIMIT ?
        "#;

    prepare_tracks_list(
        conn,
        sql,
        params![pattern, pattern, pattern, pattern, pattern, pattern, limit],
    )
}

pub fn get_similar_tracks(
    conn: &Connection,
    current_track_id: i64,
    limit: usize,
) -> Result<Vec<Track>> {
    let sql = r#"
    SELECT t.id, t.title, al.id, al.name, al.cover_art, NULL, NULL, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
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
                THEN 100
                ELSE 0
            END) +

            -- Randomness factor to add some variety
            (ABS(RANDOM() % 11))
        ) DESC
    LIMIT ?
    "#;
    prepare_tracks_list(conn, sql, params![current_track_id, limit])
}

pub fn get_all_tracks(conn: &Connection, sort_by: Option<SortBy>) -> Result<Vec<Track>> {
    let sql =
        "SELECT t.id, t.title, al.id, al.name, al.cover_art, NULL, NULL, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id";

    prepare_sorted_tracks_list(conn, sql, [], sort_by)
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

pub fn rename_playlist(conn: &Connection, playlist_id: i64, new_name: &str) -> Result<()> {
    conn.execute(
        "UPDATE playlist SET name = ? WHERE id = ?",
        params![new_name, playlist_id],
    )
    .map_err(Error::Db)?;
    Ok(())
}

pub fn get_playlist_cover_arts(conn: &Connection, playlist_id: i64) -> Result<Vec<String>> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT DISTINCT t.cover_art
            FROM track t
            JOIN playlist_track pt ON t.id = pt.track_id
            WHERE pt.playlist_id = ? AND t.cover_art IS NOT NULL
            ORDER BY pt.position ASC
            LIMIT 4
            "#,
        )
        .map_err(Error::Db)?;

    let art_iter = stmt
        .query_map(params![playlist_id], |row| row.get(0))
        .map_err(Error::Db)?;

    art_iter
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)
}

// Utils

fn prepare_tracks_list<P: Params>(conn: &Connection, sql: &str, params: P) -> Result<Vec<Track>> {
    let mut stmt = conn.prepare(sql).map_err(Error::Db)?;

    let rows = stmt
        .query_map(params, |row| {
            let album_id: Option<i64> = row.get(2)?;
            let album_title: Option<String> = row.get(3)?;
            let album_art: Option<String> = row.get(4)?;
            let album = Album {
                id: album_id.unwrap_or(0),
                name: album_title.unwrap_or_else(|| "Unknown Album".to_string()),
                cover_art: album_art,
            };

            Ok(Track {
                id: row.get(0)?,
                title: row.get(1)?,
                artists: vec![],
                album,
                duration_seconds: row.get(7)?,
                is_favorite: row.get(8)?,
                cover_art: row.get(9)?,
                added_at: row.get(10)?,
            })
        })
        .map_err(Error::Db)?;

    let mut tracks: Vec<Track> = rows
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?;

    if !tracks.is_empty() {
        let track_ids: Vec<i64> = tracks.iter().map(|t| t.id).collect();
        let artists_map = get_artists_for_tracks(conn, &track_ids)?;
        for track in &mut tracks {
            if let Some(artists) = artists_map.get(&track.id) {
                track.artists = artists.clone();
            }
        }
    }

    Ok(tracks)
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
        let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT ta.track_id, ar.id, ar.name, ar.profile_image
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
                        banner_image: None,
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

pub fn save_user_queue(conn: &mut Connection, track_ids: &[i64]) -> Result<()> {
    let tx = conn.transaction().map_err(Error::Db)?;
    tx.execute("DELETE FROM user_queue", []).map_err(Error::Db)?;
    for (i, id) in track_ids.iter().enumerate() {
        tx.execute(
            "INSERT INTO user_queue (track_id, position) VALUES (?, ?)",
            params![id, i],
        )
        .map_err(Error::Db)?;
    }
    tx.commit().map_err(Error::Db)?;
    Ok(())
}

pub fn load_user_queue(conn: &Connection) -> Result<Vec<i64>> {
    let mut stmt = conn
        .prepare("SELECT track_id FROM user_queue ORDER BY position ASC")
        .map_err(Error::Db)?;
    let ids = stmt
        .query_map([], |row| row.get::<_, i64>(0))
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?;
    Ok(ids)
}

// ---------------------------------------------------------------------------
// Stats queries
// ---------------------------------------------------------------------------

pub fn get_stats_overview(conn: &Connection, timeframe: Timeframe) -> Result<StatsOverview> {
    let time_filter = timeframe_where_clause("ph.played_at", timeframe);

    let total_tracks: i64 = conn
        .query_row("SELECT COUNT(*) FROM track", [], |row| row.get(0))
        .map_err(Error::Db)?;

    let total_artists: i64 = conn
        .query_row("SELECT COUNT(*) FROM artist", [], |row| row.get(0))
        .map_err(Error::Db)?;

    let total_albums: i64 = conn
        .query_row("SELECT COUNT(*) FROM album", [], |row| row.get(0))
        .map_err(Error::Db)?;

    let (total_plays, total_time): (i64, f64) = conn
        .query_row(
            &format!(
                "SELECT COALESCE(COUNT(ph.id), 0), COALESCE(SUM(t.duration_sec * ph.completion_percent / 100.0), 0)
                 FROM playback_history ph
                 JOIN track t ON t.id = ph.track_id
                 WHERE {}",
                time_filter
            ),
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(Error::Db)?;

    let total_time_sec = total_time as i64;

    let avg_daily: f64 = if total_time > 0.0 {
        // Get earliest play date in range
        let days: f64 = conn
            .query_row(
                &format!(
                    "SELECT COALESCE(CAST(JULIANDAY('now') - JULIANDAY(MIN(ph.played_at)) AS REAL), 1)
                     FROM playback_history ph
                     WHERE {}",
                    time_filter
                ),
                [],
                |row| row.get(0),
            )
            .map_err(Error::Db)?;
        let days = days.max(1.0);
        total_time / 60.0 / days
    } else {
        0.0
    };

    let total_file_size: i64 = conn
        .query_row("SELECT COALESCE(SUM(file_size), 0) FROM track", [], |row| row.get(0))
        .map_err(Error::Db)?;

    let largest_file: f64 = conn
        .query_row(
            "SELECT COALESCE(MAX(file_size), 0) FROM track",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map_err(Error::Db)? as f64
        / 1048576.0;

    let avg_file_size: f64 = if total_tracks > 0 {
        total_file_size as f64 / total_tracks as f64 / 1048576.0
    } else {
        0.0
    };

    let played_tracks: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT ph.track_id) FROM playback_history ph",
            [],
            |row| row.get(0),
        )
        .map_err(Error::Db)?;

    let pct_played = if total_tracks > 0 {
        played_tracks as f64 / total_tracks as f64 * 100.0
    } else {
        0.0
    };

    let unplayed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM track t LEFT JOIN track_stats s ON t.id = s.track_id WHERE s.play_count IS NULL OR s.play_count = 0",
            [],
            |row| row.get(0),
        )
        .map_err(Error::Db)?;

    let format_dist = get_format_distribution(conn)?;

    Ok(StatsOverview {
        total_tracks,
        total_artists,
        total_albums,
        total_plays,
        total_listening_time_sec: total_time_sec,
        avg_daily_listening_min: avg_daily,
        total_file_size_bytes: total_file_size,
        avg_file_size_mb: avg_file_size,
        largest_file_mb: largest_file,
        format_distribution: format_dist,
        pct_library_played: pct_played,
        unplayed_tracks: unplayed,
    })
}

pub fn get_format_distribution(conn: &Connection) -> Result<Vec<FormatStat>> {
    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM track", [], |row| row.get(0))
        .map_err(Error::Db)?;

    let mut stmt = conn
        .prepare(
            "SELECT
                LOWER(
                    CASE
                        WHEN instr(path, '.') > 0 THEN substr(path, instr(path, '.') + 1)
                        ELSE 'unknown'
                    END
                ) as ext,
                COUNT(*) as cnt,
                COALESCE(SUM(file_size), 0) as total_bytes
             FROM track
             GROUP BY ext
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
            })
        })
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?;

    Ok(stats)
}

pub fn get_top_tracks_with_stats(
    conn: &Connection,
    timeframe: Timeframe,
    limit: usize,
) -> Result<Vec<TopTrack>> {
    let time_filter = timeframe_where_clause("ph.played_at", timeframe);

    let mut stmt = conn
        .prepare(&format!(
            "SELECT t.id, t.title, al.id, al.name, al.cover_art, NULL, NULL,
                    t.duration_sec, t.is_favorite, t.cover_art, t.added_at,
                    COUNT(ph.id) as play_count,
                    COALESCE(SUM(t.duration_sec * ph.completion_percent / 100.0), 0) as total_time,
                    MAX(ph.played_at) as last_played
             FROM playback_history ph
             JOIN track t ON t.id = ph.track_id
             LEFT JOIN album_track alt ON t.id = alt.track_id
             LEFT JOIN album al ON alt.album_id = al.id
             WHERE {}
             GROUP BY t.id
             ORDER BY play_count DESC
             LIMIT ?",
            time_filter
        ))
        .map_err(Error::Db)?;

    let mut top_tracks = Vec::new();
    let rows = stmt
        .query_map(params![limit], |row| {
            let album_id: Option<i64> = row.get(2)?;
            let album_title: Option<String> = row.get(3)?;
            let album_art: Option<String> = row.get(4)?;

            let track = Track {
                id: row.get(0)?,
                title: row.get(1)?,
                artists: vec![],
                album: Album {
                    id: album_id.unwrap_or(0),
                    name: album_title.unwrap_or_else(|| "Unknown Album".to_string()),
                    cover_art: album_art,
                },
                duration_seconds: row.get(7)?,
                is_favorite: row.get(8)?,
                cover_art: row.get(9)?,
                added_at: row.get(10)?,
            };

            let play_count: i64 = row.get(11)?;
            let total_listening_time_sec: f64 = row.get(12)?;
            let last_played_at: Option<DateTime<Utc>> = row.get(13)?;

            Ok(TopTrack {
                track,
                play_count,
                total_listening_time_sec: total_listening_time_sec as i64,
                last_played_at,
            })
        })
        .map_err(Error::Db)?;

    for row in rows {
        top_tracks.push(row.map_err(Error::Db)?);
    }

    // Attach artists
    if !top_tracks.is_empty() {
        let track_ids: Vec<i64> = top_tracks.iter().map(|t| t.track.id).collect();
        let artists_map = get_artists_for_tracks(conn, &track_ids)?;
        for tt in &mut top_tracks {
            if let Some(artists) = artists_map.get(&tt.track.id) {
                tt.track.artists = artists.clone();
            }
        }
    }

    Ok(top_tracks)
}

pub fn get_top_artists_with_stats(
    conn: &Connection,
    timeframe: Timeframe,
    limit: usize,
) -> Result<Vec<TopArtist>> {
    let time_filter = timeframe_where_clause("ph.played_at", timeframe);

    let mut stmt = conn
        .prepare(&format!(
            "SELECT ar.id, ar.name, ar.profile_image,
                    COUNT(DISTINCT ph.id) as play_count,
                    COALESCE(SUM(t.duration_sec * ph.completion_percent / 100.0), 0) as total_time,
                    COUNT(DISTINCT ph.track_id) as tracks_played
             FROM playback_history ph
             JOIN track t ON t.id = ph.track_id
             JOIN track_artist ta ON ta.track_id = t.id
             JOIN artist ar ON ar.id = ta.artist_id
             WHERE {}
             GROUP BY ar.id
             ORDER BY play_count DESC
             LIMIT ?",
            time_filter
        ))
        .map_err(Error::Db)?;

    let rows = stmt
        .query_map(params![limit], |row| {
            Ok(TopArtist {
                artist: Artist {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    profile_image: row.get(2)?,
                    banner_image: None,
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

pub fn get_top_albums_with_stats(
    conn: &Connection,
    timeframe: Timeframe,
    limit: usize,
) -> Result<Vec<TopAlbum>> {
    let time_filter = timeframe_where_clause("ph.played_at", timeframe);

    let mut stmt = conn
        .prepare(&format!(
            "SELECT al.id, al.name, al.cover_art,
                    COUNT(DISTINCT ph.id) as play_count,
                    COALESCE(SUM(t.duration_sec * ph.completion_percent / 100.0), 0) as total_time,
                    COUNT(DISTINCT ph.track_id) as tracks_played
             FROM playback_history ph
             JOIN track t ON t.id = ph.track_id
             JOIN album_track alt ON alt.track_id = t.id
             JOIN album al ON al.id = alt.album_id
             WHERE {}
             GROUP BY al.id
             ORDER BY play_count DESC
             LIMIT ?",
            time_filter
        ))
        .map_err(Error::Db)?;

    let rows = stmt
        .query_map(params![limit], |row| {
            Ok(TopAlbum {
                album: Album {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    cover_art: row.get(2)?,
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
            "SELECT date(ph.played_at) as day,
                    COALESCE(SUM(t.duration_sec * ph.completion_percent / 100.0 / 60.0), 0) as minutes
             FROM playback_history ph
             JOIN track t ON t.id = ph.track_id
             WHERE {}
             GROUP BY date(ph.played_at)
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
            "SELECT DISTINCT date(played_at) as play_date,
                    (SELECT COUNT(*) FROM playback_history ph2 WHERE date(ph2.played_at) = date(ph.played_at)) as count
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

    // Also fetch all dates for all-time streaks
    let mut all_stmt = conn
        .prepare(
            "SELECT DISTINCT date(played_at) FROM playback_history ORDER BY played_at ASC",
        )
        .map_err(Error::Db)?;

    let all_dates: Vec<String> = all_stmt
        .query_map([], |row| row.get(0))
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?;

    // Compute longest streak from all dates
    let longest = compute_longest_streak(&all_dates);

    // Compute current streak from all dates
    let today = chrono::Utc::now().date_naive();
    let current = compute_current_streak(&all_dates, today);

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
            if date == expected || date == expected - chrono::Duration::days(1) {
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

pub fn get_library_growth(conn: &Connection, timeframe: Timeframe) -> Result<Vec<GrowthPoint>> {
    let time_filter = timeframe_where_clause("t.added_at", timeframe);
    // Use monthly grouping for growth
    let period_expr = "strftime('%Y-%m', t.added_at)";

    let mut stmt = conn
        .prepare(&format!(
            "SELECT {pe} as period,
                    COUNT(*) as tracks
             FROM track t
             WHERE {}
             GROUP BY period
             ORDER BY period ASC",
            time_filter,
            pe = period_expr
        ))
        .map_err(Error::Db)?;

    let track_growth: HashMap<String, i64> = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?
        .into_iter()
        .collect();

    // Artist growth: count artists whose earliest track appeared in this period
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {pe} as period,
                    COUNT(DISTINCT ar.id) as artists
             FROM artist ar
             JOIN track_artist ta ON ta.artist_id = ar.id
             JOIN track t ON t.id = ta.track_id
             WHERE {}
             GROUP BY period
             ORDER BY period ASC",
            time_filter,
            pe = period_expr
        ))
        .map_err(Error::Db)?;

    let artist_growth: HashMap<String, i64> = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?
        .into_iter()
        .collect();

    // Album growth
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {pe} as period,
                    COUNT(DISTINCT al.id) as albums
             FROM album al
             JOIN album_track alt ON alt.album_id = al.id
             JOIN track t ON t.id = alt.track_id
             WHERE {}
             GROUP BY period
             ORDER BY period ASC",
            time_filter,
            pe = period_expr
        ))
        .map_err(Error::Db)?;

    let album_growth: HashMap<String, i64> = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?
        .into_iter()
        .collect();

    let mut periods: Vec<String> = track_growth.keys().cloned().collect();
    periods.extend(artist_growth.keys().cloned());
    periods.extend(album_growth.keys().cloned());
    periods.sort();
    periods.dedup();

    let mut result = Vec::new();
    let mut cum_tracks: i64 = 0;
    let mut cum_artists: i64 = 0;
    let mut cum_albums: i64 = 0;

    // Get counts before the timeframe start
    let cum_before_sql = format!(
        "SELECT
            (SELECT COUNT(*) FROM track WHERE added_at < (SELECT MIN(added_at) FROM track WHERE {})),
            (SELECT COUNT(DISTINCT ar.id) FROM artist ar JOIN track_artist ta ON ta.artist_id = ar.id JOIN track t ON t.id = ta.track_id WHERE t.added_at < (SELECT MIN(added_at) FROM track WHERE {})),
            (SELECT COUNT(DISTINCT al.id) FROM album al JOIN album_track alt ON alt.album_id = al.id JOIN track t ON t.id = alt.track_id WHERE t.added_at < (SELECT MIN(added_at) FROM track WHERE {}))",
        time_filter, time_filter, time_filter
    );

    let _ = conn.query_row(&cum_before_sql, [], |row| {
        cum_tracks = row.get::<_, Option<i64>>(0)?.unwrap_or(0);
        cum_artists = row.get::<_, Option<i64>>(1)?.unwrap_or(0);
        cum_albums = row.get::<_, Option<i64>>(2)?.unwrap_or(0);
        Ok::<_, rusqlite::Error>(())
    });

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
        "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday",
    ];

    let mut stmt = conn
        .prepare(&format!(
            "SELECT CAST(strftime('%w', played_at) AS INTEGER) as weekday,
                    CAST(strftime('%H', played_at) AS INTEGER) as hour,
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
        "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday",
    ];

    let mut stmt = conn
        .prepare(&format!(
            "SELECT CAST(strftime('%w', played_at) AS INTEGER) as weekday,
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

pub fn get_favorite_trends(
    conn: &Connection,
    timeframe: Timeframe,
) -> Result<Vec<FavoriteTrend>> {
    let time_filter = timeframe_where_clause("ph.played_at", timeframe);
    let period_expr = "strftime('%Y-%m', ph.played_at)";

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

    let mut result = Vec::new();
    let mut current_period = String::new();
    let mut top_track_in_period: Option<(i64, String)> = None;
    let mut top_artist_in_period: Option<(i64, String)> = None;
    let mut top_album_in_period: Option<(i64, String)> = None;

    for (period, track_id, track_name, _) in &rows {
        if *period != current_period {
            if !current_period.is_empty() {
                let (track_id_opt, track_name_opt) = top_track_in_period
                    .as_ref()
                    .map(|(id, n)| (*id, n.clone()))
                    .unzip();
                let (artist_id_opt, artist_name_opt) = top_artist_in_period
                    .as_ref()
                    .map(|(id, n)| (*id, n.clone()))
                    .unzip();
                let (album_id_opt, album_name_opt) = top_album_in_period
                    .as_ref()
                    .map(|(id, n)| (*id, n.clone()))
                    .unzip();

                result.push(FavoriteTrend {
                    period: current_period.clone(),
                    top_track_id: track_id_opt,
                    top_track_name: track_name_opt,
                    top_artist_id: artist_id_opt,
                    top_artist_name: artist_name_opt,
                    top_album_id: album_id_opt,
                    top_album_name: album_name_opt,
                });
            }
            current_period = period.clone();
            top_track_in_period = Some((*track_id, track_name.clone()));
            top_artist_in_period = None;
            top_album_in_period = None;
        }

        // Get artist info for this track to compute top artist
        if top_artist_in_period.is_none() {
            if let Ok(Some((artist_id, artist_name))) = conn
                .query_row(
                    "SELECT ar.id, ar.name FROM track_artist ta JOIN artist ar ON ar.id = ta.artist_id WHERE ta.track_id = ? LIMIT 1",
                    params![track_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()
            {
                top_artist_in_period = Some((artist_id, artist_name));
            }
        }

        // Get album info
        if top_album_in_period.is_none() {
            if let Ok(Some((album_id, album_name))) = conn
                .query_row(
                    "SELECT al.id, al.name FROM album_track alt JOIN album al ON al.id = alt.album_id WHERE alt.track_id = ? LIMIT 1",
                    params![track_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()
            {
                top_album_in_period = Some((album_id, album_name));
            }
        }
    }

    // Push last period
    if !current_period.is_empty() {
        let (track_id_opt, track_name_opt) = top_track_in_period
            .as_ref()
            .map(|(id, n)| (*id, n.clone()))
            .unzip();
        let (artist_id_opt, artist_name_opt) = top_artist_in_period
            .as_ref()
            .map(|(id, n)| (*id, n.clone()))
            .unzip();
        let (album_id_opt, album_name_opt) = top_album_in_period
            .as_ref()
            .map(|(id, n)| (*id, n.clone()))
            .unzip();

        result.push(FavoriteTrend {
            period: current_period,
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

pub fn get_playback_history_timeline(
    conn: &Connection,
    timeframe: Timeframe,
    limit: usize,
) -> Result<Vec<PlaybackEvent>> {
    let time_filter = timeframe_where_clause("ph.played_at", timeframe);

    let mut stmt = conn
        .prepare(&format!(
            "SELECT ph.played_at, t.id, t.title, al.id, al.name, al.cover_art, NULL, NULL,
                    t.duration_sec, t.is_favorite, t.cover_art, t.added_at,
                    ph.completion_percent, ph.source_type
             FROM playback_history ph
             JOIN track t ON t.id = ph.track_id
             LEFT JOIN album_track alt ON t.id = alt.track_id
             LEFT JOIN album al ON alt.album_id = al.id
             WHERE {}
             ORDER BY ph.played_at DESC
             LIMIT ?",
            time_filter
        ))
        .map_err(Error::Db)?;

    let mut events = Vec::new();
    let rows = stmt
        .query_map(params![limit], |row| {
            let album_id: Option<i64> = row.get(3)?;
            let album_title: Option<String> = row.get(4)?;
            let album_art: Option<String> = row.get(5)?;

            let track = Track {
                id: row.get(1)?,
                title: row.get(2)?,
                artists: vec![],
                album: Album {
                    id: album_id.unwrap_or(0),
                    name: album_title.unwrap_or_else(|| "Unknown Album".to_string()),
                    cover_art: album_art,
                },
                duration_seconds: row.get(8)?,
                is_favorite: row.get(9)?,
                cover_art: row.get(10)?,
                added_at: row.get(11)?,
            };

            Ok(PlaybackEvent {
                played_at: row.get(0)?,
                track,
                completion_percent: row.get(12)?,
                source_type: row.get::<_, String>(13)?.to_lowercase(),
            })
        })
        .map_err(Error::Db)?;

    for row in rows {
        events.push(row.map_err(Error::Db)?);
    }

    // Attach artists
    if !events.is_empty() {
        let track_ids: Vec<i64> = events.iter().map(|e| e.track.id).collect();
        let artists_map = get_artists_for_tracks(conn, &track_ids)?;
        for event in &mut events {
            if let Some(artists) = artists_map.get(&event.track.id) {
                event.track.artists = artists.clone();
            }
        }
    }

    Ok(events)
}
