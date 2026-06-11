use crate::db::{self, DbPool, Timeframe};
use crate::engine::Player;
use crate::models::*;
use crate::scanner;
use anyhow::Result;
use tauri::State;

#[tauri::command]
pub async fn add_source(path: String, pool: State<'_, DbPool>) -> Result<(), String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    db::add_source_dir(&conn, &path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn scan_library(pool: State<'_, DbPool>) -> Result<(), String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    scanner::scan_directories(&conn).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_all_tracks(pool: State<'_, DbPool>) -> Result<Vec<Track>, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    db::get_all_tracks(&conn).map_err(|e| e.to_string())
}

pub async fn get_favorite_tracks(pool: State<'_, DbPool>) -> Result<Vec<Track>, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    db::get_favorite_tracks(&conn).map_err(|e| e.to_string())
}

pub async fn get_recently_played(
    limit: usize,
    pool: State<'_, DbPool>,
) -> Result<Vec<Track>, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    db::get_recently_played(&conn, limit).map_err(|e| e.to_string())
}

pub async fn get_top_tracks(
    timeframe: Timeframe,
    pool: State<'_, DbPool>,
) -> Result<Vec<Track>, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    db::get_top_tracks(&conn, 50, timeframe).map_err(|e| e.to_string())
}

pub async fn get_track_details(id: i64, pool: State<'_, DbPool>) -> Result<TrackDetails, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    db::get_track_details(&conn, id).map_err(|e| e.to_string())
}

pub async fn search_tracks(query: String, pool: State<'_, DbPool>) -> Result<Vec<Track>, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    db::search_tracks(&conn, &query, 10).map_err(|e| e.to_string())
}

pub async fn get_artists(pool: State<'_, DbPool>) -> Result<Vec<Artist>, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    db::get_all_artists(&conn).map_err(|e| e.to_string())
}

pub async fn get_albums(artist_id: i64, pool: State<'_, DbPool>) -> Result<Vec<Album>, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    db::get_albums_by_artist(&conn, artist_id).map_err(|e| e.to_string())
}

pub async fn get_tracks_by_album(
    album_id: i64,
    pool: State<'_, DbPool>,
) -> Result<Vec<Track>, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    db::get_tracks_by_album(&conn, album_id).map_err(|e| e.to_string())
}

pub async fn get_playlists(pool: State<'_, DbPool>) -> Result<Vec<Playlist>, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    db::get_all_playlists(&conn).map_err(|e| e.to_string())
}

pub async fn get_tracks_by_playlist(
    playlist_id: i64,
    pool: State<'_, DbPool>,
) -> Result<Vec<Track>, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    db::get_tracks_in_playlist(&conn, playlist_id).map_err(|e| e.to_string())
}

pub async fn create_playlist(name: String, pool: State<'_, DbPool>) -> Result<(), String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    db::create_playlist(&conn, &name).map_err(|e| e.to_string())
}

pub async fn add_track_to_playlist(
    track_id: i64,
    playlist_id: i64,
    pool: State<'_, DbPool>,
) -> Result<(), String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    db::add_track_to_playlist(&conn, track_id, playlist_id).map_err(|e| e.to_string())
}

pub async fn remove_track_from_playlist(
    track_id: i64,
    playlist_id: i64,
    pool: State<'_, DbPool>,
) -> Result<(), String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    db::remove_track_from_playlist(&conn, track_id, playlist_id).map_err(|e| e.to_string())
}

pub async fn delete_playlist(playlist_id: i64, pool: State<'_, DbPool>) -> Result<(), String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    db::delete_playlist(&conn, playlist_id).map_err(|e| e.to_string())
}

pub async fn rename_playlist(
    playlist_id: i64,
    new_name: String,
    pool: State<'_, DbPool>,
) -> Result<(), String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    db::rename_playlist(&conn, playlist_id, &new_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_favorite(id: i64, pool: State<'_, DbPool>) -> Result<bool, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    db::toggle_favorite(&conn, id).map_err(|e| e.to_string())
}

pub async fn get_similar_tracks(id: i64, pool: State<'_, DbPool>) -> Result<Vec<Track>, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    db::get_similar_tracks(&conn, id, 20).map_err(|e| e.to_string())
}

// Playback controls

#[tauri::command]
pub async fn play_track(
    id: i64,
    pool: State<'_, DbPool>,
    player: State<'_, Player>,
) -> Result<TrackDetails, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    let track = db::get_track_details(&conn, id).map_err(|e| e.to_string())?;
    let mut engine = player.engine.lock();
    engine.play(&track).map_err(|e| e.to_string())?;

    Ok(track)
}

#[tauri::command]
pub async fn toggle_playback(player: State<'_, Player>, playing: bool) -> Result<(), String> {
    let engine = player.engine.lock();
    if playing {
        engine.resume();
    } else {
        engine.pause();
    }
    Ok(())
}

#[tauri::command]
pub async fn set_volume(volume: u32, player: State<'_, Player>) -> Result<(), String> {
    let mut engine = player.engine.lock();
    engine.set_volume(volume as f32 / 100.0);
    Ok(())
}

#[tauri::command]
pub async fn seek(seconds: u64, player: State<'_, Player>) -> Result<(), String> {
    let engine = player.engine.lock();
    engine.seek(seconds).map_err(|e| e.to_string())
}
