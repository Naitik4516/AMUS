use super::source::PlaybackSource;
use crate::models::Track;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct QueueViewPayload {
    pub context_source_type: String,
    pub context_label: Option<String>,
    pub upcoming_context: Vec<Track>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "payload")]
pub enum PlayerEvent {
    TrackChanged {
        track: Track,
        duration_sec: u32,
        source: PlaybackSource,
    },
    StateChanged {
        is_playing: bool,
    },
    Position {
        pos_sec: f64,
        at_epoch_ms: i64,
        is_playing: bool,
    },
    QueueChanged {
        user_queue: Vec<Track>,
        context_len: usize,
        context_position: Option<usize>,
        queue_view: QueueViewPayload,
    },
    RepeatShuffleChanged {
        repeat: String,
        shuffle: bool,
    },
    VolumeChanged {
        volume: f32,
    },
    PlaybackEnded,
    Error {
        message: String,
        track_id: Option<i64>,
    },
}

pub const PLAYER_EVENT_NAME: &str = "player://event";

pub fn emit(app: &tauri::AppHandle, event: PlayerEvent) {
    use tauri::Emitter;
    let event_name = event.event_name();
    if let Err(e) = app.emit(PLAYER_EVENT_NAME, &event) {
        tracing::warn!(error = %e, event = %event_name, "failed to emit player event");
    }
}

impl PlayerEvent {
    fn event_name(&self) -> &'static str {
        match self {
            PlayerEvent::TrackChanged { .. } => "TrackChanged",
            PlayerEvent::StateChanged { .. } => "StateChanged",
            PlayerEvent::Position { .. } => "Position",
            PlayerEvent::QueueChanged { .. } => "QueueChanged",
            PlayerEvent::RepeatShuffleChanged { .. } => "RepeatShuffleChanged",
            PlayerEvent::VolumeChanged { .. } => "VolumeChanged",
            PlayerEvent::PlaybackEnded => "PlaybackEnded",
            PlayerEvent::Error { .. } => "Error",
        }
    }
}
