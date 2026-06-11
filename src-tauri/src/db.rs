use crate::models::*;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection, OptionalExtension, Params, Result};

pub type DbPool = Pool<SqliteConnectionManager>;

pub enum Timeframe {
    AllTime,
    ThisWeek,
    ThisMonth,
    Last6Months,
    ThisYear,
    Last3Years,
}

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
            name TEXT NOT NULL UNIQUE,
            profile_picture TEXT
        );
        CREATE TABLE IF NOT EXISTS albums (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            artist_id INTEGER NOT NULL,
            cover_art TEXT,
            UNIQUE(title, artist_id),
            FOREIGN KEY(artist_id) REFERENCES artists(id)
        );
        CREATE TABLE IF NOT EXISTS genres (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
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
            name TEXT NOT NULL UNIQUE,
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
        CREATE TABLE IF NOT EXISTS track_stats (
            track_id INTEGER PRIMARY KEY,
            play_count INTEGER DEFAULT 0,
            last_played_at DATETIME,
            skip_count INTEGER DEFAULT 0,
            last_skipped_at DATETIME,
            FOREIGN KEY(track_id) REFERENCES tracks(id)
        );
        COMMIT;",
    )?;
    Ok(())
}

pub fn add_source_dir(conn: &Connection, path: &str) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO source_dirs (path) VALUES (?)",
        params![path],
    )?;
    Ok(())
}

pub fn get_source_dirs(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT path FROM source_dirs")?;
    let dirs = stmt
        .query_map([], |row| row.get(0))?
        .filter_map(Result::ok)
        .collect();
    Ok(dirs)
}

pub fn get_or_create_artist(conn: &Connection, name: &str) -> Result<i64> {
    conn.execute(
        "INSERT OR IGNORE INTO artists (name) VALUES (?)",
        params![name],
    )?;
    Ok(conn.query_row(
        "SELECT id FROM artists WHERE name = ?",
        params![name],
        |row| row.get(0),
    )?)
}

pub fn get_or_create_album(conn: &Connection, title: &str, artist_id: i64) -> Result<i64> {
    conn.execute(
        "INSERT OR IGNORE INTO albums (title, artist_id) VALUES (?, ?)",
        params![title, artist_id],
    )?;
    Ok(conn.query_row(
        "SELECT id FROM albums WHERE title = ? AND artist_id = ?",
        params![title, artist_id],
        |row| row.get(0),
    )?)
}

pub fn get_or_create_genre(conn: &Connection, name: &str) -> Result<i64> {
    conn.execute(
        "INSERT OR IGNORE INTO genres (name) VALUES (?)",
        params![name],
    )?;
    Ok(conn.query_row(
        "SELECT id FROM genres WHERE name = ?",
        params![name],
        |row| row.get(0),
    )?)
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
) -> Result<()> {
    conn.execute(
        "INSERT INTO tracks (path, title, album_id, artist_id, genre_id, duration_seconds, mtime)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(path) DO UPDATE SET
        title = excluded.title,
        album_id = excluded.album_id,
        artist_id = excluded.artist_id,
        genre_id = excluded.genre_id,
        duration_seconds = excluded.duration_seconds,
        mtime = excluded.mtime",
        params![path, title, album_id, artist_id, genre_id, duration, mtime],
    )?;
    Ok(())
}

pub fn get_track_mtime(conn: &Connection, path: &str) -> Result<Option<i64>> {
    Ok(conn
        .query_row(
            "SELECT mtime FROM tracks WHERE path = ?",
            params![path],
            |row| row.get(0),
        )
        .optional()?)
}

pub fn toggle_favorite(conn: &Connection, track_id: i64) -> Result<bool> {
    let current: bool = conn.query_row(
        "SELECT is_favorite FROM tracks WHERE id = ?",
        params![track_id],
        |row| row.get(0),
    )?;
    let new_val = !current;
    conn.execute(
        "UPDATE tracks SET is_favorite = ? WHERE id = ?",
        params![new_val, track_id],
    )?;
    Ok(new_val)
}

pub fn record_play(conn: &Connection, track_id: i64) -> Result<()> {
    conn.execute(
        "INSERT INTO track_stats (track_id, play_count, last_played_at)
        VALUES (?, 1, CURRENT_TIMESTAMP)
        ON CONFLICT(track_id) DO UPDATE SET
        play_count = play_count + 1,
        last_played_at = CURRENT_TIMESTAMP",
        params![track_id],
    )?;
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
    )?;
    Ok(())
}

pub fn get_all_playlists(conn: &Connection) -> Result<Vec<Playlist>> {
    let mut stmt = conn.prepare("SELECT id, name FROM playlists")?;
    let playlist_iter = stmt.query_map([], |row| {
        Ok(Playlist {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;

    let mut playlists = Vec::new();
    for playlist in playlist_iter {
        playlists.push(playlist?);
    }
    Ok(playlists)
}

pub fn get_all_artists(conn: &Connection) -> Result<Vec<Artist>> {
    let mut stmt = conn.prepare("SELECT id, name, profile_picture FROM artists")?;
    let artist_iter = stmt.query_map([], |row| {
        Ok(Artist {
            id: row.get(0)?,
            name: row.get(1)?,
            profile_picture: row.get(2)?,
        })
    })?;

    let mut artists = Vec::new();
    for artist in artist_iter {
        artists.push(artist?);
    }
    Ok(artists)
}

pub fn get_albums_by_artist(conn: &Connection, artist_id: i64) -> Result<Vec<Album>> {
    let mut stmt = conn.prepare("SELECT id, title, cover_art FROM albums WHERE artist_id = ?")?;
    let album_iter = stmt.query_map(params![artist_id], |row| {
        Ok(Album {
            id: row.get(0)?,
            name: row.get(1)?,
            cover_art: row.get(2)?,
        })
    })?;

    album_iter.collect()
}

pub fn create_playlist(conn: &Connection, name: &str) -> Result<()> {
    conn.execute("INSERT INTO playlists (name) VALUES (?)", params![name])?;
    Ok(())
}

pub fn add_track_to_playlist(conn: &Connection, playlist_id: i64, track_id: i64) -> Result<()> {
    conn.execute(
        "INSERT INTO playlist_tracks (playlist_id, track_id) VALUES (?, ?)",
        params![playlist_id, track_id],
    )?;
    Ok(())
}

pub fn get_tracks_in_playlist(conn: &Connection, playlist_id: i64, , sort_by: Option<SortBy>) -> Result<Vec<Track>> {
    let sql = "SELECT t.id, t.title, ar.name, al.title,t.duration_seconds, t.is_favorite
        FROM tracks t
        JOIN playlist_tracks pt ON t.id = pt.track_id
        LEFT JOIN artists ar ON t.artist_id = ar.id
        LEFT JOIN albums al ON t.album_id = al.id
        WHERE pt.playlist_id = ?";

    prepare_sorted_tracks_list(conn, sql, params![playlist_id], sort_by)
}

pub fn get_tracks_by_artist(conn: &Connection, artist_id: i64, sort_by: Option<SortBy>) -> Result<Vec<Track>> {
    let sql = "SELECT t.id, t.title, ar.name, al.title,t.duration_seconds, t.is_favorite
                IFNULL(s.play_count, 0), s.last_played_at
                FROM tracks t
                LEFT JOIN artists ar ON t.artist_id = ar.id
                LEFT JOIN albums al ON t.album_id = al.id
                LEFT JOIN genres g ON t.genre_id = g.id
                LEFT JOIN track_stats s ON t.id = s.track_id
                WHERE t.artist_id = ?";

    prepare_sorted_tracks_list(conn, sql, params![artist_id], sort_by)
}

pub fn get_tracks_by_album(conn: &Connection, album_id: i64, sort_by: Option<SortBy>) -> Result<Vec<Track>> {
    let sql = "SELECT t.id, t.title, ar.name, al.title,t.duration_seconds, t.is_favorite
                FROM tracks t
                LEFT JOIN artists ar ON t.artist_id = ar.id
                LEFT JOIN albums al ON t.album_id = al.id
                WHERE t.album_id = ?";

    prepare_sorted_tracks_list(conn, sql, params![album_id], sort_by)
}

pub fn get_favorite_tracks(conn: &Connection, sort_by: Option<SortBy>) -> Result<Vec<Track>> {
    let sql = "SELECT t.id, t.title, ar.name, al.title,t.duration_seconds, t.is_favorite
                FROM tracks t
                LEFT JOIN artists ar ON t.artist_id = ar.id
                LEFT JOIN albums al ON t.album_id = al.id
                WHERE t.is_favorite = 1";

    prepare_sorted_tracks_list(conn, sql, [], sort_by)
}

pub fn get_recently_played_tracks(conn: &Connection, limit: usize) -> Result<Vec<Track>> {
    let sql = "SELECT t.id, t.title, ar.name, al.title,t.duration_seconds, t.is_favorite
                FROM tracks t
                LEFT JOIN artists ar ON t.artist_id = ar.id
                LEFT JOIN albums al ON t.album_id = al.id
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
        "SELECT t.id, t.title, ar.name, al.title,t.duration_seconds, t.is_favorite
                FROM tracks t
                LEFT JOIN artists ar ON t.artist_id = ar.id
                LEFT JOIN albums al ON t.album_id = al.id
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

    let mut stmt = conn.prepare(&sql)?;
    let artist_iter = stmt.query_map(params![limit], |row| {
        Ok(Artist {
            id: row.get(0)?,
            name: row.get(1)?,
            profile_picture: row.get(2)?,
        })
    })?;

    artist_iter.collect()
}

pub fn get_top_albums(conn: &Connection, limit: usize) -> Result<Vec<Album>> {
    let sql = "SELECT al.id, al.title, al.cover_art, SUM(IFNULL(s.play_count, 0)) as total_plays
                FROM albums al
                JOIN tracks t ON t.album_id = al.id
                LEFT JOIN track_stats s ON t.id = s.track_id
                GROUP BY al.id
                ORDER BY total_plays DESC
                LIMIT ?";

    let mut stmt = conn.prepare(&sql)?;
    let album_iter = stmt.query_map(params![limit], |row| {
        Ok(Album {
            id: row.get(0)?,
            name: row.get(1)?,
            cover_art: row.get(2)?,
        })
    })?;

    album_iter.collect()
}

pub fn get_forgotten_tracks(conn: &Connection, limit: usize) -> Result<Vec<Track>> {
    let sql = "SELECT t.id, t.title, ar.name, al.title,t.duration_seconds, t.is_favorite
                FROM tracks t
                LEFT JOIN artists ar ON t.artist_id = ar.id
                LEFT JOIN albums al ON t.album_id = al.id
                JOIN track_stats s ON t.id = s.track_id
                WHERE s.play_count > 0 AND s.last_played_at <= datetime('now', '-180 days')
                ORDER BY s.last_played_at ASC
                LIMIT ?";

    prepare_tracks_list(conn, sql, params![limit])
}

pub fn get_unplayed_tracks(conn: &Connection, limit: usize) -> Result<Vec<Track>> {
    let sql = "SELECT t.id, t.title, ar.name, al.title,t.duration_seconds, t.is_favorite
                FROM tracks t
                LEFT JOIN artists ar ON t.artist_id = ar.id
                LEFT JOIN albums al ON t.album_id = al.id
                LEFT JOIN track_stats s ON t.id = s.track_id
                WHERE s.play_count IS NULL OR s.play_count = 0
                ORDER BY t.added_at DESC
                LIMIT ?";

    prepare_tracks_list(conn, sql, params![limit])
}

pub fn get_recently_added_tracks(conn: &Connection, limit: usize) -> Result<Vec<Track>> {
    let sql = "SELECT t.id, t.title, ar.name, al.title,t.duration_seconds, t.is_favorite
                FROM tracks t
                LEFT JOIN artists ar ON t.artist_id = ar.id
                LEFT JOIN albums al ON t.album_id = al.id
                ORDER BY t.added_at DESC
                LIMIT ?";

    prepare_tracks_list(conn, sql, params![limit])
}

pub fn get_track_details(conn: &Connection, track_id: i64) -> Result<TrackDetails> {
    let sql = "SELECT t.id, t.path, t.title, ar.name, al.title, g.name, t.duration_seconds, t.is_favorite, t.mtime,
        IFNULL(s.play_count, 0), s.last_played_at, IFNULL(s.skip_count, 0), s.last_skipped_at
        FROM tracks t
        LEFT JOIN artists ar ON t.artist_id = ar.id
        LEFT JOIN albums al ON t.album_id = al.id
        LEFT JOIN genres g ON t.genre_id = g.id
        LEFT JOIN track_stats s ON t.id = s.track_id
        WHERE t.id = ?";

    conn.query_row(sql, params![track_id], |row| {
        Ok(TrackDetails {
            id: row.get(0)?,
            path: row.get(1)?,
            title: row.get(2)?,
            artist: row.get(3)?,
            album: row.get(4)?,
            genre: row.get(5)?,
            duration_seconds: row.get(6)?,
            is_favorite: row.get(7)?,
            mtime: row.get(8)?,
            play_count: row.get(9)?,
            last_played_at: row.get(10)?,
            skipped_count: row.get(11)?,
            last_skipped_at: row.get(12)?,
        })
    })
}

pub fn search_tracks(conn: &Connection, query: &str, limit: usize) -> Result<Vec<Track>> {
    let pattern = format!("%{}%", query);

    let sql = r#"
        SELECT
            t.id,
            t.title,
            ar.name,
            al.title,
            t.duration_seconds,
            t.is_favorite,
            (
                (CASE WHEN t.title LIKE ? THEN 3 ELSE 0 END) +
                (CASE WHEN ar.name LIKE ? THEN 2 ELSE 0 END) +
                (CASE WHEN al.title LIKE ? THEN 2 ELSE 0 END) +
                (CASE WHEN g.name LIKE ? THEN 1 ELSE 0 END)
            ) AS relevance
        FROM tracks t
        LEFT JOIN artists ar ON ar.id = t.artist_id
        LEFT JOIN albums al ON al.id = t.album_id
        LEFT JOIN genres g ON g.id = t.genre_id
        ORDER BY
            relevance DESC,
            IFNULL(s.play_count, 0) DESC
        LIMIT ?
        "#;

    prepare_tracks_list(
        conn,
        sql,
        params![pattern, pattern, pattern, pattern, limit],
    )
}

pub fn get_similar_tracks(
    conn: &Connection,
    current_track_id: i64,
    limit: usize,
) -> Result<Vec<Track>> {
    let sql = r#"
    SELECT t.id, t.title, ar.name, al.title,t.duration_seconds, t.is_favorite
        (
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
        ) AS score

    FROM tracks t
    JOIN tracks current ON current.id = ?
    LEFT JOIN artists ar ON ar.id = t.artist_id
    LEFT JOIN albums al ON al.id = t.album_id
    LEFT JOIN genres g ON g.id = t.genre_id
    LEFT JOIN track_stats s ON s.track_id = t.id

    WHERE t.id != current.id
    ORDER BY score DESC
    LIMIT ?
    "#;
    prepare_tracks_list(conn, sql, params![current_track_id, limit])
}

pub fn get_all_tracks(conn: &Connection, sort_by: Option<SortBy>) -> Result<Vec<Track>> {
    let sql = "SELECT t.id, t.path, t.title, ar.name, al.title, g.name, t.duration_seconds, t.is_favorite, t.mtime,
        IFNULL(s.play_count, 0), s.last_played_at
        FROM tracks t
        LEFT JOIN artists ar ON t.artist_id = ar.id
        LEFT JOIN albums al ON t.album_id = al.id
        LEFT JOIN genres g ON t.genre_id = g.id
        LEFT JOIN track_stats s ON t.id = s.track_id";

    prepare_sorted_tracks_list(conn, sql, [], sort_by)
}

pub fn delete_playlist(conn: &Connection, playlist_id: i64) -> Result<()> {
    conn.execute("DELETE FROM playlists WHERE id = ?", params![playlist_id])?;
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
    )?;
    Ok(())
}

pub fn rename_playlist(conn: &Connection, playlist_id: i64, new_name: &str) -> Result<()> {
    conn.execute(
        "UPDATE playlists SET name = ? WHERE id = ?",
        params![new_name, playlist_id],
    )?;
    Ok(())
}

// Utils

fn prepare_tracks_list<P: Params>(
    conn: &Connection,
    sql: &str,
    params: P,
) -> Result<Vec<Track>> {
    let mut stmt = conn.prepare(sql)?;

    let rows = stmt.query_map(params, |row| {
        Ok(Track {
            id: row.get(0)?,
            title: row.get(1)?,
            artist: row.get(2)?,
            album: row.get(3)?,
            duration_seconds: row.get(4)?,
            is_favorite: row.get(5)?,
        })
    })?;

    rows.collect()
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
