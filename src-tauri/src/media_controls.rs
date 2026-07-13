//! OS media controls integration via souvlaki (MPRIS / SMTC / macOS Now Playing).

use std::sync::{LazyLock, Mutex};
use std::time::Duration;

use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig};
use tauri::{AppHandle, Listener, Manager};

use crate::commands::PlayerHandle;
use crate::player::actor::PlayerCommand;

static INSTANCE: LazyLock<Mutex<Option<MediaControlsManager>>> =
    LazyLock::new(|| Mutex::new(None));

struct MediaControlsManager {
    controls: MediaControls,
    _listener_guard: tauri::EventId,
}

pub fn init(app: AppHandle) -> Result<(), String> {
    let mut guard = INSTANCE.lock().map_err(|e| e.to_string())?;
    if guard.is_some() {
        return Ok(()); // already running
    }

    let config = PlatformConfig {
        dbus_name: "amus",
        display_name: "AMUS",
        hwnd: window_hwnd(&app),
    };

    let mut controls = MediaControls::new(config).map_err(|e| format!("media controls: {e}"))?;

    let app_for_events = app.clone();
    controls
        .attach(move |event| {
            if let Some(handle) = app_for_events.try_state::<PlayerHandle>() {
                let cmd = match event {
                    MediaControlEvent::Play => Some(PlayerCommand::Play),
                    MediaControlEvent::Pause => Some(PlayerCommand::Pause),
                    MediaControlEvent::Toggle => Some(PlayerCommand::PlayPause),
                    MediaControlEvent::Next => Some(PlayerCommand::Next),
                    MediaControlEvent::Previous => Some(PlayerCommand::Previous),
                    MediaControlEvent::Stop => Some(PlayerCommand::Stop),
                    MediaControlEvent::SeekBy(dir, delta) => {
                        let sign = match dir {
                            souvlaki::SeekDirection::Forward => 1.0,
                            souvlaki::SeekDirection::Backward => -1.0,
                        };
                        Some(PlayerCommand::SeekRelative(sign * delta.as_secs_f64()))
                    }
                    MediaControlEvent::SetPosition(pos) => {
                        Some(PlayerCommand::Seek(pos.0.as_secs_f64()))
                    }
                    MediaControlEvent::SetVolume(vol) => {
                        Some(PlayerCommand::SetVolume(vol.max(0.0).min(1.0) as f32))
                    }
                    MediaControlEvent::Raise => {
                        if let Some(w) = app_for_events.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                        None
                    }
                    MediaControlEvent::Quit => {
                        app_for_events.exit(0);
                        None
                    }
                    _ => None,
                };
                if let Some(cmd) = cmd {
                    let _ = handle.0.send(cmd);
                }
            }
        })
        .map_err(|e| format!("media controls attach: {e}"))?;

    let app_for_listener = app.clone();
    let listener_guard = app.listen("player://event", move |event| {
        let payload: serde_json::Value = match serde_json::from_str(event.payload()) {
            Ok(v) => v,
            Err(_) => return,
        };
        update_controls(&payload, &app_for_listener);
    });

    *guard = Some(MediaControlsManager {
        controls,
        _listener_guard: listener_guard,
    });

    Ok(())
}

pub fn detach() -> Result<(), String> {
    let mut guard = INSTANCE.lock().map_err(|e| e.to_string())?;
    if let Some(mgr) = guard.take() {
        drop(mgr); // drops listener guard and detaches controls
    }
    Ok(())
}

pub fn is_active() -> bool {
    INSTANCE.lock().map(|g| g.is_some()).unwrap_or(false)
}

fn update_controls(payload: &serde_json::Value, _app: &AppHandle) {
    let event_type = payload.get("event").and_then(|v| v.as_str());
    let data = payload.get("payload");

    // We need mutable access to controls — re-acquire from INSTANCE
    let mut guard = match INSTANCE.lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    let Some(ref mut mgr) = *guard else { return };

    match event_type {
        Some("TrackChanged") => {
            if let Some(track) = data.and_then(|d| d.get("track")) {
                let title = track.get("title").and_then(|v| v.as_str());
                let artist = track
                    .get("artists")
                    .and_then(|a| a.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|a| a.get("name"))
                    .and_then(|v| v.as_str());
                let album = track.get("album").and_then(|a| a.get("name")).and_then(|v| v.as_str());
                let duration = data
                    .and_then(|d| d.get("duration_sec"))
                    .and_then(|v| v.as_u64());

                mgr.controls
                    .set_metadata(MediaMetadata {
                        title,
                        artist,
                        album,
                        duration: duration.map(Duration::from_secs),
                        ..Default::default()
                    })
                    .ok();
            }
        }
        Some("StateChanged") => {
            let is_playing = data
                .and_then(|d| d.get("is_playing"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if is_playing {
                mgr.controls
                    .set_playback(MediaPlayback::Playing {
                        progress: Some(MediaPosition(Duration::ZERO)),
                    })
                    .ok();
            } else {
                mgr.controls
                    .set_playback(MediaPlayback::Paused {
                        progress: Some(MediaPosition(Duration::ZERO)),
                    })
                    .ok();
            }
        }
        Some("Position") => {
            let pos_sec = data
                .and_then(|d| d.get("pos_sec"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let playback = if payload
                .get("is_playing")
                .and_then(|v| v.as_bool())
                .unwrap_or(true)
            {
                MediaPlayback::Playing {
                    progress: Some(MediaPosition(Duration::from_secs_f64(pos_sec))),
                }
            } else {
                MediaPlayback::Paused {
                    progress: Some(MediaPosition(Duration::from_secs_f64(pos_sec))),
                }
            };
            mgr.controls.set_playback(playback).ok();
        }
        Some("PlaybackEnded") => {
            mgr.controls
                .set_playback(MediaPlayback::Stopped)
                .ok();
        }
        _ => {}
    }
}

#[cfg(target_os = "windows")]
fn window_hwnd(app: &AppHandle) -> Option<*mut std::ffi::c_void> {
    use tauri::Manager;
    app.get_webview_window("main")
        .and_then(|w| w.hwnd().ok())
        .map(|raw| raw.0 as *mut std::ffi::c_void)
}

#[cfg(not(target_os = "windows"))]
fn window_hwnd(_app: &AppHandle) -> Option<*mut std::ffi::c_void> {
    None
}
