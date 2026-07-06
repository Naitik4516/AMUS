use crate::MiniPlayerPinned;
use crate::artist_pic_fetcher;
use crate::db::{self, DataAge, DbPool, SortBy, Timeframe};
use crate::error::{Error, Result};
use crate::models::*;
use crate::player::actor::{PlayerCommand, PlayerStateSnapshot};
use crate::player::source::{PlaybackSource, RepeatMode};
use crate::scanner;
use crate::sync::SyncManager;
use std::sync::mpsc::Sender;
use tauri::Manager;
use tauri::State;
use tokio::sync::oneshot;

pub struct PlayerHandle(pub Sender<PlayerCommand>);

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
    let exists: bool = conn
        .query_row("SELECT EXISTS(SELECT 1 FROM track LIMIT 1)", [], |row| {
            row.get(0)
        })
        .map_err(Error::Db)?;
    Ok(exists)
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
pub async fn global_search(
    query: String,
    limit: usize,
    pool: State<'_, DbPool>,
) -> Result<Vec<GlobalSearchResult>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::global_search(&conn, &query, limit)
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
pub async fn create_playlist(name: String, pool: State<'_, DbPool>) -> Result<i64> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::create_playlist(&conn, &name)
}

#[tauri::command]
pub async fn get_track_playlist_ids(track_id: i64, pool: State<'_, DbPool>) -> Result<Vec<i64>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_track_playlist_ids(&conn, track_id)
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

#[tauri::command]
pub async fn get_playlist(
    playlist_id: i64,
    pool: State<'_, DbPool>,
) -> Result<(i64, String, Option<String>)> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_playlist(&conn, playlist_id)
}

// ---------------------------------------------------------------------------
// Playback controls
// ---------------------------------------------------------------------------

fn send(handle: &State<PlayerHandle>, cmd: PlayerCommand) -> Result<()> {
    handle.0.send(cmd).map_err(|e| Error::Audio(e.to_string()))
}

#[tauri::command]
pub fn play_context(
    handle: State<PlayerHandle>,
    tracks: Vec<Track>,
    source_type: String,
    source_id: Option<i64>,
    start_index: usize,
    context_label: Option<String>,
) -> Result<()> {
    let source = PlaybackSource::from_db(&source_type, source_id);
    send(
        &handle,
        PlayerCommand::LoadContext {
            tracks,
            source,
            start_index,
            context_label,
        },
    )
}

#[tauri::command]
pub fn play_pause(handle: State<PlayerHandle>) -> Result<()> {
    send(&handle, PlayerCommand::PlayPause)
}

#[tauri::command]
pub fn next(handle: State<PlayerHandle>) -> Result<()> {
    send(&handle, PlayerCommand::Next)
}

#[tauri::command]
pub fn previous(handle: State<PlayerHandle>) -> Result<()> {
    send(&handle, PlayerCommand::Previous)
}

#[tauri::command]
pub fn seek(handle: State<PlayerHandle>, position_sec: f64) -> Result<()> {
    send(&handle, PlayerCommand::Seek(position_sec))
}

#[tauri::command]
pub fn set_volume(handle: State<PlayerHandle>, volume: f32) -> Result<()> {
    send(&handle, PlayerCommand::SetVolume(volume))
}

#[tauri::command]
pub fn set_repeat(handle: State<PlayerHandle>, mode: String) -> Result<()> {
    send(
        &handle,
        PlayerCommand::SetRepeat(RepeatMode::from_str(&mode)),
    )
}

#[tauri::command]
pub fn toggle_shuffle(handle: State<PlayerHandle>) -> Result<()> {
    send(&handle, PlayerCommand::ToggleShuffle)
}

#[tauri::command]
pub fn enqueue_next(handle: State<PlayerHandle>, track: Track) -> Result<()> {
    send(&handle, PlayerCommand::EnqueueNext(track))
}

#[tauri::command]
pub fn enqueue_end(handle: State<PlayerHandle>, track: Track) -> Result<()> {
    send(&handle, PlayerCommand::EnqueueEnd(track))
}

#[tauri::command]
pub fn remove_from_queue(handle: State<PlayerHandle>, queue_id: i64) -> Result<()> {
    send(&handle, PlayerCommand::RemoveFromQueue(queue_id))
}

#[tauri::command]
pub fn clear_queue(handle: State<PlayerHandle>) -> Result<()> {
    send(&handle, PlayerCommand::ClearQueue)
}

#[tauri::command]
pub fn reorder_queue(handle: State<PlayerHandle>, queue_id: i64, new_index: usize) -> Result<()> {
    send(
        &handle,
        PlayerCommand::ReorderQueue {
            queue_id,
            new_index,
        },
    )
}

#[tauri::command]
pub fn set_autoplay(handle: State<PlayerHandle>, enabled: bool) -> Result<()> {
    send(&handle, PlayerCommand::SetAutoplay(enabled))
}

#[tauri::command]
pub async fn get_current_state(handle: State<'_, PlayerHandle>) -> Result<PlayerStateSnapshot> {
    let (tx, rx) = oneshot::channel();
    handle
        .0
        .send(PlayerCommand::GetState(tx))
        .map_err(|e| Error::Audio(e.to_string()))?;
    rx.await
        .map_err(|_| Error::Unknown("channel closed".to_string()))
}

#[tauri::command]
pub async fn get_top_artists(limit: usize, pool: State<'_, DbPool>) -> Result<Vec<Artist>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_top_artists(&conn, limit)
}

#[tauri::command]
pub async fn get_top_albums(limit: usize, pool: State<'_, DbPool>) -> Result<Vec<Album>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_top_albums(&conn, limit)
}

#[tauri::command]
pub async fn get_forgotten_tracks(limit: usize, pool: State<'_, DbPool>) -> Result<Vec<Track>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_forgotten_tracks(&conn, limit)
}

#[tauri::command]
pub async fn get_unplayed_tracks(limit: usize, pool: State<'_, DbPool>) -> Result<Vec<Track>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_unplayed_tracks(&conn, limit)
}

#[tauri::command]
pub async fn get_recently_added(limit: usize, pool: State<'_, DbPool>) -> Result<Vec<Track>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_recently_added_tracks(&conn, limit)
}

// ---------------------------------------------------------------------------
// Stats commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_stats_overview(
    timeframe: Timeframe,
    pool: State<'_, DbPool>,
) -> Result<StatsOverview> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_stats_overview(&conn, timeframe)
}

#[tauri::command]
pub async fn get_top_tracks_with_stats(
    timeframe: Timeframe,
    limit: usize,
    pool: State<'_, DbPool>,
) -> Result<Vec<TopTrack>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_top_tracks_with_stats(&conn, timeframe, limit)
}

#[tauri::command]
pub async fn get_top_artists_with_stats(
    timeframe: Timeframe,
    limit: usize,
    pool: State<'_, DbPool>,
) -> Result<Vec<TopArtist>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_top_artists_with_stats(&conn, timeframe, limit)
}

#[tauri::command]
pub async fn get_top_albums_with_stats(
    timeframe: Timeframe,
    limit: usize,
    pool: State<'_, DbPool>,
) -> Result<Vec<TopAlbum>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_top_albums_with_stats(&conn, timeframe, limit)
}

#[tauri::command]
pub async fn get_listening_time_trend(
    timeframe: Timeframe,
    pool: State<'_, DbPool>,
) -> Result<Vec<TimeSeriesPoint>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_listening_time_trend(&conn, timeframe)
}

#[tauri::command]
pub async fn get_streak_data(timeframe: Timeframe, pool: State<'_, DbPool>) -> Result<StreakData> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_streak_data(&conn, timeframe)
}

#[tauri::command]
pub async fn get_library_growth(
    timeframe: Timeframe,
    pool: State<'_, DbPool>,
) -> Result<Vec<GrowthPoint>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_library_growth(&conn, timeframe)
}

#[tauri::command]
pub async fn get_format_distribution(pool: State<'_, DbPool>) -> Result<Vec<FormatStat>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_format_distribution(&conn)
}

#[tauri::command]
pub async fn get_data_age(pool: State<'_, DbPool>) -> Result<DataAge> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_data_age(&conn)
}

#[tauri::command]
pub async fn get_heatmap_hourly(
    timeframe: Timeframe,
    pool: State<'_, DbPool>,
) -> Result<Vec<HeatmapCell>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_heatmap_hourly(&conn, timeframe)
}

#[tauri::command]
pub async fn get_heatmap_weekday(
    timeframe: Timeframe,
    pool: State<'_, DbPool>,
) -> Result<Vec<HeatmapCell>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_heatmap_weekday(&conn, timeframe)
}

#[tauri::command]
pub async fn get_favorite_trends(
    timeframe: Timeframe,
    pool: State<'_, DbPool>,
) -> Result<Vec<FavoriteTrend>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_favorite_trends(&conn, timeframe)
}

#[tauri::command]
pub async fn get_playback_history_timeline(
    timeframe: Timeframe,
    limit: usize,
    pool: State<'_, DbPool>,
) -> Result<Vec<PlaybackEvent>> {
    let conn = pool.get().map_err(Error::Pool)?;
    db::get_playback_history_timeline(&conn, timeframe, limit)
}

#[tauri::command]
pub async fn fetch_artist_images(
    artist_id: i64,
    app_handle: tauri::AppHandle,
    pool: State<'_, DbPool>,
) -> Result<()> {
    let conn = pool.get().map_err(Error::Pool)?;
    let artist = db::get_artist(&conn, artist_id)?;

    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| Error::Unknown(e.to_string()))?;

    let has_photo = db::artist_has_photo(&conn, artist_id).unwrap_or(false);
    let has_banner = db::artist_has_banner(&conn, artist_id).unwrap_or(false);

    if !has_photo || !has_banner {
        artist_pic_fetcher::fetch_single_artist_images(
            artist_id,
            &artist.name,
            &app_dir,
            pool.inner().clone(),
            true,
            true,
        )
        .await
        .map_err(|e| Error::Unknown(e.to_string()))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn save_image(
    source_path: String,
    image_type: String,
    app_handle: tauri::AppHandle,
) -> Result<String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| Error::Unknown(e.to_string()))?;

    let subdir = match image_type.as_str() {
        "cover" => "covers",
        "artist" => "artists",
        "banner" => "artist_banner",
        _ => return Err(Error::Unknown(format!("Unknown image type: {image_type}"))),
    };

    scanner::save_image_to_app_dir(&app_dir, &source_path, subdir)
}

#[tauri::command]
pub async fn update_artist(
    id: i64,
    name: Option<String>,
    profile_image: Option<String>,
    banner_image: Option<String>,
    pool: State<'_, DbPool>,
) -> Result<()> {
    let conn = pool.get().map_err(Error::Pool)?;
    if let Some(n) = name {
        db::rename_artist(&conn, id, &n)?;
    }
    if let Some(pi) = profile_image {
        if pi.is_empty() {
            db::clear_artist_profile_image(&conn, id)?;
        } else {
            db::update_artist_profile_image(&conn, id, &pi)?;
        }
    }
    if let Some(bi) = banner_image {
        if bi.is_empty() {
            db::clear_artist_banner_image(&conn, id)?;
        } else {
            db::update_artist_banner_image(&conn, id, &bi)?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn update_album(
    id: i64,
    name: Option<String>,
    cover_art: Option<String>,
    pool: State<'_, DbPool>,
) -> Result<()> {
    let conn = pool.get().map_err(Error::Pool)?;
    if let Some(n) = name {
        db::rename_album(&conn, id, &n)?;
    }
    if let Some(ca) = cover_art {
        if ca.is_empty() {
            db::update_album_cover(&conn, id, None)?;
        } else {
            db::update_album_cover(&conn, id, Some(&ca))?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn update_playlist(
    id: i64,
    name: Option<String>,
    cover_art: Option<String>,
    pool: State<'_, DbPool>,
) -> Result<()> {
    let conn = pool.get().map_err(Error::Pool)?;
    let ca = cover_art.as_deref();
    db::update_playlist(&conn, id, name.as_deref(), ca)?;
    Ok(())
}

#[tauri::command]
pub(crate) fn toggle_mini_player_pin(
    app: tauri::AppHandle,
    state: tauri::State<'_, MiniPlayerPinned>,
) -> std::result::Result<bool, String> {
    let new_pinned = !state.0.load(std::sync::atomic::Ordering::Relaxed);
    state
        .0
        .store(new_pinned, std::sync::atomic::Ordering::Relaxed);
    if let Some(window) = app.get_webview_window("mini-player") {
        window
            .set_always_on_top(new_pinned)
            .map_err(|e| format!("failed to set always-on-top: {e}"))?;
    }
    Ok(new_pinned)
}

#[tauri::command]
pub(crate) fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub(crate) fn toggle_mini_player(app: tauri::AppHandle) -> std::result::Result<(), String> {
    if let Some(window) = app.get_webview_window("mini-player") {
        if window.is_visible().map_err(|e| e.to_string())? {
            window.hide().map_err(|e| e.to_string())?;
        } else {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
        }
    } else {
        crate::toggle_popup(&app);
    }
    Ok(())
}
