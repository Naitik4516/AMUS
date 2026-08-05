use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::sync::mpsc::{Receiver, SyncSender};
use std::time::Duration;
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
    RemoveFromContext(i64),
    ClearQueue,
    ReorderQueue {
        queue_id: i64,
        new_index: usize,
    },
    ReorderContext {
        from_rel: usize,
        to_rel: usize,
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
}

pub struct PlayerActor {
    rx: Receiver<PlayerCommand>,
    app: AppHandle,
    pool: DbPool,

    engine: Option<AudioEngine>,
    queue: PlaybackQueue,
    volume: f32,
    muted: bool,
    volume_before_mute: f32,
    autoplay_enabled: bool,
    autoplay_chain_failed: bool,
    now_playing: Option<NowPlaying>,
    has_track_loaded: bool,
    position_emit_counter: u32,
}

const TICK_INTERVAL: Duration = Duration::from_millis(250);
const POSITION_EMIT_EVERY_TICKS: u32 = 20;

impl PlayerActor {
    pub fn spawn(app: AppHandle, pool: DbPool) -> SyncSender<PlayerCommand> {
        let (tx, rx) = std::sync::mpsc::sync_channel::<PlayerCommand>(128);

        std::thread::Builder::new()
            .name("player-actor".into())
            .spawn(move || {
                let engine = match AudioEngine::new() {
                    Ok(e) => Some(e),
                    Err(e) => {
                        tracing::error!(error = %e, "failed to init audio engine");
                        // Keep the actor alive in degraded mode so commands still respond; playback will report an error instead.
                        emit(
                            &app,
                            PlayerEvent::Error {
                                message: format!("audio engine unavailable: {e}"),
                                track_id: None,
                            },
                        );
                        None
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
                    autoplay_chain_failed: false,
                    now_playing: None,
                    has_track_loaded: false,
                    position_emit_counter: 0,
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
                tracing::warn!(error = %e, "failed to get db connection");
                None
            }
        }
    }

    fn run(&mut self) {
        loop {
            let is_active =
                self.has_track_loaded && self.engine.as_ref().is_some_and(|e| !e.is_paused());
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

            let keep_running = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut shutting_down = false;
                match cmd {
                    PlayerCommand::LoadContext {
                        tracks,
                        source,
                        start_index,
                        context_label,
                    } => {
                        self.finalize_now_playing();
                        self.autoplay_chain_failed = false;
                        self.queue
                            .load_context(tracks, source, start_index, context_label);
                        self.load_current_into_engine(true);
                    }
                    PlayerCommand::PlayPause => self.toggle_play_pause(),
                    PlayerCommand::Play => self.handle_play(),
                    PlayerCommand::Pause => self.handle_pause(),
                    PlayerCommand::Stop => self.close_player(),
                    PlayerCommand::Next => self.handle_next(),
                    PlayerCommand::Previous => self.handle_previous(),
                    PlayerCommand::Seek(pos) => self.handle_seek(pos),
                    PlayerCommand::SeekRelative(delta) => {
                        let (pos, _) = self.engine.as_ref().map_or((0.0, true), |e| e.state());
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
                            if let Ok(db_ids) = playback::queue_insert_back_many(&mut conn, &tracks)
                            {
                                for (db_id, track) in db_ids.into_iter().zip(tracks) {
                                    self.queue.enqueue_end(db_id, track);
                                }
                                self.emit_queue_changed();
                            }
                        }
                    }
                    PlayerCommand::RemoveFromQueue(db_id) => {
                        self.queue.remove_from_user_queue(db_id);
                        if let Some(conn) = self.conn() {
                            let _ = playback::queue_remove(&conn, db_id);
                        }
                        self.emit_queue_changed();
                    }
                    PlayerCommand::RemoveFromContext(track_id) => {
                        use crate::player::queue::RemoveFromContextOutcome;
                        match self.queue.remove_from_context(track_id) {
                            Some(RemoveFromContextOutcome::RemovedCurrent { resume: Some(_) }) => {
                                // Keep playing: record the partial listen, then load the
                                // next track (as chosen by the queue's playback order).
                                self.finalize_now_playing();
                                self.load_current_into_engine(true);
                            }
                            Some(RemoveFromContextOutcome::RemovedCurrent { resume: None }) => {
                                // The removed track was the last one in the context.
                                self.finalize_now_playing();
                                if self.autoplay_enabled {
                                    self.try_autoplay();
                                } else {
                                    self.stop_playback();
                                }
                            }
                            _ => {}
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
                    PlayerCommand::ReorderContext { from_rel, to_rel } => {
                        self.queue.reorder_context(from_rel, to_rel);
                        self.emit_queue_changed();
                    }
                    PlayerCommand::SetAutoplay(v) => self.autoplay_enabled = v,
                    PlayerCommand::PlayTrackFromContext(track_id) => {
                        if self.queue.jump_to_track(track_id) {
                            // Record the partial listen of the track we're leaving.
                            self.finalize_now_playing();
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
                        self.autoplay_chain_failed = false;
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
                                if let Ok(db_ids) =
                                    playback::queue_insert_back_many(&mut conn, &user_queue_tracks)
                                {
                                    for (db_id, track) in db_ids.into_iter().zip(user_queue_tracks)
                                    {
                                        self.queue.enqueue_end(db_id, track);
                                    }
                                }
                            }
                        }
                        self.queue.set_repeat(repeat);
                        self.queue.set_shuffle(shuffle);
                        self.emit_repeat_shuffle();
                        self.volume = volume.clamp(0.0, 1.0);
                        if let Some(engine) = self.engine.as_ref() {
                            engine.set_volume(self.volume);
                        }
                        emit(
                            &self.app,
                            PlayerEvent::VolumeChanged {
                                volume: self.volume,
                            },
                        );
                        self.emit_queue_changed();
                        if let Some(engine) = self.engine.as_ref() {
                            let _ = engine.seek(Duration::from_secs_f64(position_sec.max(0.0)));
                        }
                        if let Some(np) = &mut self.now_playing {
                            np.max_position_reached = np.max_position_reached.max(position_sec);
                        }
                        if self.has_track_loaded {
                            if let Some(engine) = self.engine.as_ref() {
                                engine.play();
                            }
                            emit(&self.app, PlayerEvent::StateChanged { is_playing: true });
                        }
                        emit(
                            &self.app,
                            PlayerEvent::Position {
                                pos_sec: position_sec,
                                at_epoch_ms: now_epoch_ms(),
                                is_playing: self.has_track_loaded,
                            },
                        );
                    }
                    PlayerCommand::GetState(reply) => {
                        let _ = reply.send(self.snapshot());
                    }
                    PlayerCommand::Shutdown => {
                        self.finalize_now_playing();
                        shutting_down = true;
                    }
                    PlayerCommand::Tick => {
                        self.on_tick();
                    }
                }
                !shutting_down
            }));

            match keep_running {
                Ok(true) => {}
                Ok(false) => break,
                Err(payload) => {
                    let panic_msg = payload
                        .downcast_ref::<&str>()
                        .copied()
                        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
                        .unwrap_or("unknown panic");
                    tracing::error!(
                        panic = panic_msg,
                        "player actor: command handler panicked; keeping actor alive"
                    );
                    emit(
                        &self.app,
                        PlayerEvent::Error {
                            message: format!("internal player error: {panic_msg}"),
                            track_id: None,
                        },
                    );
                }
            }
        }
    }

    fn load_current_into_engine(&mut self, autoplay: bool) {
        let max_attempts = self.queue.context_len() + self.queue.user_queue().len() + 20;
        let mut attempts: usize = 0;

        loop {
            attempts += 1;
            if attempts > max_attempts.max(1) {
                self.stop_playback();
                return;
            }

            let Some((track, source)) = self.queue.current().cloned() else {
                self.has_track_loaded = false;
                emit(&self.app, PlayerEvent::PlaybackEnded);
                return;
            };

            let Some(path) = (match self.conn() {
                Some(conn) => db::get_track_path_by_id(&conn, track.id).ok(),
                None => None,
            }) else {
                emit(
                    &self.app,
                    PlayerEvent::Error {
                        message: "track file path not found".into(),
                        track_id: Some(track.id),
                    },
                );
                self.skip_current_track();
                continue;
            };

            let load_result = {
                let Some(engine) = self.engine.as_mut() else {
                    emit(
                        &self.app,
                        PlayerEvent::Error {
                            message: "audio engine unavailable (no output device)".into(),
                            track_id: Some(track.id),
                        },
                    );
                    self.stop_playback();
                    return;
                };
                engine.load(&path)
            };

            match load_result {
                Ok(()) => {
                    self.has_track_loaded = true;
                    self.autoplay_chain_failed = false;
                    if let Some(engine) = self.engine.as_mut() {
                        engine.set_volume(self.volume);
                    }
                    self.now_playing = Some(NowPlaying {
                        track_id: track.id,
                        duration_sec: track.duration_seconds as f64,
                        max_position_reached: 0.0,
                        source: source.clone(),
                    });
                    if autoplay {
                        if let Some(engine) = self.engine.as_ref() {
                            engine.play();
                        }
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
                    return;
                }
                Err(e) => {
                    emit(
                        &self.app,
                        PlayerEvent::Error {
                            message: format!("failed to load track: {e}"),
                            track_id: Some(track.id),
                        },
                    );
                    self.skip_current_track();
                }
            }
        }
    }

    /// Advance past a track that failed to load (broken file, missing path).
    fn skip_current_track(&mut self) {
        match self.queue.advance_next() {
            NextOutcome::Track(_, _) => {}
            NextOutcome::NeedsAutoplay => self.try_autoplay(),
            NextOutcome::End => self.stop_playback(),
        }
    }

    fn toggle_play_pause(&mut self) {
        if !self.has_track_loaded {
            return;
        }
        if self.engine.as_ref().is_none_or(|e| e.is_paused()) {
            self.handle_play();
        } else {
            self.handle_pause();
        }
    }

    fn handle_play(&mut self) {
        if !self.has_track_loaded {
            return;
        }
        let Some(engine) = self.engine.as_ref() else {
            return;
        };
        engine.play();
        let (pos, _) = engine.state();
        emit(
            &self.app,
            PlayerEvent::Position {
                pos_sec: pos,
                at_epoch_ms: now_epoch_ms(),
                is_playing: true,
            },
        );
        emit(&self.app, PlayerEvent::StateChanged { is_playing: true });
    }

    fn handle_pause(&mut self) {
        if !self.has_track_loaded {
            return;
        }
        let Some(engine) = self.engine.as_ref() else {
            return;
        };
        engine.pause();
        let (pos, _) = engine.state();
        emit(
            &self.app,
            PlayerEvent::Position {
                pos_sec: pos,
                at_epoch_ms: now_epoch_ms(),
                is_playing: false,
            },
        );
        emit(&self.app, PlayerEvent::StateChanged { is_playing: false });
    }

    fn apply_volume(&mut self, v: f32) {
        let clamped = v.clamp(0.0, 1.0);
        self.volume = clamped;
        if self.muted {
            self.muted = false;
        }
        if let Some(engine) = self.engine.as_ref() {
            engine.set_volume(self.volume);
        }
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
        } else {
            self.volume_before_mute = if self.volume > 0.0 {
                self.volume
            } else {
                self.volume_before_mute
            };
            self.muted = true;
        }
        if let Some(engine) = self.engine.as_ref() {
            engine.set_volume(if self.muted { 0.0 } else { self.volume });
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
                if let Some(engine) = self.engine.as_ref() {
                    engine.stop();
                }
                emit(&self.app, PlayerEvent::StateChanged { is_playing: false });
                emit(&self.app, PlayerEvent::PlaybackEnded);
            }
        }
    }

    fn try_autoplay(&mut self) {
        if !self.autoplay_enabled || self.autoplay_chain_failed {
            self.stop_playback();
            return;
        }
        self.autoplay_chain_failed = true;
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
                tracing::warn!(track_id = last_id, "autoplay: no similar tracks found");
                self.stop_playback();
            }
            Err(e) => {
                tracing::warn!(error = %e, track_id = last_id, "autoplay: get_similar_tracks failed");
                self.stop_playback();
            }
        }
    }

    fn stop_playback(&mut self) {
        self.has_track_loaded = false;
        if let Some(engine) = self.engine.as_ref() {
            engine.stop();
        }
        emit(&self.app, PlayerEvent::StateChanged { is_playing: false });
        emit(&self.app, PlayerEvent::PlaybackEnded);
    }

    fn close_player(&mut self) {
        self.finalize_now_playing();
        self.now_playing = None;
        self.stop_playback();
    }

    fn handle_previous(&mut self) {
        let (pos, _) = self.engine.as_ref().map_or((0.0, true), |e| e.state());
        self.finalize_now_playing();
        match self.queue.previous(pos) {
            PreviousOutcome::RestartCurrent => self.handle_seek(0.0),
            PreviousOutcome::Track(_, _) => self.load_current_into_engine(true),
        }
    }

    fn handle_seek(&mut self, pos_sec: f64) {
        let Some(engine) = self.engine.as_ref() else {
            return;
        };
        if let Err(e) = engine.seek(Duration::from_secs_f64(pos_sec.max(0.0))) {
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
        let (_, is_paused) = engine.state();
        emit(
            &self.app,
            PlayerEvent::Position {
                pos_sec,
                at_epoch_ms: now_epoch_ms(),
                is_playing: !is_paused,
            },
        );
    }

    fn on_tick(&mut self) {
        if !self.has_track_loaded {
            return;
        }
        let Some(engine) = self.engine.as_ref() else {
            return;
        };

        let (pos, is_finished) = engine.tick_status();
        if let Some(np) = &mut self.now_playing {
            np.max_position_reached = np.max_position_reached.max(pos);
        }

        // Periodically report position so OS media controls show live progress.
        self.position_emit_counter = self.position_emit_counter.wrapping_add(1);
        if self.position_emit_counter % POSITION_EMIT_EVERY_TICKS == 0 {
            let (pos, is_paused) = engine.state();
            emit(
                &self.app,
                PlayerEvent::Position {
                    pos_sec: pos,
                    at_epoch_ms: now_epoch_ms(),
                    is_playing: !is_paused,
                },
            );
        }

        let track_ended = self
            .now_playing
            .as_ref()
            .is_some_and(|np| is_finished && np.max_position_reached >= np.duration_sec - 0.5);

        if track_ended {
            self.handle_next();
        }
    }

    fn finalize_now_playing(&mut self) {
        if let Some(np) = self.now_playing.take() {
            if np.duration_sec > 0.0 {
                let pct = (np.max_position_reached / np.duration_sec * 100.0).clamp(0.0, 100.0);
                if let Some(conn) = self.conn() {
                    if let Err(e) =
                        playback::record_playback(&conn, np.track_id, np.source.type_str(), pct)
                    {
                        tracing::warn!(error = %e, "failed to record playback history");
                    }
                }
            }
        }
    }

    fn snapshot(&self) -> PlayerStateSnapshot {
        let (track, duration) = match self.queue.current() {
            Some((t, _)) => (Some(t.clone()), t.duration_seconds),
            None => (None, 0),
        };
        let (pos, is_paused) = self.engine.as_ref().map_or((0.0, true), |e| e.state());
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
                .map(|q: &QueueItem| {
                    let mut t = q.track.clone();
                    t.queue_id = Some(q.db_id);
                    t
                })
                .collect(),
            queue_view: self.build_queue_view(),
        }
    }

    fn build_queue_view(&self) -> QueueViewPayload {
        QueueViewPayload {
            context_source_type: self.queue.context_source().type_str().to_string(),
            context_label: self.queue.context_label().map(str::to_string),
            upcoming_context: self.queue.upcoming_context(self.queue.context_len()),
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
                    .map(|q| {
                        let mut t = q.track.clone();
                        t.queue_id = Some(q.db_id);
                        t
                    })
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
