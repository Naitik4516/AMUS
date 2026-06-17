use crate::db::{self, DbPool, SortBy, Timeframe};
use crate::engine::Player;
use crate::error::{Error, Result};
use crate::models::*;
use crate::scanner;
use crate::sync::SyncManager;
use tauri::State;

#[tauri::command]
pub async fn add_source(
    path: String,
    pool: State<'_, DbPool>,
    sync_manager: State<'_, SyncManager>,
    app_handle: tauri::AppHandle,
) -> Result<()> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::add_source_dir(&conn, &path)?;
    let _ = sync_manager.refresh_watcher(&app_handle);
    Ok(())
}

#[tauri::command]
pub async fn get_source_dirs(pool: State<'_, DbPool>) -> Result<Vec<String>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_source_dirs(&conn)
}

#[tauri::command]
pub async fn remove_source(
    path: String,
    pool: State<'_, DbPool>,
    sync_manager: State<'_, SyncManager>,
    app_handle: tauri::AppHandle,
) -> Result<()> {
    let mut conn = pool.get().map_err(Error::Pool)?;
    db::remove_source_dir(&mut conn, &path)?;
    let _ = sync_manager.refresh_watcher(&app_handle);
    Ok(())
}

#[tauri::command]
pub async fn refresh_watcher(
    sync_manager: State<'_, SyncManager>,
    app_handle: tauri::AppHandle,
) -> Result<()> {
    sync_manager
        .refresh_watcher(&app_handle)
        .map_err(|e| Error::Unknown(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn has_music(pool: State<'_, DbPool>) -> Result<bool> {
    let conn = pool.get().map_err(Error::Pool)?;
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0))
        .map_err(Error::Db)?;
    Ok(count > 0)
}

#[tauri::command]
pub async fn scan_library(app_handle: tauri::AppHandle, pool: State<'_, DbPool>) -> Result<()> {
    let mut conn = pool.get().map_err(Error::Pool)?;
    scanner::scan_directories(&mut conn, &app_handle)?;
    Ok(())
}

#[tauri::command]
pub async fn get_all_tracks(
    sort_by: Option<SortBy>,
    pool: State<'_, DbPool>,
) -> Result<Vec<Track>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_all_tracks(&conn, sort_by)
}

#[tauri::command]
pub async fn get_favorite_tracks(
    sort_by: Option<SortBy>,
    pool: State<'_, DbPool>,
) -> Result<Vec<Track>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_favorite_tracks(&conn, sort_by)
}

#[tauri::command]
pub async fn get_recently_played(limit: usize, pool: State<'_, DbPool>) -> Result<Vec<Track>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_recently_played_tracks(&conn, limit)
}

#[tauri::command]
pub async fn get_most_played_tracks(
    timeframe: Timeframe,
    limit: usize,
    pool: State<'_, DbPool>,
) -> Result<Vec<Track>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_most_played_tracks(&conn, limit, timeframe)
}

#[tauri::command]
pub async fn get_track_details(id: i64, pool: State<'_, DbPool>) -> Result<TrackDetails> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_track_details(&conn, id)
}

#[tauri::command]
pub async fn search_tracks(
    query: String,
    limit: usize,
    pool: State<'_, DbPool>,
) -> Result<Vec<Track>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::search_tracks(&conn, &query, limit)
}

#[tauri::command]
pub async fn get_artists(pool: State<'_, DbPool>) -> Result<Vec<Artist>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_all_artists(&conn)
}

#[tauri::command]
pub async fn get_artist(id: i64, pool: State<'_, DbPool>) -> Result<Artist> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_artist(&conn, id)
}

#[tauri::command]
pub async fn get_all_albums(pool: State<'_, DbPool>) -> Result<Vec<Album>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_all_albums(&conn)
}

#[tauri::command]
pub async fn get_album(id: i64, pool: State<'_, DbPool>) -> Result<Album> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_album(&conn, id)
}

#[tauri::command]
pub async fn get_albums(artist_id: i64, pool: State<'_, DbPool>) -> Result<Vec<Album>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_albums_by_artist(&conn, artist_id)
}

#[tauri::command]
pub async fn get_tracks_by_album(
    album_id: i64,
    sort_by: Option<SortBy>,
    pool: State<'_, DbPool>,
) -> Result<Vec<Track>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_tracks_by_album(&conn, album_id, sort_by)
}

#[tauri::command]
pub async fn get_tracks_by_artist(
    artist_id: i64,
    sort_by: Option<SortBy>,
    pool: State<'_, DbPool>,
) -> Result<Vec<Track>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_tracks_by_artist(&conn, artist_id, sort_by)
}

#[tauri::command]
pub async fn get_playlists(pool: State<'_, DbPool>) -> Result<Vec<Playlist>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_all_playlists(&conn)
}

#[tauri::command]
pub async fn get_tracks_by_playlist(
    playlist_id: i64,
    sort_by: Option<SortBy>,
    pool: State<'_, DbPool>,
) -> Result<Vec<Track>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_tracks_in_playlist(&conn, playlist_id, sort_by)
}

#[tauri::command]
pub async fn create_playlist(name: String, pool: State<'_, DbPool>) -> Result<()> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::create_playlist(&conn, &name)
}

#[tauri::command]
pub async fn add_track_to_playlist(
    track_id: i64,
    playlist_id: i64,
    pool: State<'_, DbPool>,
) -> Result<()> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::add_track_to_playlist(&conn, playlist_id, track_id)
}

#[tauri::command]
pub async fn remove_track_from_playlist(
    track_id: i64,
    playlist_id: i64,
    pool: State<'_, DbPool>,
) -> Result<()> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::remove_track_from_playlist(&conn, playlist_id, track_id)
}

#[tauri::command]
pub async fn delete_playlist(playlist_id: i64, pool: State<'_, DbPool>) -> Result<()> {
    let mut conn = pool.get().map_err(Error::Pool)?;
    let tx = conn.transaction().map_err(Error::Db)?;
    db::delete_playlist(&tx, playlist_id)?;
    tx.commit().map_err(Error::Db)?;
    Ok(())
}

#[tauri::command]
pub async fn rename_playlist(
    playlist_id: i64,
    new_name: String,
    pool: State<'_, DbPool>,
) -> Result<()> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::rename_playlist(&conn, playlist_id, &new_name)
}

#[tauri::command]
pub async fn toggle_favorite(id: i64, pool: State<'_, DbPool>) -> Result<bool> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::toggle_favorite(&conn, id)
}

#[tauri::command]
pub async fn get_similar_songs(id: i64, pool: State<'_, DbPool>) -> Result<Vec<Track>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_similar_tracks(&conn, id, 20)
}

#[tauri::command]
pub async fn get_playlist_cover_arts(
    playlist_id: i64,
    pool: State<'_, DbPool>,
) -> Result<Vec<String>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_playlist_cover_arts(&conn, playlist_id)
}

// Playback controls

#[tauri::command]
pub async fn play_track(
    id: i64,
    pool: State<'_, DbPool>,
    player: State<'_, Player>,
) -> Result<TrackDetails> {
    let conn = pool.get().map_err(Error::Pool)?;
    let track = db::get_track_details(&conn, id)?;
    let mut engine = player.engine.lock();
    engine
        .play(&track)
        .map_err(|e| Error::Audio(e.to_string()))?;

    Ok(track)
}

#[tauri::command]
pub async fn toggle_playback(player: State<'_, Player>, playing: bool) -> Result<()> {
    let engine = player.engine.lock();
    if playing {
        engine.resume();
    } else {
        engine.pause();
    }
    Ok(())
}

#[tauri::command]
pub async fn set_volume(volume: u32, player: State<'_, Player>) -> Result<()> {
    let mut engine = player.engine.lock();
    engine.set_volume(volume as f32 / 100.0);
    Ok(())
}

#[tauri::command]
pub async fn seek(seconds: u64, player: State<'_, Player>) -> Result<()> {
    let engine = player.engine.lock();
    engine
        .seek(seconds)
        .map_err(|e| Error::Audio(e.to_string()))
}
