use crate::db::{self, DbPool};
use crate::error::Error;
use crate::models::{RepeatMode, SourceType, TrackDetails};
use parking_lot::Mutex;
use rodio::{Player, decoder::Decoder, mixer::Mixer};
use std::sync::Arc;
use std::{collections::VecDeque, fs::File, io::BufReader, time::Duration};
use tauri::{AppHandle, Emitter};

fn is_track_near_end(pos_ms: u64, duration_sec: u32) -> bool {
    let near_end_threshold = std::cmp::min(5, (duration_sec as f64 * 0.05).ceil() as u32);
    pos_ms > 0
        && duration_sec > 0
        && (pos_ms / 1000) >= duration_sec.saturating_sub(near_end_threshold) as u64
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PlaybackState {
    pub is_playing: bool,
    pub position_ms: u64,
    pub volume: f32,
    pub current_track: Option<TrackDetails>,
    pub user_queue: Vec<TrackDetails>,
    pub play_next: Vec<TrackDetails>,
    pub shuffle: bool,
    pub repeat: u8,
}

pub struct AudioEngine {
    player: Player,
    pool: DbPool,
    user_queue: VecDeque<TrackDetails>,
    play_next: VecDeque<TrackDetails>,
    history: Vec<TrackDetails>,
    current_track: Option<TrackDetails>,
    shuffle: bool,
    repeat: RepeatMode,
    volume: f32,
    tracks_played_this_gen: u32,
    session_source_type: SourceType,
}

impl AudioEngine {
    pub fn new(mixer: &Mixer, pool: DbPool) -> Self {
        Self {
            player: Player::connect_new(mixer),
            pool: pool,
            user_queue: VecDeque::new(),
            play_next: VecDeque::new(),
            history: Vec::new(),
            current_track: None,
            shuffle: false,
            repeat: RepeatMode::Off,
            volume: 1.0,
            tracks_played_this_gen: 0,
            session_source_type: SourceType::Other,
        }
    }

    fn play_inner(&mut self, track: &TrackDetails) -> anyhow::Result<()> {
        let path = track.path.as_str();
        let file = File::open(path)?;
        let len = file.metadata()?.len();
        let decoder = Decoder::builder()
            .with_data(BufReader::with_capacity(256 * 1024, file))
            .with_byte_len(len)
            .with_seekable(true)
            .build()?;

        self.player.clear();
        self.player.append(decoder);
        self.current_track = Some(track.clone());
        self.player.play();

        Ok(())
    }

    pub fn play(
        &mut self,
        track: &TrackDetails,
        play_next_tracks: &[TrackDetails],
        source_type: SourceType,
        _source_id: Option<i64>,
    ) -> anyhow::Result<()> {
        self.session_source_type = source_type;
        self.play_next.clear();
        self.play_inner(track);

        match source_type {
            SourceType::Other => {
                self.generate_play_next(track.id);
            }
            _ => {
                self.play_next = VecDeque::from(play_next_tracks.to_vec());
            }
        }

        Ok(())
    }

    pub fn play_next(&mut self) -> anyhow::Result<bool> {
        match self.repeat {
            RepeatMode::Track => {
                if let Some(current) = &self.current_track.clone() {
                    self.play_inner(current)?;
                    return Ok(true);
                }
            }
            RepeatMode::All => {
                let next = self.history.first().cloned();
                let history = self.history.clone();
                if let Some(ref track) = next {
                    self.play(
                        track,
                        &history,
                        self.session_source_type,
                        self.session_source_id,
                    )
                    .map(|_| true)?;
                    self.history.clear();
                    return Ok(true);
                }
            }
            RepeatMode::Off => {}
        };

        self.push_history();

        if !self.user_queue.is_empty() {
            let next: TrackDetails;
            if self.shuffle {
                let idx = rand::random::<u32>() as usize % self.user_queue.len();
                next = self.user_queue.remove(idx).unwrap();
            } else {
                next = self.user_queue.pop_front().unwrap();
            }
            return self.play_inner(&next).map(|_| true);
        }

        if !self.play_next.is_empty() {
            let next: TrackDetails;
            if self.shuffle {
                let idx = rand::random::<u32>() as usize % self.play_next.len();
                next = self.play_next.remove(idx).unwrap();
            } else {
                next = self.play_next.pop_front().unwrap();
            }
            if self.play_next.len() <= 2 {
                self.generate_play_next(self.history.last().map(|t| t.id).unwrap_or(0))
            }
            return self.play_inner(&next).map(|_| true);
        }

        self.player.stop();

        Ok(false)
    }

    pub fn play_previous(&mut self) -> anyhow::Result<bool> {
        let Some(previous) = self.history.pop() else {
            return Ok(false);
        };

        if let Some(current) = self.current_track.take() {
            self.user_queue.push_front(current);
        }

        self.play_inner(&previous)?;
        Ok(true)
    }

    pub fn generate_play_next(&mut self, track_id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get().map_err(Error::Pool)?;
        if let Ok(similar) = db::get_similar_tracks(&conn, track_id, 10) {
            let details: Vec<TrackDetails> = similar
                .into_iter()
                .filter_map(|t| db::get_track_details(&conn, t.id).ok())
                .collect();

            let current_id = self.current_track.as_ref().map(|t| t.id);
            let filtered: Vec<TrackDetails> = details
                .into_iter()
                .filter(|t| {
                    t.id != track_id
                        && current_id.map_or(true, |id| t.id != id)
                        && !self.user_queue.iter().any(|qt| qt.id == t.id)
                        && !self.play_next.iter().any(|pt| pt.id == t.id)
                })
                .collect();

            if !filtered.is_empty() {
                self.play_next = VecDeque::from(filtered);
            }
        }
        Ok(())
    }

    pub fn add_to_queue(&mut self, track: &TrackDetails) {
        if !self.user_queue.iter().any(|t| t.id == track.id) {
            self.user_queue.push_back(track.clone());
        }
    }

    pub fn play_next_in_queue(&mut self, track: &TrackDetails) {
        if !self.user_queue.iter().any(|t| t.id == track.id) {
            self.user_queue.push_front(track.clone());
        }
    }

    pub fn remove_from_queue(&mut self, index: usize) -> Option<TrackDetails> {
        if index < self.user_queue.len() {
            self.user_queue.remove(index)
        } else {
            None
        }
    }

    pub fn clear_queue(&mut self) {
        self.user_queue.clear();
    }

    pub fn pause(&self) {
        self.player.pause();
    }

    pub fn resume(&self) {
        self.player.play();
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

    pub fn set_shuffle(&mut self, enabled: bool) {
        self.shuffle = enabled;
    }

    pub fn set_repeat(&mut self, mode: RepeatMode) {
        self.repeat = mode;
    }

    pub fn has_finished(&self) -> bool {
        self.player.empty()
    }

    fn push_history(&mut self) {
        if let Some(current) = self.current_track.take() {
            self.history.push(current);
        }
    }


    pub fn get_user_queue(&self) -> Vec<TrackDetails> {
        self.user_queue.iter().cloned().collect()
    }

    pub fn get_play_next(&self) -> Vec<TrackDetails> {
        self.play_next.iter().cloned().collect()
    }

    pub fn current_track(&self) -> Option<&TrackDetails> {
        self.current_track.as_ref()
    }

    pub fn session_source_type(&self) -> SourceType {
        self.session_source_type
    }

    pub fn get_playback_state(&self) -> PlaybackState {
        PlaybackState {
            is_playing: !self.player.is_paused() && !self.player.empty(),
            position_ms: self.player.get_pos().as_millis() as u64,
            volume: self.player.volume(),
            current_track: self.current_track.clone(),
            user_queue: self.get_user_queue(),
            play_next: self.get_play_next(),
            shuffle: self.shuffle,
            repeat: self.repeat as u8,
        }
    }
}

pub fn spawn_playback_monitor(
    app_handle: AppHandle,
    engine: Arc<Mutex<AudioEngine>>,
    pool: DbPool,
) {
    std::thread::spawn(move || {
        let mut last_track_id: Option<i64> = None;
        loop {
            std::thread::sleep(Duration::from_millis(250));

            // Phase 1: Check for completion under the lock, extract info, then drop lock
            let (should_advance, rec_info): (bool, Option<(TrackDetails, SourceType)>) = {
                let eng = engine.lock();

                let finished = eng.has_finished() && !eng.is_paused();
                let near_end = eng.current_track.as_ref().map_or(false, |track| {
                    is_track_near_end(eng.position_ms(), track.duration_seconds)
                });
                let should = finished && near_end;
                let info = should.then(|| {
                    (
                        eng.current_track.clone().unwrap(),
                        eng.session_source_type(),
                    )
                });
                (should, info)
            };

            // Phase 2: Record playback outside the lock (fire-and-forget)
            if let Some((track, src)) = rec_info {
                let pool = pool.clone();
                std::thread::spawn(move || {
                    if let Ok(conn) = pool.get() {
                        let _ = db::record_playback(&conn, track.id, src.to_db_string(), 100.0);
                    }
                });
            }

            // Phase 3: Advance to next track under the lock (if still applicable)
            let (state, track_changed) = {
                let mut eng = engine.lock();

                if should_advance && eng.has_finished() && !eng.is_paused() {
                    let _ = eng.play_next();
                }

                let new_id = eng.current_track.as_ref().map(|t| t.id);
                let changed = new_id != last_track_id;
                last_track_id = new_id;
                (eng.get_playback_state(), changed)
            };

            let _ = app_handle.emit("playback-state", &state);
            if track_changed {
                let _ = app_handle.emit("track-changed", &state);
            }
        }
    });
}
