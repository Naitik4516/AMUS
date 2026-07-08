use crate::error::{Error, Result};
use crate::models::*;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, Params, params, types::ToSql};
use rusqlite_migration::{M, Migrations};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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

const MIGRATIONS_SLICE: &[M<'_>] = &[
    M::up(include_str!("../migrations/001_initial_schema.sql")),
    M::up(include_str!("../migrations/002_add_album_artist.sql")),
    M::up(include_str!("../migrations/003_add_fetch_tracking.sql")),
    M::up(include_str!("../migrations/004_add_playlist_cover_art.sql")),
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

pub fn get_or_create_artist(conn: &Connection, name: &str) -> Result<i64> {
    conn
        .prepare_cached(
            "INSERT INTO artist (name) VALUES (?1) ON CONFLICT(name) DO UPDATE SET name=?1 RETURNING id",
        )
        .map_err(Error::Db)?
        .query_row(params![name], |row| row.get(0))
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

pub fn update_artist_profile_image(conn: &Connection, artist_id: i64, path: &str) -> Result<()> {
    conn.execute(
        "UPDATE artist SET profile_image = ? WHERE id = ?",
        params![path, artist_id],
    )
    .map_err(Error::Db)?;
    Ok(())
}

pub fn update_artist_banner_image(conn: &Connection, artist_id: i64, path: &str) -> Result<()> {
    conn.execute(
        "UPDATE artist SET banner_image = ? WHERE id = ?",
        params![path, artist_id],
    )
    .map_err(Error::Db)?;
    Ok(())
}

pub fn clear_artist_profile_image(conn: &Connection, artist_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE artist SET profile_image = NULL WHERE id = ?",
        params![artist_id],
    )
    .map_err(Error::Db)?;
    Ok(())
}

pub fn clear_artist_banner_image(conn: &Connection, artist_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE artist SET banner_image = NULL WHERE id = ?",
        params![artist_id],
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

pub fn set_album_artist(conn: &Connection, name: &str, album_artist: &str) -> Result<()> {
    conn.prepare_cached(
        "UPDATE album SET album_artist = ? WHERE name = ? AND album_artist IS NULL",
    )
    .map_err(Error::Db)?
    .execute(params![album_artist, name])
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
) -> Result<i64> {
    conn.prepare_cached(
        "INSERT INTO track (path, title, duration_sec, year, mtime, file_size, cover_art)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(path) DO UPDATE SET
            title = excluded.title,
            duration_sec = excluded.duration_sec,
            year = excluded.year,
            mtime = excluded.mtime,
            file_size = excluded.file_size,
            cover_art = excluded.cover_art
            RETURNING id",
    )
    .map_err(Error::Db)?
    .query_row(
        params![path, title, duration_sec, year, mtime, file_size, cover_art],
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
    let placeholders: Vec<String> = pairs.iter().map(|_| "(?, ?)".to_string()).collect();
    let sql = format!(
        "INSERT OR IGNORE INTO track_artist (track_id, artist_id) VALUES {}",
        placeholders.join(", ")
    );
    let mut param_values: Vec<Box<dyn ToSql>> = Vec::with_capacity(pairs.len() * 2);
    for &(tid, aid) in pairs {
        param_values.push(Box::new(tid));
        param_values.push(Box::new(aid));
    }
    let params_refs: Vec<&dyn ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
    conn.prepare_cached(&sql)
        .map_err(Error::Db)?
        .execute(params_refs.as_slice())
        .map_err(Error::Db)?;
    Ok(())
}

pub fn bulk_insert_track_albums(conn: &Connection, entries: &[(i64, i64, i32)]) -> Result<()> {
    if entries.is_empty() {
        return Ok(());
    }
    let placeholders: Vec<String> = entries.iter().map(|_| "(?, ?, ?)".to_string()).collect();
    let sql = format!(
        "INSERT OR IGNORE INTO album_track (album_id, track_id, track_number) VALUES {}",
        placeholders.join(", ")
    );
    let mut param_values: Vec<Box<dyn ToSql>> = Vec::with_capacity(entries.len() * 3);
    for &(aid, tid, tn) in entries {
        param_values.push(Box::new(aid));
        param_values.push(Box::new(tid));
        param_values.push(Box::new(tn));
    }
    let params_refs: Vec<&dyn ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
    conn.prepare_cached(&sql)
        .map_err(Error::Db)?
        .execute(params_refs.as_slice())
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

pub fn get_track_by_id(conn: &Connection, id: i64) -> Result<Track> {
    let sql =
        "SELECT t.id, t.title, al.id, al.name, al.cover_art, alt.track_number, al.album_artist, al.year, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        WHERE t.id = ?";
    let tracks = prepare_tracks_list(conn, sql, params![id])?;
    tracks.into_iter().next().ok_or_else(|| {
        Error::Db(rusqlite::Error::QueryReturnedNoRows)
    })
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
    let sql = "SELECT id, name, cover_art, album_artist, year
        FROM album
        ORDER BY name COLLATE NOCASE ASC";

    let mut stmt = conn.prepare(sql).map_err(Error::Db)?;
    let raw: Vec<(Album, Option<String>)> = stmt
        .query_map([], |row| {
            let album = Album {
                id: row.get(0)?,
                name: row.get(1)?,
                cover_art: row.get(2)?,
                album_artist: None,
                year: row.get::<_, Option<i32>>(4)?.map(|y| y as u32),
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
                };
                Ok((album, row.get::<_, Option<String>>(3)?))
            },
        )
        .map_err(Error::Db)?;
    album.album_artist = resolve_album_artist(conn, aa)?;
    Ok(album)
}

pub fn get_all_playlists(conn: &Connection) -> Result<Vec<Playlist>> {
    let mut stmt = conn
        .prepare("SELECT id, name, cover_art FROM playlist")
        .map_err(Error::Db)?;
    let playlist_iter = stmt
        .query_map([], |row| {
            Ok(Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
                cover_art: row.get(2)?,
            })
        })
        .map_err(Error::Db)?;

    playlist_iter
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)
}

pub fn get_all_artists(conn: &Connection) -> Result<Vec<Artist>> {
    let mut stmt = conn
        .prepare("SELECT id, name, profile_image, banner_image FROM artist")
        .map_err(Error::Db)?;
    let artist_iter = stmt
        .query_map([], |row| {
            Ok(Artist {
                id: row.get(0)?,
                name: row.get(1)?,
                profile_image: row.get(2)?,
                banner_image: row.get(3)?,
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
            "SELECT t.id, t.title, al.id, al.name, al.cover_art, alt.track_number, al.album_artist, al.year, t.duration_sec, t.is_favorite, t.cover_art, t.added_at
        FROM track t
        JOIN playlist_track pt ON t.id = pt.track_id
        LEFT JOIN album_track alt ON t.id = alt.track_id
        LEFT JOIN album al ON alt.album_id = al.id
        WHERE pt.playlist_id = ?";

    prepare_sorted_tracks_list(conn, sql, params![playlist_id], sort_by)
}

pub fn get_track_details(conn: &Connection, track_id: i64) -> Result<TrackDetails> {
    let sql = "SELECT t.id, t.path, t.title, al.id, al.name, al.cover_art, alt.track_number, al.album_artist, al.year, t.duration_sec, t.is_favorite, t.mtime,
        IFNULL(s.play_count, 0), s.last_played_at, IFNULL(s.skip_count, 0), s.last_skipped_at, t.cover_art, t.added_at
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
                THEN 100
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

pub fn update_playlist(
    conn: &Connection,
    playlist_id: i64,
    name: Option<&str>,
    cover_art: Option<&str>,
) -> Result<Playlist> {
    if let Some(n) = name {
        conn.execute(
            "UPDATE playlist SET name = ? WHERE id = ?",
            params![n, playlist_id],
        )
        .map_err(Error::Db)?;
    }
    if let Some(ca) = cover_art {
        if ca.is_empty() {
            conn.execute(
                "UPDATE playlist SET cover_art = NULL WHERE id = ?",
                params![playlist_id],
            )
            .map_err(Error::Db)?;
        } else {
            conn.execute(
                "UPDATE playlist SET cover_art = ? WHERE id = ?",
                params![ca, playlist_id],
            )
            .map_err(Error::Db)?;
        }
    }
    get_playlist(conn, playlist_id)
}

pub fn rename_artist(conn: &Connection, artist_id: i64, new_name: &str) -> Result<()> {
    conn.execute(
        "UPDATE artist SET name = ? WHERE id = ?",
        params![new_name, artist_id],
    )
    .map_err(Error::Db)?;
    Ok(())
}

pub fn rename_album(conn: &Connection, album_id: i64, new_name: &str) -> Result<()> {
    conn.execute(
        "UPDATE album SET name = ? WHERE id = ?",
        params![new_name, album_id],
    )
    .map_err(Error::Db)?;
    Ok(())
}

pub fn update_album_cover(conn: &Connection, album_id: i64, cover_art: Option<&str>) -> Result<()> {
    match cover_art {
        Some(path) => conn.execute(
            "UPDATE album SET cover_art = ? WHERE id = ?",
            params![path, album_id],
        ),
        None => conn.execute(
            "UPDATE album SET cover_art = NULL WHERE id = ?",
            params![album_id],
        ),
    }
    .map_err(Error::Db)?;
    Ok(())
}

pub fn update_artist_full(conn: &Connection, id: i64, name: Option<&str>, profile_image: Option<&str>, banner_image: Option<&str>) -> Result<Artist> {
    if let Some(n) = name {
        rename_artist(conn, id, n)?;
    }
    if let Some(pi) = profile_image {
        if pi.is_empty() {
            clear_artist_profile_image(conn, id)?;
        } else {
            update_artist_profile_image(conn, id, pi)?;
        }
    }
    if let Some(bi) = banner_image {
        if bi.is_empty() {
            clear_artist_banner_image(conn, id)?;
        } else {
            update_artist_banner_image(conn, id, bi)?;
        }
    }
    get_artist(conn, id)
}

pub fn update_album_full(conn: &Connection, id: i64, name: Option<&str>, cover_art: Option<&str>) -> Result<Album> {
    if let Some(n) = name {
        rename_album(conn, id, n)?;
    }
    if let Some(ca) = cover_art {
        if ca.is_empty() {
            update_album_cover(conn, id, None)?;
        } else {
            update_album_cover(conn, id, Some(ca))?;
        }
    }
    get_album(conn, id)
}

pub fn get_all_playlist_tracks(conn: &Connection) -> Result<Vec<(i64, Vec<i64>)>> {
    let mut stmt = conn
        .prepare("SELECT playlist_id, track_id FROM playlist_track ORDER BY playlist_id, position")
        .map_err(Error::Db)?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(Error::Db)?;

    let mut map: std::collections::HashMap<i64, Vec<i64>> = std::collections::HashMap::new();
    for row in rows {
        let (pid, tid) = row.map_err(Error::Db)?;
        map.entry(pid).or_default().push(tid);
    }
    Ok(map.into_iter().collect())
}

use crate::models::PlaylistWithCovers;

pub fn get_all_playlists_with_covers(conn: &Connection) -> Result<Vec<PlaylistWithCovers>> {
    let playlists = get_all_playlists(conn)?;
    let mut result = Vec::with_capacity(playlists.len());
    for p in playlists {
        let cover_arts = get_playlist_cover_arts(conn, p.id)?;
        result.push(PlaylistWithCovers {
            id: p.id,
            name: p.name,
            cover_art: p.cover_art,
            cover_arts,
        });
    }
    Ok(result)
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
            })
        },
    )
    .map_err(Error::Db)
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
                },
                album_artist_name,
            })
        })
        .map_err(Error::Db)?;

    let mut raw_tracks: Vec<RawTrack> = rows
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?;

    if !raw_tracks.is_empty() {
        // Batch-resolve album artists
        let names: Vec<&str> = raw_tracks
            .iter()
            .filter_map(|r| r.album_artist_name.as_deref())
            .collect();
        if !names.is_empty() {
            let mut artist_map = std::collections::HashMap::<String, Vec<Artist>>::new();
            let placeholders = names.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!(
                "SELECT ar.name, ar.id, ar.name, ar.profile_image, ar.banner_image
                 FROM artist ar
                 WHERE ar.name IN ({})",
                placeholders
            );
            if let Ok(mut stmt) = conn.prepare(&sql) {
                if let Ok(rows) = stmt.query_map(rusqlite::params_from_iter(&names), |row| {
                    let name: String = row.get(0)?;
                    Ok((
                        name,
                        Artist {
                            id: row.get(1)?,
                            name: row.get(2)?,
                            profile_image: row.get(3)?,
                            banner_image: row.get(4)?,
                        },
                    ))
                }) {
                    for row in rows {
                        if let Ok((name, artist)) = row {
                            artist_map.entry(name).or_default().push(artist);
                        }
                    }
                }
            }
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
        .query_map(params![limit as i64], |row| {
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
        .query_row("SELECT COALESCE(SUM(file_size), 0) FROM track", [], |row| {
            row.get(0)
        })
        .map_err(Error::Db)?;

    let largest_file: f64 = conn
        .query_row("SELECT COALESCE(MAX(file_size), 0) FROM track", [], |row| {
            row.get::<_, i64>(0)
        })
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
            "SELECT t.id, t.title, al.id, al.name, al.cover_art, alt.track_number, al.album_artist, al.year,
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

    struct RawTopTrack {
        top: TopTrack,
        album_artist_name: Option<String>,
    }

    let mut raw_list = Vec::new();
    let rows = stmt
        .query_map(params![limit as i64], |row| {
            let album_id: Option<i64> = row.get(2)?;
            let album_title: Option<String> = row.get(3)?;
            let album_art: Option<String> = row.get(4)?;
            let album_artist_name: Option<String> = row.get(6)?;
            let album_year: Option<i32> = row.get(7)?;

            let track = Track {
                id: row.get(0)?,
                title: row.get(1)?,
                artists: vec![],
                album: Album {
                    id: album_id.unwrap_or(0),
                    name: album_title.unwrap_or_else(|| "Unknown Album".to_string()),
                    cover_art: album_art,
                    album_artist: None,
                    year: album_year.map(|y| y as u32),
                },
                duration_seconds: row.get(8)?,
                is_favorite: row.get(9)?,
                cover_art: row.get(10)?,
                added_at: row.get(11)?,
                track_number: row.get::<_, Option<i32>>(5)?.map(|n| n as u32),
            };

            let play_count: i64 = row.get(12)?;
            let total_listening_time_sec: f64 = row.get(13)?;
            let last_played_at: Option<DateTime<Utc>> = row.get(14)?;

            Ok(RawTopTrack {
                top: TopTrack {
                    track,
                    play_count,
                    total_listening_time_sec: total_listening_time_sec as i64,
                    last_played_at,
                },
                album_artist_name,
            })
        })
        .map_err(Error::Db)?;

    for row in rows {
        raw_list.push(row.map_err(Error::Db)?);
    }

    // Resolve album artists
    if !raw_list.is_empty() {
        let names: Vec<&str> = raw_list
            .iter()
            .filter_map(|r| r.album_artist_name.as_deref())
            .collect();
        if !names.is_empty() {
            let mut album_artist_map = std::collections::HashMap::<String, Vec<Artist>>::new();
            let placeholders = names.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            if let Ok(mut stmt) = conn.prepare(&format!(
                "SELECT ar.name, ar.id, ar.name, ar.profile_image, ar.banner_image
                 FROM artist ar WHERE ar.name IN ({})",
                placeholders
            )) {
                if let Ok(rows) = stmt.query_map(rusqlite::params_from_iter(&names), |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        Artist {
                            id: row.get(1)?,
                            name: row.get(2)?,
                            profile_image: row.get(3)?,
                            banner_image: row.get(4)?,
                        },
                    ))
                }) {
                    for row in rows {
                        if let Ok((name, artist)) = row {
                            album_artist_map.entry(name).or_default().push(artist);
                        }
                    }
                }
            }
            for rt in &mut raw_list {
                if let Some(ref name) = rt.album_artist_name {
                    if let Some(artists) = album_artist_map.get(name) {
                        rt.top.track.album.album_artist = Some(artists.clone());
                    }
                }
            }
        }

        let track_ids: Vec<i64> = raw_list.iter().map(|r| r.top.track.id).collect();
        let artists_map = get_artists_for_tracks(conn, &track_ids)?;
        for rt in &mut raw_list {
            if let Some(artists) = artists_map.get(&rt.top.track.id) {
                rt.top.track.artists = artists.clone();
            }
        }
    }

    Ok(raw_list.into_iter().map(|r| r.top).collect())
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
        .query_map(params![limit as i64], |row| {
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
             ORDER BY play_count DESC
             LIMIT ?",
            time_filter
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
        .prepare("SELECT DISTINCT date(played_at) FROM playback_history ORDER BY played_at ASC")
        .map_err(Error::Db)?;

    let all_dates: Vec<String> = all_stmt
        .query_map([], |row| row.get(0))
        .map_err(Error::Db)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Error::Db)?;

    let longest = compute_longest_streak(&all_dates);

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

pub fn get_library_growth(conn: &Connection, _timeframe: Timeframe) -> Result<Vec<GrowthPoint>> {
    let (_data_span_days, period_fmt): (i64, String) = conn
        .query_row(
            "SELECT
                COALESCE(CAST(JULIANDAY(MAX(added_at)) - JULIANDAY(MIN(added_at)) AS INTEGER), 0),
                CASE
                    WHEN JULIANDAY(MAX(added_at)) - JULIANDAY(MIN(added_at)) < 31 THEN '%Y-%m-%d'
                    WHEN JULIANDAY(MAX(added_at)) - JULIANDAY(MIN(added_at)) < 365 THEN '%Y-%W'
                    ELSE '%Y-%m'
                END
             FROM track",
            [],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
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

pub fn get_favorite_trends(conn: &Connection, timeframe: Timeframe) -> Result<Vec<FavoriteTrend>> {
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
            "SELECT ph.played_at, t.id, t.title, al.id, al.name, al.cover_art, alt.track_number, al.album_artist, al.year,
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

    struct RawEvent {
        event: PlaybackEvent,
        album_artist_name: Option<String>,
    }

    let mut raw_events = Vec::new();
    let rows = stmt
        .query_map(params![limit as i64], |row| {
            let album_id: Option<i64> = row.get(3)?;
            let album_title: Option<String> = row.get(4)?;
            let album_art: Option<String> = row.get(5)?;
            let album_artist_name: Option<String> = row.get(7)?;
            let album_year: Option<i32> = row.get(8)?;

            let track = Track {
                id: row.get(1)?,
                title: row.get(2)?,
                artists: vec![],
                album: Album {
                    id: album_id.unwrap_or(0),
                    name: album_title.unwrap_or_else(|| "Unknown Album".to_string()),
                    cover_art: album_art,
                    album_artist: None,
                    year: album_year.map(|y| y as u32),
                },
                duration_seconds: row.get(9)?,
                is_favorite: row.get(10)?,
                cover_art: row.get(11)?,
                added_at: row.get(12)?,
                track_number: row.get::<_, Option<i32>>(6)?.map(|n| n as u32),
            };

            Ok(RawEvent {
                event: PlaybackEvent {
                    played_at: row.get(0)?,
                    track,
                    completion_percent: row.get(13)?,
                    source_type: row.get::<_, String>(14)?.to_lowercase(),
                },
                album_artist_name,
            })
        })
        .map_err(Error::Db)?;

    for row in rows {
        raw_events.push(row.map_err(Error::Db)?);
    }

    if !raw_events.is_empty() {
        let names: Vec<&str> = raw_events
            .iter()
            .filter_map(|r| r.album_artist_name.as_deref())
            .collect();
        if !names.is_empty() {
            let mut album_artist_map = std::collections::HashMap::<String, Vec<Artist>>::new();
            let placeholders = names.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            if let Ok(mut stmt) = conn.prepare(&format!(
                "SELECT ar.name, ar.id, ar.name, ar.profile_image, ar.banner_image
                 FROM artist ar WHERE ar.name IN ({})",
                placeholders
            )) {
                if let Ok(rows) = stmt.query_map(rusqlite::params_from_iter(&names), |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        Artist {
                            id: row.get(1)?,
                            name: row.get(2)?,
                            profile_image: row.get(3)?,
                            banner_image: row.get(4)?,
                        },
                    ))
                }) {
                    for row in rows {
                        if let Ok((name, artist)) = row {
                            album_artist_map.entry(name).or_default().push(artist);
                        }
                    }
                }
            }
            for re in &mut raw_events {
                if let Some(ref name) = re.album_artist_name {
                    if let Some(artists) = album_artist_map.get(name) {
                        re.event.track.album.album_artist = Some(artists.clone());
                    }
                }
            }
        }

        let track_ids: Vec<i64> = raw_events.iter().map(|e| e.event.track.id).collect();
        let artists_map = get_artists_for_tracks(conn, &track_ids)?;
        for re in &mut raw_events {
            if let Some(artists) = artists_map.get(&re.event.track.id) {
                re.event.track.artists = artists.clone();
            }
        }
    }

    Ok(raw_events.into_iter().map(|r| r.event).collect())
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
    fn migrations_validate() {
        assert!(MIGRATIONS.validate().is_ok());
    }
}
