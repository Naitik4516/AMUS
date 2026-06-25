pub mod engine;
pub use engine::{AudioEngine, PlaybackState, spawn_playback_monitor};

use parking_lot::Mutex;
use std::sync::Arc;

pub struct Player {
    pub engine: Arc<Mutex<AudioEngine>>,
}
