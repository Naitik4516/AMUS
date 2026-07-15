//! Dispatch CLI commands against the running app state.

use std::path::Path;
use std::sync::mpsc::Sender;

use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::oneshot;

use super::protocol::{
    CliCommand, CliData, CliResponse, QueueTrackLine, SearchKind, SearchNamedLine, SearchTrackLine,
};
use crate::commands::PlayerHandle;
use crate::db::{self, DbPool};
use crate::models::Track;
use crate::player::actor::PlayerCommand;
use crate::player::source::PlaybackSource;
use crate::scanner;
use crate::sync::SyncManager;


/// Public entry: play a set of file paths (auto-imports missing tracks).
pub fn play_paths(app: &AppHandle, paths: &[String]) -> Result<(), String> {
    let tracks = resolve_paths_to_tracks(app, paths, true)?;
    if tracks.is_empty() {
        return Err("no tracks to play".into());
    }
    send_player(
        app,
        PlayerCommand::LoadContext {
            tracks,
            source: PlaybackSource::Direct,
            start_index: 0,
            context_label: Some("Files".into()),
        },
    )?;
    Ok(())
}

pub fn handle(app: &AppHandle, cmd: CliCommand, id: u64) -> CliResponse {
    match dispatch(app, cmd) {
        Ok(data) => CliResponse::ok(id, data),
        Err(e) => CliResponse::err(id, e),
    }
}

fn dispatch(app: &AppHandle, cmd: CliCommand) -> Result<CliData, String> {
    match cmd {
        CliCommand::Play => {
            send_player(app, PlayerCommand::Play)?;
            Ok(msg("Playing"))
        }
        CliCommand::Pause => {
            send_player(app, PlayerCommand::Pause)?;
            Ok(msg("Paused"))
        }
        CliCommand::Stop => {
            send_player(app, PlayerCommand::Stop)?;
            Ok(msg("Stopped"))
        }
        CliCommand::Toggle => {
            send_player(app, PlayerCommand::PlayPause)?;
            Ok(msg("Toggled"))
        }
        CliCommand::Next => {
            send_player(app, PlayerCommand::Next)?;
            Ok(msg("Next"))
        }
        CliCommand::Previous => {
            send_player(app, PlayerCommand::Previous)?;
            Ok(msg("Previous"))
        }
        CliCommand::Seek { value, relative } => {
            if relative {
                send_player(app, PlayerCommand::SeekRelative(value))?;
            } else {
                send_player(app, PlayerCommand::Seek(value.max(0.0)))?;
            }
            Ok(msg("Seeked"))
        }
        CliCommand::Volume { value, relative } => {
            if relative {
                // value is in percent points
                send_player(app, PlayerCommand::AdjustVolume((value / 100.0) as f32))?;
            } else {
                let v = (value / 100.0).clamp(0.0, 1.0) as f32;
                send_player(app, PlayerCommand::SetVolume(v))?;
            }
            Ok(msg("Volume updated"))
        }
        CliCommand::Mute => {
            send_player(app, PlayerCommand::ToggleMute)?;
            Ok(msg("Mute toggled"))
        }
        CliCommand::Status => status(app),
        CliCommand::QueueShow => queue_show(app),
        CliCommand::QueueClear => {
            send_player(app, PlayerCommand::ClearQueue)?;
            Ok(msg("Queue cleared"))
        }
        CliCommand::QueueShuffle => {
            send_player(app, PlayerCommand::ToggleShuffle)?;
            Ok(msg("Shuffle toggled"))
        }
        CliCommand::QueueAddPaths { paths } => {
            let tracks = resolve_paths_to_tracks(app, &paths, true)?;
            send_player(app, PlayerCommand::EnqueueEndMany(tracks))?;
            Ok(msg("Added to queue"))
        }
        CliCommand::QueueAddSearch { query, kind } => {
            let tracks = resolve_search_to_tracks(app, &query, kind)?;
            send_player(app, PlayerCommand::EnqueueEndMany(tracks))?;
            Ok(msg("Added search results to queue"))
        }
        CliCommand::PlayPaths { paths } => {
            let tracks = resolve_paths_to_tracks(app, &paths, true)?;
            if tracks.is_empty() {
                return Err("no tracks to play".into());
            }
            send_player(
                app,
                PlayerCommand::LoadContext {
                    tracks,
                    source: PlaybackSource::Direct,
                    start_index: 0,
                    context_label: Some("Files".into()),
                },
            )?;
            Ok(msg("Playing"))
        }
        CliCommand::PlaySearch { query, kind } => {
            let (tracks, source, label) = resolve_search_to_context(app, &query, kind)?;
            if tracks.is_empty() {
                return Err("no results".into());
            }
            send_player(
                app,
                PlayerCommand::LoadContext {
                    tracks,
                    source,
                    start_index: 0,
                    context_label: Some(label),
                },
            )?;
            Ok(msg("Playing"))
        }
        CliCommand::LibraryRescan => {
            let pool = pool(app)?;
            let mut conn = pool.get().map_err(|e| e.to_string())?;
            scanner::scan_directories(&mut conn, app).map_err(|e| e.to_string())?;
            Ok(msg("Library rescanned"))
        }
        CliCommand::Import { path } => {
            let pool = pool(app)?;
            let mut conn = pool.get().map_err(|e| e.to_string())?;
            db::add_source_dir(&conn, &path).map_err(|e| e.to_string())?;
            if let Some(sync) = app.try_state::<SyncManager>() {
                let _ = sync.refresh_watcher(app);
            }
            scanner::scan_directories(&mut conn, app).map_err(|e| e.to_string())?;
            Ok(msg(format!("Imported and scanned: {path}")))
        }
        CliCommand::Search { query, kind } => search(app, &query, kind),
        CliCommand::PlaylistCreate { name } => {
            let pool = pool(app)?;
            let conn = pool.get().map_err(|e| e.to_string())?;
            let pl = db::create_playlist(&conn, &name).map_err(|e| e.to_string())?;
            let _ = app.emit("library-updated", ());
            Ok(msg(format!("Created playlist \"{}\" (id {})", pl.name, pl.id)))
        }
        CliCommand::PlaylistAdd { playlist, path } => {
            playlist_add(app, &playlist, &path)
        }
        CliCommand::PlaylistRemove { playlist, path } => {
            playlist_remove(app, &playlist, &path)
        }
        CliCommand::PlaylistPlay { playlist } => playlist_play(app, &playlist),
        CliCommand::PlaylistDelete { playlist } => {
            let pool = pool(app)?;
            let conn = pool.get().map_err(|e| e.to_string())?;
            let pl = resolve_playlist(&conn, &playlist)?;
            db::delete_playlist(&conn, pl.id).map_err(|e| e.to_string())?;
            let _ = app.emit("library-updated", ());
            Ok(msg(format!("Deleted playlist \"{}\"", pl.name)))
        }
        CliCommand::PlaylistShow { playlist: None } => list_playlists(app),
        CliCommand::PlaylistShow {
            playlist: Some(name),
        } => show_playlist(app, &name),
        CliCommand::ListAlbums => list_albums(app),
        CliCommand::ListArtists => list_artists(app),
        CliCommand::ListPlaylists => list_playlists(app),
        CliCommand::ShowAlbum { id_or_name } => show_album(app, &id_or_name),
        CliCommand::ShowArtist { id_or_name } => show_artist(app, &id_or_name),
        CliCommand::Open | CliCommand::Show => {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
            Ok(msg("Window shown"))
        }
        CliCommand::Hide => {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.hide();
            }
            Ok(msg("Window hidden"))
        }
        CliCommand::Close => {
            if let Some(w) = app.get_webview_window("main") {
                let keep = crate::sync::get_setting(app, "keepRunningInBg", true).unwrap_or(true);
                if keep {
                    let _ = w.hide();
                    Ok(msg("Window hidden (running in background)"))
                } else {
                    app.exit(0);
                    Ok(msg("Closing"))
                }
            } else {
                Ok(msg("No window"))
            }
        }
        CliCommand::Update => run_update(app),
        CliCommand::Version => Ok(CliData::Version {
            version: app.package_info().version.to_string(),
        }),
        CliCommand::Reset { force: _ } => {
            let app_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("failed to resolve app data dir: {e}"))?;

            let _ = std::fs::remove_file(app_dir.join("music.db"));
            let _ = std::fs::remove_file(app_dir.join("music.db-wal"));
            let _ = std::fs::remove_file(app_dir.join("music.db-shm"));

            let _ = std::fs::remove_file(app_dir.join("session.json"));
            let _ = std::fs::remove_file(app_dir.join("settings.json"));

            for dir in &["artists", "artist_banner", "cover_art"] {
                let _ = std::fs::remove_dir_all(app_dir.join(dir));
            }

            // Spawn a fresh instance before exiting
            if let Ok(exe) = std::env::current_exe() {
                let _ = std::process::Command::new(exe).spawn();
            }
            app.exit(0);

            #[allow(unreachable_code)]
            Ok(msg("App data reset"))
        }
    }
}

fn msg(text: impl Into<String>) -> CliData {
    CliData::Message { text: text.into() }
}

fn pool(app: &AppHandle) -> Result<tauri::State<'_, DbPool>, String> {
    app.try_state::<DbPool>()
        .ok_or_else(|| "database not ready".to_string())
}

fn player_tx(app: &AppHandle) -> Result<Sender<PlayerCommand>, String> {
    let handle = app
        .try_state::<PlayerHandle>()
        .ok_or_else(|| "player not ready".to_string())?;
    Ok(handle.0.clone())
}

fn send_player(app: &AppHandle, cmd: PlayerCommand) -> Result<(), String> {
    player_tx(app)?
        .send(cmd)
        .map_err(|e| format!("player: {e}"))
}

fn get_state(
    app: &AppHandle,
) -> Result<crate::player::actor::PlayerStateSnapshot, String> {
    let (tx, rx) = oneshot::channel();
    send_player(app, PlayerCommand::GetState(tx))?;
    // Block with timeout via recv on blocking context (we're on a worker thread)
    rx.blocking_recv()
        .map_err(|_| "player state channel closed".to_string())
}

fn status(app: &AppHandle) -> Result<CliData, String> {
    let s = get_state(app)?;
    let (title, artist, album) = match &s.current_track {
        Some(t) => (
            Some(t.title.clone()),
            Some(
                t.artists
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            Some(t.album.name.clone()),
        ),
        None => (None, None, None),
    };
    Ok(CliData::Status {
        is_playing: s.is_playing,
        has_track: s.current_track.is_some(),
        title,
        artist,
        album,
        position_sec: s.position_sec,
        duration_sec: s.duration_sec,
        volume_percent: (s.volume * 100.0).round() as u32,
        muted: s.muted,
        shuffle: s.shuffle,
        repeat: s.repeat,
    })
}

fn queue_show(app: &AppHandle) -> Result<CliData, String> {
    let s = get_state(app)?;
    Ok(CliData::Queue {
        tracks: s.user_queue.iter().map(track_line).collect(),
    })
}

fn track_line(t: &Track) -> QueueTrackLine {
    QueueTrackLine {
        id: t.id,
        title: t.title.clone(),
        artist: t
            .artists
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>()
            .join(", "),
        album: t.album.name.clone(),
        duration_sec: t.duration_seconds,
    }
}

fn search_track_line(t: &Track) -> SearchTrackLine {
    SearchTrackLine {
        id: t.id,
        title: t.title.clone(),
        artist: t
            .artists
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>()
            .join(", "),
        album: t.album.name.clone(),
    }
}

/// Resolve filesystem paths to library tracks. When `ensure` is true, import missing files into DB.
fn resolve_paths_to_tracks(
    app: &AppHandle,
    paths: &[String],
    ensure: bool,
) -> Result<Vec<Track>, String> {
    let pool = pool(app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    let mut tracks = Vec::new();
    for p in paths {
        let path = Path::new(p);
        if !path.exists() {
            return Err(format!("path not found: {p}"));
        }
        match db::get_track_id_by_path(&conn, p) {
            Ok(id) => {
                tracks.push(db::get_track_by_id(&conn, id).map_err(|e| e.to_string())?);
            }
            Err(_) if ensure => {
                let id = scanner::ensure_track_in_db(&conn, path, &app_dir)
                    .map_err(|e| e.to_string())?;
                tracks.push(db::get_track_by_id(&conn, id).map_err(|e| e.to_string())?);
            }
            Err(_) => {
                return Err(format!(
                    "track not in library: {p}\nImport it first with: amus import <folder>"
                ));
            }
        }
    }
    Ok(tracks)
}

fn resolve_search_to_tracks(
    app: &AppHandle,
    query: &str,
    kind: SearchKind,
) -> Result<Vec<Track>, String> {
    let (tracks, _, _) = resolve_search_to_context(app, query, kind)?;
    Ok(tracks)
}

fn resolve_search_to_context(
    app: &AppHandle,
    query: &str,
    kind: SearchKind,
) -> Result<(Vec<Track>, PlaybackSource, String), String> {
    let pool = pool(app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    match kind {
        SearchKind::Artist => {
            let artists = db::search_artists(&conn, query, 1).map_err(|e| e.to_string())?;
            let a = artists
                .into_iter()
                .next()
                .ok_or_else(|| format!("no artist matching \"{query}\""))?;
            let tracks =
                db::get_tracks_by_artist(&conn, a.id).map_err(|e| e.to_string())?;
            Ok((
                tracks,
                PlaybackSource::Artist(a.id),
                a.name,
            ))
        }
        SearchKind::Album => {
            let albums = db::search_albums(&conn, query, 1).map_err(|e| e.to_string())?;
            let al = albums
                .into_iter()
                .next()
                .ok_or_else(|| format!("no album matching \"{query}\""))?;
            let tracks = db::get_tracks_by_album(&conn, al.id).map_err(|e| e.to_string())?;
            Ok((tracks, PlaybackSource::Album(al.id), al.name))
        }
        SearchKind::Track | SearchKind::All => {
            let tracks = db::search_tracks(&conn, query, 1).map_err(|e| e.to_string())?;
            let t = tracks
                .into_iter()
                .next()
                .ok_or_else(|| format!("no track matching \"{query}\""))?;
            let label = t.title.clone();
            Ok((vec![t], PlaybackSource::Direct, label))
        }
    }
}

fn search(app: &AppHandle, query: &str, kind: SearchKind) -> Result<CliData, String> {
    let pool = pool(app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    let limit = 20usize;
    let (tracks, artists, albums) = match kind {
        SearchKind::Track => (
            db::search_tracks(&conn, query, limit).map_err(|e| e.to_string())?,
            vec![],
            vec![],
        ),
        SearchKind::Artist => (
            vec![],
            db::search_artists(&conn, query, limit).map_err(|e| e.to_string())?,
            vec![],
        ),
        SearchKind::Album => (
            vec![],
            vec![],
            db::search_albums(&conn, query, limit).map_err(|e| e.to_string())?,
        ),
        SearchKind::All => (
            db::search_tracks(&conn, query, limit).map_err(|e| e.to_string())?,
            db::search_artists(&conn, query, limit).map_err(|e| e.to_string())?,
            db::search_albums(&conn, query, limit).map_err(|e| e.to_string())?,
        ),
    };
    Ok(CliData::SearchResults {
        tracks: tracks.iter().map(search_track_line).collect(),
        artists: artists
            .into_iter()
            .map(|a| SearchNamedLine {
                id: a.id,
                name: a.name,
            })
            .collect(),
        albums: albums
            .into_iter()
            .map(|a| SearchNamedLine {
                id: a.id,
                name: a.name,
            })
            .collect(),
    })
}

fn resolve_playlist(
    conn: &rusqlite::Connection,
    id_or_name: &str,
) -> Result<crate::models::Playlist, String> {
    if let Ok(id) = id_or_name.parse::<i64>() {
        if let Ok(p) = db::get_playlist(conn, id) {
            return Ok(p);
        }
    }
    db::get_playlist_by_name(conn, id_or_name).map_err(|_| {
        format!("playlist not found: {id_or_name}")
    })
}

fn playlist_add(app: &AppHandle, playlist: &str, path: &str) -> Result<CliData, String> {
    let pool = pool(app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    let pl = resolve_playlist(&conn, playlist)?;

    // Library-only: do not auto-import.
    let track_id = db::get_track_id_by_path(&conn, path).map_err(|_| {
        format!(
            "Track not in library: {path}\n\
Add it to your library first, e.g.:\n  amus import <folder-containing-the-track>\n\
Then rescan if needed: amus library rescan"
        )
    })?;

    db::add_track_to_playlist(&conn, pl.id, track_id).map_err(|e| e.to_string())?;
    let _ = app.emit("library-updated", ());
    let track = db::get_track_by_id(&conn, track_id).map_err(|e| e.to_string())?;
    Ok(msg(format!(
        "Added \"{}\" to playlist \"{}\"",
        track.title, pl.name
    )))
}

fn playlist_remove(app: &AppHandle, playlist: &str, path: &str) -> Result<CliData, String> {
    let pool = pool(app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    let pl = resolve_playlist(&conn, playlist)?;
    let track_id = db::get_track_id_by_path(&conn, path)
        .map_err(|_| format!("track not in library: {path}"))?;
    db::remove_track_from_playlist(&conn, pl.id, track_id).map_err(|e| e.to_string())?;
    let _ = app.emit("library-updated", ());
    Ok(msg(format!("Removed track from playlist \"{}\"", pl.name)))
}

fn playlist_play(app: &AppHandle, playlist: &str) -> Result<CliData, String> {
    let pool = pool(app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    let pl = resolve_playlist(&conn, playlist)?;
    let tracks =
        db::get_tracks_in_playlist(&conn, pl.id, None).map_err(|e| e.to_string())?;
    if tracks.is_empty() {
        return Err(format!("playlist \"{}\" is empty", pl.name));
    }
    let name = pl.name.clone();
    send_player(
        app,
        PlayerCommand::LoadContext {
            tracks,
            source: PlaybackSource::Playlist(pl.id),
            start_index: 0,
            context_label: Some(name.clone()),
        },
    )?;
    Ok(msg(format!("Playing playlist \"{name}\"")))
}

fn show_playlist(app: &AppHandle, name: &str) -> Result<CliData, String> {
    let pool = pool(app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    let pl = resolve_playlist(&conn, name)?;
    let tracks =
        db::get_tracks_in_playlist(&conn, pl.id, None).map_err(|e| e.to_string())?;
    Ok(CliData::TrackList {
        label: format!("Playlist: {} (id {})", pl.name, pl.id),
        tracks: tracks.iter().map(track_line).collect(),
    })
}

fn list_playlists(app: &AppHandle) -> Result<CliData, String> {
    let pool = pool(app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    let items = db::get_all_playlists(&conn).map_err(|e| e.to_string())?;
    Ok(CliData::NamedList {
        items: items
            .into_iter()
            .map(|p| SearchNamedLine {
                id: p.id,
                name: p.name,
            })
            .collect(),
    })
}

fn list_albums(app: &AppHandle) -> Result<CliData, String> {
    let pool = pool(app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    let items = db::get_all_albums(&conn).map_err(|e| e.to_string())?;
    Ok(CliData::NamedList {
        items: items
            .into_iter()
            .map(|a| SearchNamedLine {
                id: a.id,
                name: a.name,
            })
            .collect(),
    })
}

fn list_artists(app: &AppHandle) -> Result<CliData, String> {
    let pool = pool(app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    let items = db::get_all_artists(&conn).map_err(|e| e.to_string())?;
    Ok(CliData::NamedList {
        items: items
            .into_iter()
            .map(|a| SearchNamedLine {
                id: a.id,
                name: a.name,
            })
            .collect(),
    })
}

fn show_album(app: &AppHandle, id_or_name: &str) -> Result<CliData, String> {
    let pool = pool(app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    let album = if let Ok(id) = id_or_name.parse::<i64>() {
        db::get_album(&conn, id).map_err(|e| e.to_string())?
    } else {
        db::get_album_by_name(&conn, id_or_name)
            .map_err(|_| format!("album not found: {id_or_name}"))?
    };
    let tracks = db::get_tracks_by_album(&conn, album.id).map_err(|e| e.to_string())?;
    Ok(CliData::TrackList {
        label: format!("Album: {} (id {})", album.name, album.id),
        tracks: tracks.iter().map(track_line).collect(),
    })
}

fn show_artist(app: &AppHandle, id_or_name: &str) -> Result<CliData, String> {
    let pool = pool(app)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    let artist = if let Ok(id) = id_or_name.parse::<i64>() {
        db::get_artist(&conn, id).map_err(|e| e.to_string())?
    } else {
        db::get_artist_by_name(&conn, id_or_name)
            .map_err(|_| format!("artist not found: {id_or_name}"))?
    };
    let tracks = db::get_tracks_by_artist(&conn, artist.id).map_err(|e| e.to_string())?;
    Ok(CliData::TrackList {
        label: format!("Artist: {} (id {})", artist.name, artist.id),
        tracks: tracks.iter().map(track_line).collect(),
    })
}

fn run_update(app: &AppHandle) -> Result<CliData, String> {
    // Updater requires async; spawn and wait.
    let handle = app.clone();
    let result = std::thread::spawn(move || {
        tauri::async_runtime::block_on(async move {
            use tauri_plugin_updater::UpdaterExt;
            let updater = handle
                .updater_builder()
                .build()
                .map_err(|e| e.to_string())?;
            match updater.check().await {
                Ok(Some(update)) => {
                    update
                        .download_and_install(|_, _| {}, || {})
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok(format!(
                        "Updated to {}. Restart AMUS to finish.",
                        update.version
                    ))
                }
                Ok(None) => Ok("Already up to date.".into()),
                Err(e) => Err(format!("update check failed: {e}")),
            }
        })
    })
    .join()
    .map_err(|_| "update thread panicked".to_string())??;

    Ok(CliData::UpdateResult { message: result })
}
