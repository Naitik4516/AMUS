use std::{collections::VecDeque, fs::File, io::BufReader, time::Duration};
use rodio::{decoder::Decoder, mixer::Mixer, Player};
use crate::models::TrackDetails;
use tauri::{AppHandle, Emitter};
use std::sync::Arc;
use parking_lot::Mutex;

#[derive(Debug, Clone, serde::Serialize)]
pub struct PlaybackState {
    pub is_playing: bool,
    pub position_ms: u64,
    pub volume: f32,
    pub queue_len: usize,
    pub current_track: Option<TrackDetails>,
}

pub struct AudioEngine {
    player: Player,
    queue: VecDeque<TrackDetails>,
    history: Vec<TrackDetails>,
    current_track: Option<TrackDetails>,
    volume: f32,
}

impl AudioEngine {
    pub fn new(mixer: &Mixer) -> Self {
        Self {
            player: Player::connect_new(mixer),
            queue: VecDeque::new(),
            history: Vec::new(),
            current_track: None,
            volume: 1.0,
        }
    }

    pub fn play(&mut self, track: &TrackDetails) -> anyhow::Result<()> {
        let path = track.path.as_str();
        let file = BufReader::new(File::open(path)?);
        let decoder = Decoder::try_from(file)?;

        self.player.clear();
        self.player.append(decoder);
        self.current_track = Some(track.clone());
        self.player.play();

        Ok(())
    }

    pub fn enqueue(&mut self, track: &TrackDetails) {
        self.queue.push_back(track.clone());
    }

    pub fn pause(&self) {
        self.player.pause();
    }

    pub fn resume(&self) {
        self.player.play();
    }

    pub fn stop(&self) {
        self.player.stop();
    }

    pub fn is_paused(&self) -> bool {
        self.player.is_paused()
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        self.player.set_volume(volume);
    }

    pub fn volume(&self) -> f32 {
        self.player.volume()
    }

    pub fn seek(&self, position: u64) -> std::result::Result<(), rodio::source::SeekError> {
        self.player.try_seek(Duration::from_secs(position))
    }

    pub fn position_ms(&self) -> u64 {
        self.player.get_pos().as_millis() as u64
    }

    pub fn play_next(&mut self) -> anyhow::Result<()> {
        let Some(next) = self.queue.pop_front() else {
            return Ok(());
        };

        if let Some(current) = self.current_track.take() {
            self.history.push(current);
        }

        self.play(&next)
    }

    pub fn play_previous(&mut self) -> anyhow::Result<()> {
        let Some(previous) = self.history.pop() else {
            return Ok(());
        };

        if let Some(current) = self.current_track.take() {
            self.queue.push_front(current);
        }

        self.play(&previous)
    }

    pub fn skip_current(&mut self) -> anyhow::Result<()> {
        self.player.skip_one();
        self.play_next()
    }

    pub fn clear_queue(&mut self) {
        self.queue.clear();
    }

    pub fn has_finished(&self) -> bool {
        self.player.empty()
    }

    pub fn get_playback_state(&self) -> PlaybackState {
        PlaybackState {
            is_playing: !self.player.is_paused(),
            position_ms: self.player.get_pos().as_millis() as u64,
            volume: self.player.volume(),
            queue_len: self.queue.len(),
            current_track: self.current_track.clone(),
        }
    }
}

pub fn spawn_playback_monitor(app_handle: AppHandle, engine: Arc<Mutex<AudioEngine>>) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(500));
            
            let state = {
                let mut engine_lock = engine.lock();
                
                // Handle auto-play next if finished
                if engine_lock.has_finished() && !engine_lock.is_paused() {
                    let _ = engine_lock.play_next();
                }
                
                engine_lock.get_playback_state()
            };

            let _ = app_handle.emit("playback-state", &state);
        }
    });
}
