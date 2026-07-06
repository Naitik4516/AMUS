use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};
use tokio::sync::oneshot;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tauri::AppHandle;

use crate::db;
use crate::models::Track;
use super::engine::AudioEngine;
use super::events::{emit, PlayerEvent};
use super::queue::{NextOutcome, PlaybackQueue, PreviousOutcome, QueueItem};
use super::source::{PlaybackSource, RepeatMode};
use super::playback;

pub type DbPool = Pool<SqliteConnectionManager>;

pub enum PlayerCommand {
    LoadContext { tracks: Vec<Track>, source: PlaybackSource, start_index: usize },
    PlayPause,
    Next,
    Previous,
    Seek(f64),
    SetVolume(f32),
    SetRepeat(RepeatMode),
    ToggleShuffle,
    EnqueueNext(Track),
    EnqueueEnd(Track),
    RemoveFromQueue(i64),
    ReorderQueue { queue_id: i64, new_index: usize },
    SetAutoplay(bool),
    GetState(oneshot::Sender<PlayerStateSnapshot>),
    Shutdown,
    Tick, // internal
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
    pub user_queue: Vec<Track>,
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
    tx_self: Sender<PlayerCommand>, // handed to the ticker thread
    app: AppHandle,
    pool: DbPool,

    engine: AudioEngine,
    queue: PlaybackQueue,
    volume: f32,
    autoplay_enabled: bool,
    now_playing: Option<NowPlaying>,
    has_track_loaded: bool,
    last_emitted_pos_at: Instant,
}

const TICK_INTERVAL: Duration = Duration::from_millis(250);
const POSITION_EMIT_INTERVAL: Duration = Duration::from_millis(1000);
const SESSION_CHECKPOINT_INTERVAL: Duration = Duration::from_secs(30);

impl PlayerActor {
    /// Spawns the actor on its own OS thread (audio output is not Send-safe
    /// across an async runtime's worker threads, so it never leaves this
    /// thread) plus a lightweight ticker thread that just posts `Tick`.
    /// Returns a command sender the rest of the app talks to.
    pub fn spawn(app: AppHandle, pool: DbPool) -> Sender<PlayerCommand> {
        let (tx, rx) = std::sync::mpsc::channel::<PlayerCommand>();
        let tx_ticker = tx.clone();
        let tx_self = tx.clone();

        std::thread::spawn(move || {
            std::thread::spawn(move || loop {
                std::thread::sleep(TICK_INTERVAL);
                if tx_ticker.send(PlayerCommand::Tick).is_err() {
                    break; // actor thread gone
                }
            });

            let engine = AudioEngine::new().expect("failed to init audio engine");
            let mut actor = PlayerActor {
                rx,
                tx_self,
                app,
                pool,
                engine,
                queue: PlaybackQueue::new(),
                volume: 1.0,
                autoplay_enabled: true,
                now_playing: None,
                has_track_loaded: false,
                last_emitted_pos_at: Instant::now(),
            };
            actor.restore_session();
            actor.run();
        });

        tx
    }

    fn conn(&self) -> r2d2::PooledConnection<SqliteConnectionManager> {
        self.pool.get().expect("db pool exhausted")
    }

    fn run(&mut self) {
        let mut last_checkpoint = Instant::now();
        while let Ok(cmd) = self.rx.recv() {
            match cmd {
                PlayerCommand::LoadContext { tracks, source, start_index } => {
                    self.finalize_now_playing(); // close out whatever was playing
                    self.queue.load_context(tracks, source, start_index);
                    self.load_current_into_engine(true);
                }
                PlayerCommand::PlayPause => self.toggle_play_pause(),
                PlayerCommand::Next => self.handle_next(),
                PlayerCommand::Previous => self.handle_previous(),
                PlayerCommand::Seek(pos) => self.handle_seek(pos),
                PlayerCommand::SetVolume(v) => {
                    self.volume = v.clamp(0.0, 1.0);
                    self.engine.set_volume(self.volume);
                    emit(&self.app, PlayerEvent::VolumeChanged { volume: self.volume });
                }
                PlayerCommand::SetRepeat(mode) => {
                    self.queue.set_repeat(mode);
                    self.emit_repeat_shuffle();
                }
                PlayerCommand::ToggleShuffle => {
                    self.queue.set_shuffle(!self.queue.shuffle_enabled());
                    self.emit_repeat_shuffle();
                }
                PlayerCommand::EnqueueNext(track) => {
                    let conn = self.conn();
                    if let Ok(db_id) = playback::queue_insert_front(&conn, track.id) {
                        self.queue.enqueue_next(db_id, track);
                        self.emit_queue_changed();
                    }
                }
                PlayerCommand::EnqueueEnd(track) => {
                    let conn = self.conn();
                    if let Ok(db_id) = playback::queue_insert_back(&conn, track.id) {
                        self.queue.enqueue_end(db_id, track);
                        self.emit_queue_changed();
                    }
                }
                PlayerCommand::RemoveFromQueue(db_id) => {
                    self.queue.remove_from_queue(db_id);
                    let conn = self.conn();
                    let _ = playback::queue_remove(&conn, db_id);
                    self.emit_queue_changed();
                }
                PlayerCommand::ReorderQueue { queue_id, new_index } => {
                    self.queue.reorder_queue(queue_id, new_index);
                    let conn = self.conn();
                    let _ = playback::queue_reorder(&conn, queue_id, new_index);
                    self.emit_queue_changed();
                }
                PlayerCommand::SetAutoplay(v) => self.autoplay_enabled = v,
                PlayerCommand::GetState(reply) => {
                    let _ = reply.send(self.snapshot());
                }
                PlayerCommand::Shutdown => {
                    self.finalize_now_playing();
                    self.save_session();
                    break;
                }
                PlayerCommand::Tick => {
                    self.on_tick();
                }
            }

            if last_checkpoint.elapsed() >= SESSION_CHECKPOINT_INTERVAL {
                self.save_session();
                last_checkpoint = Instant::now();
            }
        }
    }

    // ---------- core transitions ----------

    fn load_current_into_engine(&mut self, autoplay: bool) {
        let Some((track, source)) = self.queue.current().cloned() else {
            self.has_track_loaded = false;
            emit(&self.app, PlayerEvent::PlaybackEnded);
            return;
        };

        let conn = self.conn();
        let path = match db::get_track_path_by_id(&conn, track.id) {
            Ok(p) => p,
            _ => {
                emit(&self.app, PlayerEvent::Error {
                    message: "track file path not found".into(),
                    track_id: Some(track.id),
                });
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
                emit(&self.app, PlayerEvent::TrackChanged {
                    track: track.clone(),
                    duration_sec: track.duration_seconds,
                    source,
                });
                emit(&self.app, PlayerEvent::StateChanged { is_playing: autoplay });
                self.emit_queue_changed();
            }
            Err(e) => {
                emit(&self.app, PlayerEvent::Error {
                    message: format!("failed to load track: {e}"),
                    track_id: Some(track.id),
                });
                self.handle_next(); // corrupted/missing file, skip forward
            }
        }
    }

    fn toggle_play_pause(&mut self) {
        if !self.has_track_loaded {
            return;
        }
        if self.engine.is_paused() {
            self.engine.play();
            emit(&self.app, PlayerEvent::StateChanged { is_playing: true });
        } else {
            self.engine.pause();
            emit(&self.app, PlayerEvent::StateChanged { is_playing: false });
        }
    }

    fn handle_next(&mut self) {
        self.finalize_now_playing();
        match self.queue.advance_next() {
            NextOutcome::Track(_, _) => self.load_current_into_engine(true),
            NextOutcome::NeedsAutoplay => self.try_autoplay(),
            NextOutcome::End => {
                self.has_track_loaded = false;
                self.engine.stop();
                emit(&self.app, PlayerEvent::PlaybackEnded);
            }
        }
    }

    fn try_autoplay(&mut self) {
        if !self.autoplay_enabled {
            self.has_track_loaded = false;
            self.engine.stop();
            emit(&self.app, PlayerEvent::PlaybackEnded);
            return;
        }
        let Some(last_id) = self.now_playing_or_last_track_id() else {
            emit(&self.app, PlayerEvent::PlaybackEnded);
            return;
        };
        let conn = self.conn();
        match db::get_similar_tracks(&conn, last_id, 20) {
            Ok(recs) if !recs.is_empty() => {
                self.queue.extend_with_autoplay(recs);
                self.load_current_into_engine(true);
            }
            _ => {
                self.has_track_loaded = false;
                emit(&self.app, PlayerEvent::PlaybackEnded);
            }
        }
    }

    fn now_playing_or_last_track_id(&self) -> Option<i64> {
        self.queue.current().map(|(t, _)| t.id)
    }

    fn handle_previous(&mut self) {
        let elapsed = self.now_playing.as_ref().map(|n| n.started_at.elapsed().as_secs_f64()).unwrap_or(0.0);
        self.finalize_now_playing();
        match self.queue.previous(elapsed) {
            PreviousOutcome::RestartCurrent => self.handle_seek(0.0),
            PreviousOutcome::Track(_, _) => self.load_current_into_engine(true),
        }
    }

    fn handle_seek(&mut self, pos_sec: f64) {
        if let Err(e) = self.engine.seek(Duration::from_secs_f64(pos_sec.max(0.0))) {
            emit(&self.app, PlayerEvent::Error { message: format!("seek failed: {e}"), track_id: None });
            return;
        }
        if let Some(np) = &mut self.now_playing {
            np.max_position_reached = np.max_position_reached.max(pos_sec);
        }
        emit(&self.app, PlayerEvent::Position { pos_sec, at_epoch_ms: now_epoch_ms() });
    }

    fn on_tick(&mut self) {
        if !self.has_track_loaded {
            return;
        }

        let pos = self.engine.position().as_secs_f64();
        if let Some(np) = &mut self.now_playing {
            np.max_position_reached = np.max_position_reached.max(pos);
        }

        if self.engine.is_finished() && !self.engine.is_paused() {
            // is_paused() guards against the split second right after `load()`
            // (paused, empty) being mistaken for "finished playing"
        }

        // natural end-of-track detection: sink drained AND we've actually
        // been playing (max_position_reached close to duration)
        let track_ended = self.now_playing.as_ref().map_or(false, |np| {
            self.engine.is_finished() && np.max_position_reached >= np.duration_sec - 0.5
        });

        if track_ended {
            self.handle_next();
            return;
        }

        if self.last_emitted_pos_at.elapsed() >= POSITION_EMIT_INTERVAL {
            emit(&self.app, PlayerEvent::Position { pos_sec: pos, at_epoch_ms: now_epoch_ms() });
            self.last_emitted_pos_at = Instant::now();
        }
    }

    // ---------- playback recording ----------

    fn finalize_now_playing(&mut self) {
        if let Some(np) = self.now_playing.take() {
            if np.duration_sec > 0.0 {
                let pct = (np.max_position_reached / np.duration_sec * 100.0).clamp(0.0, 100.0);
                let conn = self.conn();
                let _ = playback::record_playback(&conn, np.track_id, np.source.type_str(), pct);
            }
        }
    }

    // ---------- session persistence ----------

    fn save_session(&self) {
        let conn = self.conn();
        let session = playback::SessionState {
            context_type: Some(self.queue.context_source().type_str().to_string()),
            context_id: self.queue.context_source().source_id(),
            current_track_id: self.queue.current().map(|(t, _)| t.id),
            context_position: self.queue.context_position().map(|p| p as i64),
            position_sec: self.engine.position().as_secs_f64(),
            repeat_mode: self.queue.repeat_mode(),
            shuffle: self.queue.shuffle_enabled(),
            shuffle_order: None, // populated below if present
        };
        let _ = playback::session_save(&conn, &session);
    }

    fn restore_session(&mut self) {
        let conn = self.conn();
        let Ok(Some(session)) = playback::session_load(&conn) else { return };
        drop(conn);

        self.queue.set_repeat(session.repeat_mode);
        self.queue.set_shuffle(session.shuffle);

        // Re-fetch the actual context tracks from source of truth (album/
        // playlist may have changed since last session) rather than trusting
        // a stale snapshot. Left as a call-site TODO since it depends on
        // your album/playlist fetch helpers:
        //
        // if let (Some(ctype), Some(cid)) = (&session.context_type, session.context_id) {
        //     let tracks = fetch_context_tracks(ctype, cid);
        //     self.queue.load_context(tracks, PlaybackSource::from_db(ctype, Some(cid)),
        //                              session.context_position.unwrap_or(0) as usize);
        // }
        //
        // Load track paused at saved position rather than autoplaying on launch:
        if session.current_track_id.is_some() {
            self.load_current_into_engine(false);
            let _ = self.engine.seek(Duration::from_secs_f64(session.position_sec));
        }

        // restore explicit user queue
        let conn = self.conn();
        if let Ok(rows) = playback::queue_load_all(&conn) {
            for (db_id, track_id) in rows {
                if let Ok(track) = db::get_track_by_id(&conn, track_id) {
                    self.queue.enqueue_end(db_id, track);
                }
            }
        }
    }

    fn snapshot(&self) -> PlayerStateSnapshot {
        let (track, duration) = match self.queue.current() {
            Some((t, _)) => (Some(t.clone()), t.duration_seconds),
            None => (None, 0),
        };
        PlayerStateSnapshot {
            current_track: track,
            is_playing: self.has_track_loaded && !self.engine.is_paused(),
            position_sec: self.engine.position().as_secs_f64(),
            duration_sec: duration,
            repeat: self.queue.repeat_mode().as_str().to_string(),
            shuffle: self.queue.shuffle_enabled(),
            volume: self.volume,
            user_queue: self.queue.user_queue().iter().map(|q: &QueueItem| q.track.clone()).collect(),
        }
    }

    fn emit_queue_changed(&self) {
        emit(&self.app, PlayerEvent::QueueChanged {
            user_queue: self.queue.user_queue().iter().map(|q| q.track.clone()).collect(),
            context_len: self.queue.context_len(),
            context_position: self.queue.context_position(),
        });
    }

    fn emit_repeat_shuffle(&self) {
        emit(&self.app, PlayerEvent::RepeatShuffleChanged {
            repeat: self.queue.repeat_mode().as_str().to_string(),
            shuffle: self.queue.shuffle_enabled(),
        });
    }
}

fn now_epoch_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}
