use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};
use tauri::AppHandle;
use tokio::sync::oneshot;

use super::engine::AudioEngine;
use super::events::{PlayerEvent, QueueViewPayload, emit};
use super::playback;
use super::queue::{NextOutcome, PlaybackQueue, PreviousOutcome, QueueItem};
use super::source::{PlaybackSource, RepeatMode};
use crate::db;
use crate::models::Track;

pub type DbPool = Pool<SqliteConnectionManager>;

pub enum PlayerCommand {
    LoadContext {
        tracks: Vec<Track>,
        source: PlaybackSource,
        start_index: usize,
        context_label: Option<String>,
    },
    PlayPause,
    Play,
    Pause,
    Next,
    Previous,
    Seek(f64),
    SeekRelative(f64),
    SetVolume(f32),
    AdjustVolume(f32),
    ToggleMute,
    SetRepeat(RepeatMode),
    ToggleShuffle,
    EnqueueNext(Track),
    EnqueueEnd(Track),
    EnqueueEndMany(Vec<Track>),
    RemoveFromQueue(i64),
    ClearQueue,
    ReorderQueue {
        queue_id: i64,
        new_index: usize,
    },
    Stop,
    SetAutoplay(bool),
    PlayTrackFromContext(i64),
    RestoreSession {
        context_tracks: Vec<Track>,
        source: PlaybackSource,
        start_index: usize,
        context_label: Option<String>,
        user_queue_tracks: Vec<Track>,
        position_sec: f64,
        volume: f32,
        repeat: RepeatMode,
        shuffle: bool,
    },
    GetState(oneshot::Sender<PlayerStateSnapshot>),
    Shutdown,
    Tick,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PlayerStateSnapshot {
    pub current_track: Option<Track>,
    pub is_playing: bool,
    pub position_sec: f64,
    pub duration_sec: u32,
    pub repeat: String,
    pub shuffle: bool,
    pub volume: f32,
    pub muted: bool,
    pub user_queue: Vec<Track>,
    pub queue_view: QueueViewPayload,
}

struct NowPlaying {
    track_id: i64,
    duration_sec: f64,
    max_position_reached: f64,
    source: PlaybackSource,
    started_at: Instant,
}

pub struct PlayerActor {
    rx: Receiver<PlayerCommand>,
    app: AppHandle,
    pool: DbPool,

    engine: AudioEngine,
    queue: PlaybackQueue,
    volume: f32,
    muted: bool,
    volume_before_mute: f32,
    autoplay_enabled: bool,
    now_playing: Option<NowPlaying>,
    has_track_loaded: bool,
    last_emitted_pos_at: Instant,
}

const TICK_INTERVAL: Duration = Duration::from_millis(250);
const POSITION_EMIT_INTERVAL: Duration = Duration::from_millis(1000);

impl PlayerActor {
    pub fn spawn(app: AppHandle, pool: DbPool) -> Sender<PlayerCommand> {
        let (tx, rx) = std::sync::mpsc::channel::<PlayerCommand>();

        std::thread::Builder::new()
            .name("player-actor".into())
            .spawn(move || {
                let engine = match AudioEngine::new() {
                    Ok(e) => e,
                    Err(e) => {
                        eprintln!("fatal: failed to init audio engine: {e}");
                        return;
                    }
                };
                let mut actor = PlayerActor {
                    rx,
                    app,
                    pool,
                    engine,
                    queue: PlaybackQueue::new(),
                    volume: 1.0,
                    muted: false,
                    volume_before_mute: 1.0,
                    autoplay_enabled: true,
                    now_playing: None,
                    has_track_loaded: false,
                    last_emitted_pos_at: Instant::now(),
                };
                actor.run();
            })
            .expect("failed to spawn player-actor thread");

        tx
    }

    fn conn(&self) -> Option<r2d2::PooledConnection<SqliteConnectionManager>> {
        match self.pool.get() {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("failed to get db connection: {e}");
                None
            }
        }
    }

    fn run(&mut self) {
        loop {
            let is_active = self.has_track_loaded && !self.engine.is_paused();
            let cmd = if is_active {
                match self.rx.recv_timeout(TICK_INTERVAL) {
                    Ok(c) => c,
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => PlayerCommand::Tick,
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                }
            } else {
                match self.rx.recv() {
                    Ok(c) => c,
                    Err(_) => break,
                }
            };

            match cmd {
                PlayerCommand::LoadContext {
                    tracks,
                    source,
                    start_index,
                    context_label,
                } => {
                    self.finalize_now_playing();
                    self.queue
                        .load_context(tracks, source, start_index, context_label);
                    self.load_current_into_engine(true);
                }
                PlayerCommand::PlayPause => self.toggle_play_pause(),
                PlayerCommand::Play => self.handle_play(),
                PlayerCommand::Pause => self.handle_pause(),
                PlayerCommand::Stop => self.stop_playback(),
                PlayerCommand::Next => self.handle_next(),
                PlayerCommand::Previous => self.handle_previous(),
                PlayerCommand::Seek(pos) => self.handle_seek(pos),
                PlayerCommand::SeekRelative(delta) => {
                    let (pos, _) = self.engine.state();
                    self.handle_seek((pos + delta).max(0.0));
                }
                PlayerCommand::SetVolume(v) => self.apply_volume(v),
                PlayerCommand::AdjustVolume(delta) => {
                    self.apply_volume(self.volume + delta);
                }
                PlayerCommand::ToggleMute => self.toggle_mute(),
                PlayerCommand::SetRepeat(mode) => {
                    self.queue.set_repeat(mode);
                    self.emit_repeat_shuffle();
                }
                PlayerCommand::ToggleShuffle => {
                    self.queue.set_shuffle(!self.queue.shuffle_enabled());
                    self.emit_repeat_shuffle();
                }
                PlayerCommand::EnqueueNext(track) => {
                    if let Some(conn) = self.conn() {
                        if let Ok(db_id) = playback::queue_insert_front(&conn, track.id) {
                            self.queue.enqueue_next(db_id, track);
                            self.emit_queue_changed();
                        }
                    }
                }
                PlayerCommand::EnqueueEnd(track) => {
                    if let Some(conn) = self.conn() {
                        if let Ok(db_id) = playback::queue_insert_back(&conn, track.id) {
                            self.queue.enqueue_end(db_id, track);
                            self.emit_queue_changed();
                        }
                    }
                }
                PlayerCommand::EnqueueEndMany(tracks) => {
                    if let Some(mut conn) = self.conn() {
                        if let Ok(db_ids) = playback::queue_insert_back_many(&mut conn, &tracks) {
                            for (db_id, track) in db_ids.into_iter().zip(tracks) {
                                self.queue.enqueue_end(db_id, track);
                            }
                            self.emit_queue_changed();
                        }
                    }
                }
                PlayerCommand::RemoveFromQueue(db_id) => {
                    self.queue.remove_from_queue(db_id);
                    if let Some(conn) = self.conn() {
                        let _ = playback::queue_remove(&conn, db_id);
                    }
                    self.emit_queue_changed();
                }
                PlayerCommand::ClearQueue => {
                    self.queue.clear_queue();
                    if let Some(conn) = self.conn() {
                        let _ = playback::queue_clear_all(&conn);
                    }
                    self.emit_queue_changed();
                }
                PlayerCommand::ReorderQueue {
                    queue_id,
                    new_index,
                } => {
                    self.queue.reorder_queue(queue_id, new_index);
                    if let Some(conn) = self.conn() {
                        let _ = playback::queue_reorder(&conn, queue_id, new_index);
                    }
                    self.emit_queue_changed();
                }
                PlayerCommand::SetAutoplay(v) => self.autoplay_enabled = v,
                PlayerCommand::PlayTrackFromContext(track_id) => {
                    if self.queue.jump_to_track(track_id) {
                        self.load_current_into_engine(true);
                    }
                }
                PlayerCommand::RestoreSession {
                    context_tracks,
                    source,
                    start_index,
                    context_label,
                    user_queue_tracks,
                    position_sec,
                    volume,
                    repeat,
                    shuffle,
                } => {
                    self.finalize_now_playing();
                    self.queue.clear_queue();
                    if let Some(conn) = self.conn() {
                        let _ = playback::queue_clear_all(&conn);
                    }
                    self.queue
                        .load_context(context_tracks, source, start_index, context_label);
                    // load current track with autoplay=false to prevent transient playback before seek
                    self.load_current_into_engine(false);
                    if !user_queue_tracks.is_empty() {
                        if let Some(mut conn) = self.conn() {
                            if let Ok(db_ids) = playback::queue_insert_back_many(&mut conn, &user_queue_tracks) {
                                for (db_id, track) in db_ids.into_iter().zip(user_queue_tracks) {
                                    self.queue.enqueue_end(db_id, track);
                                }
                            }
                        }
                    }
                    self.queue.set_repeat(repeat);
                    self.queue.set_shuffle(shuffle);
                    self.emit_repeat_shuffle();
                    self.volume = volume.clamp(0.0, 1.0);
                    self.engine.set_volume(self.volume);
                    emit(&self.app, PlayerEvent::VolumeChanged { volume: self.volume });
                    self.emit_queue_changed();
                    let _ = self.engine.seek(Duration::from_secs_f64(position_sec.max(0.0)));
                    if let Some(np) = &mut self.now_playing {
                        np.max_position_reached = np.max_position_reached.max(position_sec);
                    }
                    // Since session is restored, we trigger play if it was loaded
                    if self.has_track_loaded {
                        self.engine.play();
                        emit(&self.app, PlayerEvent::StateChanged { is_playing: true });
                    }
                    emit(&self.app, PlayerEvent::Position { pos_sec: position_sec, at_epoch_ms: now_epoch_ms() });
                }
                PlayerCommand::GetState(reply) => {
                    let _ = reply.send(self.snapshot());
                }
                PlayerCommand::Shutdown => {
                    self.finalize_now_playing();
                    break;
                }
                PlayerCommand::Tick => {
                    self.on_tick();
                }
            }
        }
    }


    fn load_current_into_engine(&mut self, autoplay: bool) {
        let Some((track, source)) = self.queue.current().cloned() else {
            self.has_track_loaded = false;
            emit(&self.app, PlayerEvent::PlaybackEnded);
            return;
        };

        let conn = match self.conn() {
            Some(c) => c,
            None => {
                emit(
                    &self.app,
                    PlayerEvent::Error {
                        message: "failed to connect to database".into(),
                        track_id: Some(track.id),
                    },
                );
                return;
            }
        };
        let path = match db::get_track_path_by_id(&conn, track.id) {
            Ok(p) => p,
            _ => {
                emit(
                    &self.app,
                    PlayerEvent::Error {
                        message: "track file path not found".into(),
                        track_id: Some(track.id),
                    },
                );
                drop(conn);
                self.handle_next(); // skip broken track
                return;
            }
        };
        drop(conn);

        match self.engine.load(&path) {
            Ok(()) => {
                self.has_track_loaded = true;
                self.engine.set_volume(self.volume);
                self.now_playing = Some(NowPlaying {
                    track_id: track.id,
                    duration_sec: track.duration_seconds as f64,
                    max_position_reached: 0.0,
                    source: source.clone(),
                    started_at: Instant::now(),
                });
                if autoplay {
                    self.engine.play();
                }
                emit(
                    &self.app,
                    PlayerEvent::TrackChanged {
                        track: track.clone(),
                        duration_sec: track.duration_seconds,
                        source,
                    },
                );
                emit(
                    &self.app,
                    PlayerEvent::StateChanged {
                        is_playing: autoplay,
                    },
                );
                self.emit_queue_changed();
            }
            Err(e) => {
                emit(
                    &self.app,
                    PlayerEvent::Error {
                        message: format!("failed to load track: {e}"),
                        track_id: Some(track.id),
                    },
                );
                self.handle_next();
            }
        }
    }

    fn toggle_play_pause(&mut self) {
        if !self.has_track_loaded {
            return;
        }
        if self.engine.is_paused() {
            self.handle_play();
        } else {
            self.handle_pause();
        }
    }

    fn handle_play(&mut self) {
        if !self.has_track_loaded {
            return;
        }
        self.engine.play();
        emit(&self.app, PlayerEvent::StateChanged { is_playing: true });
    }

    fn handle_pause(&mut self) {
        if !self.has_track_loaded {
            return;
        }
        self.engine.pause();
        emit(&self.app, PlayerEvent::StateChanged { is_playing: false });
    }

    fn apply_volume(&mut self, v: f32) {
        let clamped = v.clamp(0.0, 1.0);
        self.volume = clamped;
        if self.muted {
            // Unmute when volume is set explicitly while muted
            self.muted = false;
        }
        self.engine.set_volume(self.volume);
        emit(
            &self.app,
            PlayerEvent::VolumeChanged {
                volume: self.volume,
            },
        );
    }

    fn toggle_mute(&mut self) {
        if self.muted {
            self.muted = false;
            self.volume = self.volume_before_mute;
            self.engine.set_volume(self.volume);
        } else {
            self.volume_before_mute = if self.volume > 0.0 {
                self.volume
            } else {
                self.volume_before_mute
            };
            self.muted = true;
            self.engine.set_volume(0.0);
        }
        emit(
            &self.app,
            PlayerEvent::VolumeChanged {
                volume: if self.muted { 0.0 } else { self.volume },
            },
        );
    }

    fn handle_next(&mut self) {
        self.finalize_now_playing();
        match self.queue.advance_next() {
            NextOutcome::Track(_, _) => self.load_current_into_engine(true),
            NextOutcome::NeedsAutoplay => self.try_autoplay(),
            NextOutcome::End => {
                self.has_track_loaded = false;
                self.engine.stop();
                emit(&self.app, PlayerEvent::StateChanged { is_playing: false }); 
                emit(&self.app, PlayerEvent::PlaybackEnded);
            }
        }
    }

    fn try_autoplay(&mut self) {
        if !self.autoplay_enabled {
            self.stop_playback();
            return;
        }
        let Some(last_id) = self.queue.last_played_id() else {
            self.stop_playback();
            return;
        };
        let conn = match self.conn() {
            Some(c) => c,
            None => {
                self.stop_playback();
                return;
            }
        };
        match db::get_similar_tracks(&conn, last_id, 20) {
            Ok(recs) if !recs.is_empty() => {
                self.queue.extend_with_autoplay(recs);
                self.load_current_into_engine(true);
            }
            Ok(_) => {
                eprintln!("autoplay: get_similar_tracks returned 0 recommendations for track {last_id}");
                self.stop_playback();
            }
            Err(e) => {
                eprintln!("autoplay: get_similar_tracks failed for track {last_id}: {e}");
                self.stop_playback();
            }
        }
    }
    
    fn stop_playback(&mut self) {
        self.has_track_loaded = false;
        self.engine.stop();
        emit(&self.app, PlayerEvent::StateChanged { is_playing: false });
        emit(&self.app, PlayerEvent::PlaybackEnded);
    }

    fn handle_previous(&mut self) {
        let elapsed = self
            .now_playing
            .as_ref()
            .map(|n| n.started_at.elapsed().as_secs_f64())
            .unwrap_or(0.0);
        self.finalize_now_playing();
        match self.queue.previous(elapsed) {
            PreviousOutcome::RestartCurrent => self.handle_seek(0.0),
            PreviousOutcome::Track(_, _) => self.load_current_into_engine(true),
        }
    }

    fn handle_seek(&mut self, pos_sec: f64) {
        if let Err(e) = self.engine.seek(Duration::from_secs_f64(pos_sec.max(0.0))) {
            emit(
                &self.app,
                PlayerEvent::Error {
                    message: format!("seek failed: {e}"),
                    track_id: None,
                },
            );
            return;
        }
        if let Some(np) = &mut self.now_playing {
            np.max_position_reached = np.max_position_reached.max(pos_sec);
        }
        emit(
            &self.app,
            PlayerEvent::Position {
                pos_sec,
                at_epoch_ms: now_epoch_ms(),
            },
        );
    }

    fn on_tick(&mut self) {
        if !self.has_track_loaded {
            return;
        }

        let (pos, is_finished) = self.engine.tick_status();
        if let Some(np) = &mut self.now_playing {
            np.max_position_reached = np.max_position_reached.max(pos);
        }

        let track_ended = self.now_playing.as_ref().map_or(false, |np| {
            is_finished && np.max_position_reached >= np.duration_sec - 0.5
        });

        if track_ended {
            self.handle_next();
            return;
        }

        if self.last_emitted_pos_at.elapsed() >= POSITION_EMIT_INTERVAL {
            emit(
                &self.app,
                PlayerEvent::Position {
                    pos_sec: pos,
                    at_epoch_ms: now_epoch_ms(),
                },
            );
            self.last_emitted_pos_at = Instant::now();
        }
    }


    fn finalize_now_playing(&mut self) {
        if let Some(np) = self.now_playing.take() {
            if np.duration_sec > 0.0 {
                let pct = (np.max_position_reached / np.duration_sec * 100.0).clamp(0.0, 100.0);
                if let Some(conn) = self.conn() {
                    let _ = playback::record_playback(&conn, np.track_id, np.source.type_str(), pct);
                }
            }
        }
    }


    fn snapshot(&self) -> PlayerStateSnapshot {
        let (track, duration) = match self.queue.current() {
            Some((t, _)) => (Some(t.clone()), t.duration_seconds),
            None => (None, 0),
        };
        let (pos, is_paused) = self.engine.state();
        PlayerStateSnapshot {
            current_track: track,
            is_playing: self.has_track_loaded && !is_paused,
            position_sec: pos,
            duration_sec: duration,
            repeat: self.queue.repeat_mode().as_str().to_string(),
            shuffle: self.queue.shuffle_enabled(),
            volume: if self.muted { 0.0 } else { self.volume },
            muted: self.muted,
            user_queue: self
                .queue
                .user_queue()
                .iter()
                .map(|q: &QueueItem| q.track.clone())
                .collect(),
            queue_view: self.build_queue_view(),
        }
    }

    fn build_queue_view(&self) -> QueueViewPayload {
        QueueViewPayload {
            context_source_type: self.queue.context_source().type_str().to_string(),
            context_label: self.queue.context_label().map(str::to_string),
            upcoming_context: self.queue.upcoming_context(20),
        }
    }

    fn emit_queue_changed(&self) {
        emit(
            &self.app,
            PlayerEvent::QueueChanged {
                user_queue: self
                    .queue
                    .user_queue()
                    .iter()
                    .map(|q| q.track.clone())
                    .collect(),
                context_len: self.queue.context_len(),
                context_position: self.queue.context_position(),
                queue_view: self.build_queue_view(),
            },
        );
    }

    fn emit_repeat_shuffle(&self) {
        emit(
            &self.app,
            PlayerEvent::RepeatShuffleChanged {
                repeat: self.queue.repeat_mode().as_str().to_string(),
                shuffle: self.queue.shuffle_enabled(),
            },
        );
    }
}

fn now_epoch_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}
