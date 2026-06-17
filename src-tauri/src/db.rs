use crate::error::{Error, Result};
use crate::models::*;
use rusqlite::{Connection, OptionalExtension, Params, params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type DbPool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Timeframe {
    AllTime,
    ThisWeek,
    ThisMonth,
    Last6Months,
    ThisYear,
    Last3Years,
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
        CREATE TABLE IF NOT EXISTS source_dirs (
            id INTEGER PRIMARY KEY,
            path TEXT NOT NULL UNIQUE
        );
        CREATE TABLE IF NOT EXISTS artists (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE COLLATE NOCASE,
            profile_picture TEXT
        );
        CREATE TABLE IF NOT EXISTS albums (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL COLLATE NOCASE,
            artist_id INTEGER NOT NULL,
            cover_art TEXT,
            UNIQUE(title, artist_id),
            FOREIGN KEY(artist_id) REFERENCES artists(id)
        );
        CREATE TABLE IF NOT EXISTS genres (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE COLLATE NOCASE
        );
        CREATE TABLE IF NOT EXISTS tracks (
            id INTEGER PRIMARY KEY,
            path TEXT NOT NULL UNIQUE,
            title TEXT NOT NULL,
            album_id INTEGER,
            artist_id INTEGER,
            genre_id INTEGER,
            duration_seconds INTEGER NOT NULL,
            mtime INTEGER NOT NULL,
            is_favorite BOOLEAN DEFAULT 0,
            cover_art TEXT,
            added_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(album_id) REFERENCES albums(id),
            FOREIGN KEY(artist_id) REFERENCES artists(id),
            FOREIGN KEY(genre_id) REFERENCES genres(id)
        );
        CREATE TABLE IF NOT EXISTS playlists (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE COLLATE NOCASE,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS playlist_tracks (
            playlist_id INTEGER NOT NULL,
            track_id INTEGER NOT NULL,
            added_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY(playlist_id, track_id),
            FOREIGN KEY(playlist_id) REFERENCES playlists(id),
            FOREIGN KEY(track_id) REFERENCES tracks(id)
        );
        CREATE TABLE IF NOT EXISTS track_artists (
            track_id INTEGER NOT NULL,
            artist_id INTEGER NOT NULL,
            PRIMARY KEY(track_id, artist_id),
            FOREIGN KEY(track_id) REFERENCES tracks(id),
            FOREIGN KEY(artist_id) REFERENCES artists(id)
        );
        CREATE TABLE IF NOT EXISTS track_stats (
            track_id INTEGER PRIMARY KEY,
            play_count INTEGER DEFAULT 0,
            last_played_at DATETIME,
            skip_count INTEGER DEFAULT 0,
            last_skipped_at DATETIME,
            FOREIGN KEY(track_id) REFERENCES tracks(id)
        );
        COMMIT;",
    )
    .map_err(Error::Db)?;

    conn.execute_batch(
        "BEGIN;
        CREATE INDEX IF NOT EXISTS idx_tracks_path ON tracks(path);
        CREATE INDEX IF NOT EXISTS idx_tracks_mtime ON tracks(mtime);
        CREATE INDEX IF NOT EXISTS idx_tracks_artist_id ON tracks(artist_id);
        CREATE INDEX IF NOT EXISTS idx_tracks_album_id ON tracks(album_id);
        CREATE INDEX IF NOT EXISTS idx_tracks_genre_id ON tracks(genre_id);

        CREATE INDEX IF NOT EXISTS idx_artists_name ON artists(name);
        CREATE INDEX IF NOT EXISTS idx_albums_title_artist ON albums(title, artist_id);

        CREATE INDEX IF NOT EXISTS idx_track_stats_last_played
        ON track_stats(last_played_at);

        CREATE INDEX IF NOT EXISTS idx_track_stats_play_count
        ON track_stats(play_count);
        COMMIT;",
    )
    .map_err(Error::Db)?;
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

    // 1. Get all tracks that are inside this directory
    let track_paths: Vec<String> = {
        let mut stmt = tx
            .prepare("SELECT path FROM tracks WHERE path LIKE ? || '%'")
            .map_err(Error::Db)?;
        stmt.query_map(params![path], |row| row.get(0))
            .map_err(Error::Db)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Error::Db)?
    };

    // 2. Delete these tracks and perform cleanup
    if !track_paths.is_empty() {
        delete_tracks_by_paths(&tx, &track_paths)?;
    }

    // 3. Remove the source directory itself
    tx.execute("DELETE FROM source_dirs WHERE path = ?", params![path])
        .map_err(Error::Db)?;

    tx.commit().map_err(Error::Db)?;
    Ok(())
}

pub fn get_all_track_paths_and_mtimes(conn: &Connection) -> Result<HashMap<String, i64>> {
    let mut stmt = conn
        .prepare("SELECT path, mtime FROM tracks")
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
        let sql = format!("DELETE FROM tracks WHERE path IN ({})", placeholders);

        let mut stmt = conn.prepare(&sql).map_err(Error::Db)?;
        let count = stmt.execute(rusqlite::params_from_iter(chunk)).map_err(Error::Db)?;
        total_deleted += count;
    }

    conn.execute_batch(
        "DELETE FROM track_artists WHERE track_id NOT IN (SELECT id FROM tracks);
         DELETE FROM albums WHERE id NOT IN (SELECT DISTINCT album_id FROM tracks WHERE album_id IS NOT NULL);
         DELETE FROM artists WHERE id NOT IN (SELECT DISTINCT artist_id FROM tracks WHERE artist_id IS NOT NULL) AND id NOT IN (SELECT DISTINCT artist_id FROM track_artists) AND profile_picture IS NULL;
         DELETE FROM genres WHERE id NOT IN (SELECT DISTINCT genre_id FROM tracks WHERE genre_id IS NOT NULL);
         DELETE FROM track_stats WHERE track_id NOT IN (SELECT id FROM tracks);"
    ).map_err(Error::Db)?;

    Ok(total_deleted)
}

pub fn get_or_create_artist(conn: &Connection, name: &str) -> Result<i64> {
    conn.execute(
        "INSERT OR IGNORE INTO artists (name) VALUES (?)",
        params![name],
    )
    .map_err(Error::Db)?;

    conn.query_row(
        "SELECT id FROM artists WHERE name = ?",
        params![name],
        |row| row.get(0),
    )
    .map_err(Error::Db)
}

pub fn set_track_artists(conn: &Connection, track_id: i64, artist_ids: &[i64]) -> Result<()> {
    conn.execute(
        "DELETE FROM track_artists WHERE track_id = ?",
        params![track_id],
    ).map_err(Error::Db)?;
    
    for artist_id in artist_ids {
        conn.execute(
            "INSERT OR IGNORE INTO track_artists (track_id, artist_id) VALUES (?, ?)",
            params![track_id, artist_id],
        ).map_err(Error::Db)?;
    }
    Ok(())
}

pub fn update_artist_profile_picture(conn: &Connection, artist_id: i64, path: &str) -> Result<()> {
    conn.execute(
        "UPDATE artists SET profile_picture = ? WHERE id = ? AND profile_picture IS NULL",
        params![path, artist_id],
    )
    .map_err(Error::Db)?;
    Ok(())
}

pub fn artist_has_photo(conn: &Connection, artist_id: i64) -> Result<bool> {
    let res: Option<String> = conn
        .query_row(
            "SELECT profile_picture FROM artists WHERE id = ?",
            params![artist_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(Error::Db)?;
    Ok(res.is_some())
}

pub fn get_or_create_album(
    conn: &Connection,
    title: &str,
    artist_id: i64,
    cover_art: Option<&str>,
) -> Result<i64> {
    conn.execute(
        "INSERT OR IGNORE INTO albums (title, artist_id, cover_art) VALUES (?, ?, ?)",
        params![title, artist_id, cover_art],
    ).map_err(Error::Db)?;

    if let Some(art) = cover_art {
        conn.execute(
            "UPDATE albums SET cover_art = ? WHERE title = ? AND artist_id = ? AND cover_art IS NULL",
            params![art, title, artist_id],
        ).map_err(Error::Db)?;
    }

    conn.query_row(
        "SELECT id FROM albums WHERE title = ? AND artist_id = ?",
        params![title, artist_id],
        |row| row.get(0),
    ).map_err(Error::Db)
}

pub fn get_or_create_genre(conn: &Connection, name: &str) -> Result<i64> {
    conn.execute(
        "INSERT OR IGNORE INTO genres (name) VALUES (?)",
        params![name],
    )
    .map_err(Error::Db)?;

    conn.query_row(
        "SELECT id FROM genres WHERE name = ?",
        params![name],
        |row| row.get(0),
    )
    .map_err(Error::Db)
}

pub fn update_track(
    conn: &Connection,
    path: &str,
    title: &str,
    album_id: i64,
    artist_id: i64,
    genre_id: i64,
    duration: u32,
    mtime: i64,
    cover_art: Option<&str>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO tracks (path, title, album_id, artist_id, genre_id, duration_seconds, mtime, cover_art)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(path) DO UPDATE SET
        title = excluded.title,
        album_id = excluded.album_id,
        artist_id = excluded.artist_id,
        genre_id = excluded.genre_id,
        duration_seconds = excluded.duration_seconds,
        mtime = excluded.mtime,
        cover_art = excluded.cover_art",
        params![path, title, album_id, artist_id, genre_id, duration, mtime, cover_art],
    ).map_err(Error::Db)?;
    
    conn.query_row(
        "SELECT id FROM tracks WHERE path = ?",
        params![path],
        |row| row.get(0)
    ).map_err(Error::Db)
}

pub fn get_track_mtime(conn: &Connection, path: &str) -> Result<Option<i64>> {
    conn.query_row(
        "SELECT mtime FROM tracks WHERE path = ?",
        params![path],
        |row| row.get(0),
    )
    .optional()
    .map_err(Error::Db)
}

pub fn get_track_id_by_path(conn: &Connection, path: &str) -> Result<i64> {
    conn.query_row(
        "SELECT id FROM tracks WHERE path = ?",
        params![path],
        |row| row.get(0),
    )
    .map_err(Error::Db)
}

pub fn toggle_favorite(conn: &Connection, track_id: i64) -> Result<bool> {
    conn.query_row(
        "UPDATE tracks SET is_favorite = NOT is_favorite WHERE id = ? RETURNING is_favorite",
        params![track_id],
        |row| row.get(0),
    )
    .map_err(Error::Db)
}

pub fn record_play(conn: &Connection, track_id: i64) -> Result<()> {
    conn.execute(
        "INSERT INTO track_stats (track_id, play_count, last_played_at)
        VALUES (?, 1, CURRENT_TIMESTAMP)
        ON CONFLICT(track_id) DO UPDATE SET
        play_count = play_count + 1,
        last_played_at = CURRENT_TIMESTAMP",
        params![track_id],
    )
    .map_err(Error::Db)?;
    Ok(())
}

pub fn record_skip(conn: &Connection, track_id: i64) -> Result<()> {
    conn.execute(
        "INSERT INTO track_stats (track_id, skip_count, last_skipped_at)
        VALUES (?, 1, CURRENT_TIMESTAMP)
        ON CONFLICT(track_id) DO UPDATE SET
        skip_count = skip_count + 1,
        last_skipped_at = CURRENT_TIMESTAMP",
        params![track_id],
    )
    .map_err(Error::Db)?;
    Ok(())
}

pub fn get_all_albums(conn: &Connection) -> Result<Vec<Album>> {
    let sql = "SELECT al.id, al.title, al.cover_art
        FROM albums al
        ORDER BY al.title COLLATE NOCASE ASC";

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
        "SELECT id, name, profile_picture FROM artists WHERE id = ?",
        params![artist_id],
        |row| {
            Ok(Artist {
                id: row.get(0)?,
                name: row.get(1)?,
                profile_picture: row.get(2)?,
            })
        },
    )
    .map_err(Error::Db)
}

pub fn get_album(conn: &Connection, album_id: i64) -> Result<Album> {
    conn.query_row(
        "SELECT id, title, cover_art FROM albums WHERE id = ?",
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
        .prepare("SELECT id, name FROM playlists")
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
        .prepare("SELECT id, name, profile_picture FROM artists")
        .map_err(Error::Db)?;
    let artist_iter = stmt
        .query_map([], |row| {
            Ok(Artist {
                id: row.get(0)?,
                name: row.get(1)?,
                profile_picture: row.get(2)?,
            })
        })
        .map_err(Error::Db)?;

    artist_iter
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)
}

pub fn get_albums_by_artist(conn: &Connection, artist_id: i64) -> Result<Vec<Album>> {
    let mut stmt = conn
        .prepare("SELECT id, title, cover_art FROM albums WHERE artist_id = ?")
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
    conn.execute("INSERT INTO playlists (name) VALUES (?)", params![name])
        .map_err(Error::Db)?;
    Ok(())
}

pub fn add_track_to_playlist(conn: &Connection, playlist_id: i64, track_id: i64) -> Result<()> {
    conn.execute(
        "INSERT INTO playlist_tracks (playlist_id, track_id) VALUES (?, ?)",
        params![playlist_id, track_id],
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
        "SELECT t.id, t.title, al.id, al.title, al.cover_art, g.id, g.name, t.duration_seconds, t.is_favorite, t.cover_art
        FROM tracks t
        JOIN playlist_tracks pt ON t.id = pt.track_id
        LEFT JOIN artists ar ON t.artist_id = ar.id
        LEFT JOIN albums al ON t.album_id = al.id
        LEFT JOIN genres g ON t.genre_id = g.id
        WHERE pt.playlist_id = ?";

    prepare_sorted_tracks_list(conn, sql, params![playlist_id], sort_by)
}

pub fn get_tracks_by_artist(
    conn: &Connection,
    artist_id: i64,
    sort_by: Option<SortBy>,
) -> Result<Vec<Track>> {
    let sql =
        "SELECT t.id, t.title, al.id, al.title, al.cover_art, g.id, g.name, t.duration_seconds, t.is_favorite, t.cover_art
                FROM tracks t
                LEFT JOIN track_artists ta ON t.id = ta.track_id
                LEFT JOIN artists ar ON ta.artist_id = ar.id
                LEFT JOIN albums al ON t.album_id = al.id
                LEFT JOIN genres g ON t.genre_id = g.id
                WHERE ta.artist_id = ?";

    prepare_sorted_tracks_list(conn, sql, params![artist_id], sort_by)
}

pub fn get_tracks_by_album(
    conn: &Connection,
    album_id: i64,
    sort_by: Option<SortBy>,
) -> Result<Vec<Track>> {
    let sql = "SELECT t.id, t.title, al.id, al.title, al.cover_art, g.id, g.name, t.duration_seconds, t.is_favorite, t.cover_art
                FROM tracks t
                LEFT JOIN artists ar ON t.artist_id = ar.id
                LEFT JOIN albums al ON t.album_id = al.id
                LEFT JOIN genres g ON t.genre_id = g.id
                WHERE t.album_id = ?";

    prepare_sorted_tracks_list(conn, sql, params![album_id], sort_by)
}

pub fn get_favorite_tracks(conn: &Connection, sort_by: Option<SortBy>) -> Result<Vec<Track>> {
    let sql =
        "SELECT t.id, t.title, al.id, al.title, al.cover_art, g.id, g.name, t.duration_seconds, t.is_favorite, t.cover_art
                FROM tracks t
                LEFT JOIN artists ar ON t.artist_id = ar.id
                LEFT JOIN albums al ON t.album_id = al.id
                LEFT JOIN genres g ON t.genre_id = g.id
                WHERE t.is_favorite = 1";

    prepare_sorted_tracks_list(conn, sql, [], sort_by)
}

pub fn get_recently_played_tracks(conn: &Connection, limit: usize) -> Result<Vec<Track>> {
    let sql =
        "SELECT t.id, t.title, al.id, al.title, al.cover_art, g.id, g.name, t.duration_seconds, t.is_favorite, t.cover_art
                FROM tracks t
                LEFT JOIN artists ar ON t.artist_id = ar.id
                LEFT JOIN albums al ON t.album_id = al.id
                LEFT JOIN genres g ON t.genre_id = g.id
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
    let time_filter = match timeframe {
        Timeframe::AllTime => "1=1",
        Timeframe::ThisWeek => "s.last_played_at >= datetime('now', '-7 days')",
        Timeframe::ThisMonth => "s.last_played_at >= datetime('now', '-30 days')",
        Timeframe::Last6Months => "s.last_played_at >= datetime('now', '-180 days')",
        Timeframe::ThisYear => "s.last_played_at >= datetime('now', 'start of year')",
        Timeframe::Last3Years => "s.last_played_at >= datetime('now', '-3 years')",
    };

    let sql = format!(
        "SELECT t.id, t.title, al.id, al.title, al.cover_art, g.id, g.name, t.duration_seconds, t.is_favorite, t.cover_art
                FROM tracks t
                LEFT JOIN artists ar ON t.artist_id = ar.id
                LEFT JOIN albums al ON t.album_id = al.id
                LEFT JOIN genres g ON t.genre_id = g.id
                JOIN track_stats s ON t.id = s.track_id
                WHERE s.play_count > 0 AND {}
                ORDER BY s.play_count DESC
                LIMIT ?",
        time_filter
    );

    prepare_tracks_list(conn, &sql, params![limit])
}

pub fn get_top_artists(conn: &Connection, limit: usize) -> Result<Vec<Artist>> {
    let sql =
        "SELECT ar.id, ar.name, ar.profile_picture, SUM(IFNULL(s.play_count, 0)) as total_plays
                FROM artists ar
                JOIN tracks t ON t.artist_id = ar.id
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
                profile_picture: row.get(2)?,
            })
        })
        .map_err(Error::Db)?;

    artist_iter
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)
}

pub fn get_top_albums(conn: &Connection, limit: usize) -> Result<Vec<Album>> {
    let sql = "SELECT al.id, al.title, al.cover_art, SUM(IFNULL(s.play_count, 0)) as total_plays
                FROM albums al
                JOIN tracks t ON t.album_id = al.id
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
        "SELECT t.id, t.title, al.id, al.title, al.cover_art, g.id, g.name, t.duration_seconds, t.is_favorite, t.cover_art
                FROM tracks t
                LEFT JOIN artists ar ON t.artist_id = ar.id
                LEFT JOIN albums al ON t.album_id = al.id
                LEFT JOIN genres g ON t.genre_id = g.id
                JOIN track_stats s ON t.id = s.track_id
                WHERE s.play_count > 0 AND s.last_played_at <= datetime('now', '-180 days')
                ORDER BY s.last_played_at ASC
                LIMIT ?";

    prepare_tracks_list(conn, sql, params![limit])
}

pub fn get_unplayed_tracks(conn: &Connection, limit: usize) -> Result<Vec<Track>> {
    let sql =
        "SELECT t.id, t.title, al.id, al.title, al.cover_art, g.id, g.name, t.duration_seconds, t.is_favorite, t.cover_art
                FROM tracks t
                LEFT JOIN artists ar ON t.artist_id = ar.id
                LEFT JOIN albums al ON t.album_id = al.id
                LEFT JOIN genres g ON t.genre_id = g.id
                LEFT JOIN track_stats s ON t.id = s.track_id
                WHERE s.play_count IS NULL OR s.play_count = 0
                ORDER BY t.added_at DESC
                LIMIT ?";

    prepare_tracks_list(conn, sql, params![limit])
}

pub fn get_recently_added_tracks(conn: &Connection, limit: usize) -> Result<Vec<Track>> {
    let sql =
        "SELECT t.id, t.title, al.id, al.title, al.cover_art, g.id, g.name, t.duration_seconds, t.is_favorite, t.cover_art
                FROM tracks t
                LEFT JOIN artists ar ON t.artist_id = ar.id
                LEFT JOIN albums al ON t.album_id = al.id
                LEFT JOIN genres g ON t.genre_id = g.id
                ORDER BY t.added_at DESC
                LIMIT ?";

    prepare_tracks_list(conn, sql, params![limit])
}

pub fn get_track_details(conn: &Connection, track_id: i64) -> Result<TrackDetails> {
    let sql = "SELECT t.id, t.path, t.title, al.id, al.title, al.cover_art, g.id, g.name, t.duration_seconds, t.is_favorite, t.mtime,
        IFNULL(s.play_count, 0), s.last_played_at, IFNULL(s.skip_count, 0), s.last_skipped_at, t.cover_art
        FROM tracks t
        LEFT JOIN artists ar ON t.artist_id = ar.id
        LEFT JOIN albums al ON t.album_id = al.id
        LEFT JOIN genres g ON t.genre_id = g.id
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

            let genre_id: Option<i64> = row.get(6)?;
            let genre_name: Option<String> = row.get(7)?;
            let genre = if let (Some(id), Some(name)) = (genre_id, genre_name) {
                vec![Genre { id, name }]
            } else {
                vec![]
            };

            Ok(TrackDetails {
                id: row.get(0)?,
                path: row.get(1)?,
                title: row.get(2)?,
                artists: vec![],
                album,
                genre,
                duration_seconds: row.get(8)?,
                is_favorite: row.get(9)?,
                mtime: row.get(10)?,
                play_count: row.get(11)?,
                last_played_at: row.get(12)?,
                skipped_count: row.get(13)?,
                last_skipped_at: row.get(14)?,
                cover_art: row.get(15)?,
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
            al.title,
            al.cover_art,
            g.id,
            g.name,
            t.duration_seconds,
            t.is_favorite,
            t.cover_art
        FROM tracks t
        LEFT JOIN track_artists ta ON t.id = ta.track_id
        LEFT JOIN artists ar ON ta.artist_id = ar.id
        LEFT JOIN albums al ON al.id = t.album_id
        LEFT JOIN genres g ON g.id = t.genre_id
        LEFT JOIN track_stats s ON s.track_id = t.id
        WHERE t.title LIKE ? OR ar.name LIKE ? OR al.title LIKE ? OR g.name LIKE ?
        GROUP BY t.id
        ORDER BY
            (CASE WHEN t.title LIKE ? THEN 3 ELSE 0 END) +
            (CASE WHEN ar.name LIKE ? THEN 2 ELSE 0 END) +
            (CASE WHEN al.title LIKE ? THEN 2 ELSE 0 END) +
            (CASE WHEN g.name LIKE ? THEN 1 ELSE 0 END) DESC,
            IFNULL(s.play_count, 0) DESC
        LIMIT ?
        "#;

    prepare_tracks_list(
        conn,
        sql,
        params![
            pattern, pattern, pattern, pattern, pattern, pattern, pattern, pattern, limit
        ],
    )
}

pub fn get_similar_tracks(
    conn: &Connection,
    current_track_id: i64,
    limit: usize,
) -> Result<Vec<Track>> {
    let sql = r#"
    SELECT t.id, t.title, al.id, al.title, al.cover_art, g.id, g.name, t.duration_seconds, t.is_favorite, t.cover_art
    FROM tracks t
    JOIN tracks current ON current.id = ?
    LEFT JOIN artists ar ON ar.id = t.artist_id
    LEFT JOIN albums al ON al.id = t.album_id
    LEFT JOIN genres g ON g.id = t.genre_id
    LEFT JOIN track_stats s ON s.track_id = t.id
    WHERE t.id != current.id
    ORDER BY (
            (CASE WHEN t.artist_id IS current.artist_id THEN 50 ELSE 0 END) +
            (CASE WHEN t.album_id  IS current.album_id  THEN 20 ELSE 0 END) +
            (CASE WHEN t.genre_id  IS current.genre_id  THEN 15 ELSE 0 END) +
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

            -- 2-Hour Cool Down so that recently played tracks don't dominate the list
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
        "SELECT t.id, t.title, al.id, al.title, al.cover_art, g.id, g.name, t.duration_seconds, t.is_favorite, t.cover_art
        FROM tracks t
        LEFT JOIN artists ar ON t.artist_id = ar.id
        LEFT JOIN albums al ON t.album_id = al.id
        LEFT JOIN genres g ON t.genre_id = g.id";

    prepare_sorted_tracks_list(conn, sql, [], sort_by)
}

pub fn delete_playlist(conn: &Connection, playlist_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM playlist_tracks WHERE playlist_id = ?",
        params![playlist_id],
    ).map_err(Error::Db)?;
    
    conn.execute("DELETE FROM playlists WHERE id = ?", params![playlist_id]).map_err(Error::Db)?;
    Ok(())
}

pub fn remove_track_from_playlist(
    conn: &Connection,
    playlist_id: i64,
    track_id: i64,
) -> Result<()> {
    conn.execute(
        "DELETE FROM playlist_tracks WHERE playlist_id = ? AND track_id = ?",
        params![playlist_id, track_id],
    )
    .map_err(Error::Db)?;
    Ok(())
}

pub fn rename_playlist(conn: &Connection, playlist_id: i64, new_name: &str) -> Result<()> {
    conn.execute(
        "UPDATE playlists SET name = ? WHERE id = ?",
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
            FROM tracks t
            JOIN playlist_tracks pt ON t.id = pt.track_id
            WHERE pt.playlist_id = ? AND t.cover_art IS NOT NULL
            ORDER BY pt.added_at ASC
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

            let genre_id: Option<i64> = row.get(5)?;
            let genre_name: Option<String> = row.get(6)?;
            let genre = if let (Some(id), Some(name)) = (genre_id, genre_name) {
                vec![Genre { id, name }]
            } else {
                vec![]
            };

            Ok(Track {
                id: row.get(0)?,
                title: row.get(1)?,
                artists: vec![],
                album,
                genre,
                duration_seconds: row.get(7)?,
                is_favorite: row.get(8)?,
                cover_art: row.get(9)?,
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
            "SELECT ta.track_id, ar.id, ar.name, ar.profile_picture
             FROM track_artists ta
             JOIN artists ar ON ta.artist_id = ar.id
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
                        profile_picture: row.get(3)?,
                    },
                ))
            })
            .map_err(Error::Db)?;

        for row in rows {
            let (track_id, artist) = row.map_err(Error::Db)?;
            map.entry(track_id).or_insert_with(Vec::new).push(artist);
        }
    }

    let missing: Vec<i64> = track_ids
        .iter()
        .filter(|id| !map.contains_key(id))
        .copied()
        .collect();

    if !missing.is_empty() {
        for chunk in missing.chunks(900) {
            let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!(
                "SELECT t.id, ar.id, ar.name, ar.profile_picture
                 FROM tracks t
                 LEFT JOIN artists ar ON t.artist_id = ar.id
                 WHERE t.id IN ({})",
                placeholders
            );
            let mut stmt = conn.prepare(&sql).map_err(Error::Db)?;
            let rows = stmt
                .query_map(rusqlite::params_from_iter(chunk), |row| {
                    let track_id: i64 = row.get(0)?;
                    let artist_id: Option<i64> = row.get(1)?;
                    let artist_name: Option<String> = row.get(2)?;
                    let artist_pic: Option<String> = row.get(3)?;
                    Ok((track_id, artist_id, artist_name, artist_pic))
                })
                .map_err(Error::Db)?;

            for row in rows {
                let (track_id, artist_id, artist_name, artist_pic) = row.map_err(Error::Db)?;
                if let (Some(id), Some(name)) = (artist_id, artist_name) {
                    map.entry(track_id).or_insert_with(Vec::new).push(Artist {
                        id,
                        name,
                        profile_picture: artist_pic,
                    });
                }
            }
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
        Some(SortBy::Artist) => format!("{} ORDER BY ar.name COLLATE NOCASE ASC", sql),
        Some(SortBy::Album) => format!("{} ORDER BY al.title COLLATE NOCASE ASC", sql),
        Some(SortBy::Duration) => format!("{} ORDER BY t.duration_seconds ASC", sql),
        Some(SortBy::RecentlyAdded) => format!("{} ORDER BY t.added_at DESC", sql),
        None => format!("{} ORDER BY t.added_at ASC", sql),
    };

    prepare_tracks_list(conn, &sql, params)
}
