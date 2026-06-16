pub mod engine;
pub use engine::{AudioEngine, PlaybackState};

use parking_lot::Mutex;
use std::sync::Arc;

pub struct Player {
    pub engine: Arc<Mutex<AudioEngine>>,
}
