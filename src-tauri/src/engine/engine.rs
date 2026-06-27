use std::{collections::VecDeque, fs::File, io::BufReader, time::Duration};
use rodio::{decoder::Decoder, mixer::Mixer, Player};
use crate::db::{self, DbPool};
use crate::models::{RepeatMode, SourceType, TrackDetails};
use tauri::{AppHandle, Emitter};
use std::sync::Arc;
use parking_lot::Mutex;

fn is_track_near_end(pos_ms: u64, duration_sec: u32) -> bool {
    let near_end_threshold = std::cmp::min(5, (duration_sec as f64 * 0.05).ceil() as u32);
    pos_ms > 0 && duration_sec > 0 && (pos_ms / 1000) >= duration_sec.saturating_sub(near_end_threshold) as u64
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
    pub fn new(mixer: &Mixer) -> Self {
        Self {
            player: Player::connect_new(mixer),
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
        let file = BufReader::with_capacity(256 * 1024, File::open(path)?);
        let decoder = Decoder::try_from(file)?;

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
        self.user_queue.clear();
        self.play_next.clear();
        self.history.clear();
        self.tracks_played_this_gen = 0;

        if play_next_tracks.is_empty() && source_type == SourceType::Other {
            self.tracks_played_this_gen = 0;
        } else if !play_next_tracks.is_empty() {
            self.play_next = VecDeque::from(play_next_tracks.to_vec());
            self.tracks_played_this_gen = u32::MAX;
        }

        self.play_inner(track)
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

    pub fn reorder_queue(&mut self, from: usize, to: usize) {
        if from < self.user_queue.len() && to < self.user_queue.len() && from != to {
            let track = self.user_queue.remove(from).unwrap();
            self.user_queue.insert(to, track);
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

    pub fn set_shuffle(&mut self, enabled: bool) {
        self.shuffle = enabled;
    }

    pub fn set_repeat(&mut self, mode: RepeatMode) {
        self.repeat = mode;
    }

    pub fn shuffle(&self) -> bool {
        self.shuffle
    }

    pub fn repeat(&self) -> RepeatMode {
        self.repeat
    }

    pub fn has_finished(&self) -> bool {
        self.player.empty()
    }

    fn push_history(&mut self) {
        if let Some(current) = self.current_track.take() {
            self.history.push(current);
        }
    }

    pub fn play_next(&mut self, conn: Option<&rusqlite::Connection>) -> anyhow::Result<bool> {
        if self.repeat == RepeatMode::Track {
            if let Some(current) = &self.current_track.clone() {
                self.play_inner(current)?;
                return Ok(true);
            }
        }

        self.push_history();

        if self.shuffle {
            if !self.user_queue.is_empty() {
                let idx = rand::random::<u32>() as usize % self.user_queue.len();
                let next = self.user_queue.remove(idx).unwrap();
                return self.play_inner(&next).map(|_| true);
            }
            if !self.play_next.is_empty() {
                let idx = rand::random::<u32>() as usize % self.play_next.len();
                let next = self.play_next.remove(idx).unwrap();
                self.tracks_played_this_gen = self.tracks_played_this_gen.saturating_add(1);
                if let Some(conn) = conn {
                    self.maybe_regenerate(conn);
                }
                return self.play_inner(&next).map(|_| true);
            }
        } else {
            if let Some(next) = self.user_queue.pop_front() {
                return self.play_inner(&next).map(|_| true);
            }
            if let Some(next) = self.play_next.pop_front() {
                self.tracks_played_this_gen = self.tracks_played_this_gen.saturating_add(1);
                if let Some(conn) = conn {
                    self.maybe_regenerate(conn);
                }
                return self.play_inner(&next).map(|_| true);
            }
        }

        if self.repeat == RepeatMode::All {
            if let Some(first) = self.history.first() {
                let track = first.clone();
                self.history.clear();
                self.history.push(track.clone());
                return self.play_inner(&track).map(|_| true);
            }
        }

        // Ad-hoc playback: generate similar tracks when queues are empty
        if self.play_next.is_empty() && self.user_queue.is_empty() {
            if let Some(conn) = conn {
                let seed = self.history.last().map(|t| t.id);
                if let Some(seed_id) = seed {
                    self.regenerate_play_next(seed_id, conn);
                    if !self.play_next.is_empty() {
                        let next = if self.shuffle {
                            let idx = rand::random::<u32>() as usize % self.play_next.len();
                            self.play_next.remove(idx).unwrap()
                        } else {
                            self.play_next.pop_front().unwrap()
                        };
                        return self.play_inner(&next).map(|_| true);
                    }
                }
            }
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

    pub fn skip_current(&mut self, conn: Option<&rusqlite::Connection>) -> anyhow::Result<bool> {
        self.player.skip_one();
        self.play_next(conn)
    }

    fn maybe_regenerate(&mut self, conn: &rusqlite::Connection) {
        if self.tracks_played_this_gen >= 4 && self.play_next.len() < 5 {
            let seed = self
                .current_track
                .as_ref()
                .or_else(|| self.history.last())
                .map(|t| t.id);
            if let Some(seed_id) = seed {
                self.regenerate_play_next(seed_id, conn);
            }
        }
    }

    pub fn regenerate_play_next(&mut self, track_id: i64, conn: &rusqlite::Connection) {
        if let Ok(similar) = db::get_similar_tracks(conn, track_id, 10) {
            let details: Vec<TrackDetails> = similar
                .into_iter()
                .filter_map(|t| db::get_track_details(conn, t.id).ok())
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
                self.tracks_played_this_gen = 0;
            }
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
                    (eng.current_track.clone().unwrap(), eng.session_source_type())
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
                    let conn = pool.get().ok();
                    let _ = eng.play_next(conn.as_ref().map(|c| &**c));
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
