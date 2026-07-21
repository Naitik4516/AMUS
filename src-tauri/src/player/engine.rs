use rodio::{Decoder, DeviceSinkBuilder, Player};
use std::fs::File;
use std::sync::Mutex;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("failed to open output device: {0}")]
    Device(String),
    #[error("failed to open file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to decode: {0}")]
    Decode(#[from] rodio::decoder::DecoderError),
    #[error("seek failed: {0}")]
    Seek(String),
    #[error("no track loaded")]
    NoTrack,
}

pub struct AudioEngine {
    handle: rodio::MixerDeviceSink,
    player: Mutex<Option<Player>>,
}

impl AudioEngine {
    pub fn new() -> Result<Self, EngineError> {
        let handle = DeviceSinkBuilder::open_default_sink()
            .map_err(|e| EngineError::Device(e.to_string()))?;
        Ok(Self {
            handle,
            player: Mutex::new(None),
        })
    }

    pub fn load(&self, path: &str) -> Result<(), EngineError> {
        let file = File::open(path)?;
        let source = Decoder::try_from(file)?;
        let player = Player::connect_new(&self.handle.mixer());
        player.append(source);
        player.pause();
        *self.player.lock().unwrap() = Some(player);
        Ok(())
    }

    pub fn play(&self) {
        if let Some(p) = self.player.lock().unwrap().as_ref() {
            p.play();
        }
    }

    pub fn pause(&self) {
        if let Some(p) = self.player.lock().unwrap().as_ref() {
            p.pause();
        }
    }

    pub fn is_paused(&self) -> bool {
        self.player
            .lock()
            .unwrap()
            .as_ref()
            .map(|p| p.is_paused())
            .unwrap_or(true)
    }

    pub fn position(&self) -> Duration {
        self.player
            .lock()
            .unwrap()
            .as_ref()
            .map(|p| p.get_pos())
            .unwrap_or_default()
    }

    pub fn seek(&self, pos: Duration) -> Result<(), EngineError> {
        let guard = self.player.lock().unwrap();
        let player = guard.as_ref().ok_or(EngineError::NoTrack)?;
        player
            .try_seek(pos)
            .map_err(|e| EngineError::Seek(e.to_string()))
    }

    pub fn set_volume(&self, v: f32) {
        if let Some(p) = self.player.lock().unwrap().as_ref() {
            p.set_volume(v.clamp(0.0, 1.0));
        }
    }

    pub fn is_finished(&self) -> bool {
        self.player
            .lock()
            .unwrap()
            .as_ref()
            .map(|p| p.empty())
            .unwrap_or(true)
    }

    pub fn tick_status(&self) -> (f64, bool) {
        let guard = self.player.lock().unwrap();
        match guard.as_ref() {
            Some(p) => (p.get_pos().as_secs_f64(), p.empty()),
            None => (0.0, true),
        }
    }

    pub fn state(&self) -> (f64, bool) {
        let guard = self.player.lock().unwrap();
        match guard.as_ref() {
            Some(p) => (p.get_pos().as_secs_f64(), p.is_paused()),
            None => (0.0, true),
        }
    }

    pub fn stop(&self) {
        *self.player.lock().unwrap() = None;
    }
}
