pub mod engine;
pub use engine::{AudioEngine, PlaybackState};

use parking_lot::Mutex;

pub struct Player {
    pub playback_state: Mutex<PlaybackState>,
    pub engine: Mutex<AudioEngine>,
}
