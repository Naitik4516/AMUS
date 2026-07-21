use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{OptionalExtension, params};

use super::source::RepeatMode;
use crate::models::Track;

type Conn = PooledConnection<SqliteConnectionManager>;

pub fn record_playback(
    conn: &Conn,
    track_id: i64,
    source_type: &str,
    completion_percent: f64,
) -> rusqlite::Result<()> {
    let clamped = completion_percent.clamp(0.0, 100.0);
    conn.execute(
        "INSERT INTO playback_history (track_id, source_type, completion_percent) VALUES (?, ?, ?)",
        params![track_id, source_type, clamped],
    )?;
    Ok(())
}

pub fn queue_insert_front(conn: &Conn, track_id: i64) -> rusqlite::Result<i64> {
    let min_pos: Option<f64> = conn
        .query_row("SELECT MIN(position) FROM user_queue", [], |r| r.get(0))
        .optional()?
        .flatten();
    let new_pos = min_pos.map(|p| p - 1.0).unwrap_or(1.0);
    conn.execute(
        "INSERT INTO user_queue (track_id, position) VALUES (?, ?)",
        params![track_id, new_pos],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn queue_insert_back(conn: &Conn, track_id: i64) -> rusqlite::Result<i64> {
    let max_pos: Option<f64> = conn
        .query_row("SELECT MAX(position) FROM user_queue", [], |r| r.get(0))
        .optional()?
        .flatten();
    let new_pos = max_pos.map(|p| p + 1.0).unwrap_or(1.0);
    conn.execute(
        "INSERT INTO user_queue (track_id, position) VALUES (?, ?)",
        params![track_id, new_pos],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn queue_insert_back_many(conn: &mut Conn, tracks: &[Track]) -> rusqlite::Result<Vec<i64>> {
    let max_pos: Option<f64> = conn
        .query_row("SELECT MAX(position) FROM user_queue", [], |r| r.get(0))
        .optional()?
        .flatten();

    // Start at max_pos + 1.0 so the first new row does not collide with the  existing last row and produce a non-deterministic ORDER BY result.
    let mut start = max_pos.map(|p| p + 1.0).unwrap_or(1.0);

    let tx = conn.transaction()?;
    let mut ids = Vec::with_capacity(tracks.len());
    for track in tracks {
        tx.execute(
            "INSERT INTO user_queue (track_id, position) VALUES (?, ?)",
            params![track.id, start],
        )?;
        ids.push(tx.last_insert_rowid());
        start += 1.0;
    }
    tx.commit()?;
    Ok(ids)
}

pub fn queue_remove(conn: &Conn, queue_id: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM user_queue WHERE id = ?", params![queue_id])?;
    Ok(())
}

pub fn queue_clear_all(conn: &Conn) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM user_queue", [])?;
    Ok(())
}

pub fn queue_pop_front(conn: &Conn) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM user_queue WHERE id = (SELECT id FROM user_queue ORDER BY position ASC LIMIT 1)",
        [],
    )?;
    Ok(())
}

pub fn queue_reorder(conn: &Conn, queue_id: i64, new_index: usize) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare("SELECT id, position FROM user_queue ORDER BY position ASC")?;
    let mut rows: Vec<(i64, f64)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    drop(stmt);

    let cur_idx = match rows.iter().position(|(id, _)| *id == queue_id) {
        Some(i) => i,
        None => return Ok(()),
    };
    let item = rows.remove(cur_idx);
    let insert_at = new_index.min(rows.len());

    let before = if insert_at == 0 {
        None
    } else {
        rows.get(insert_at - 1)
    };
    let after = rows.get(insert_at);

    let new_pos = match (before, after) {
        (Some((_, b)), Some((_, a))) => (b + a) / 2.0,
        (Some((_, b)), None) => b + 1.0,
        (None, Some((_, a))) => a - 1.0,
        (None, None) => 1.0,
    };

    if let (Some((_, b)), Some((_, a))) = (before, after) {
        if (a - b).abs() < 1e-6 {
            renumber_queue(conn, &rows, item.0, insert_at)?;
            return Ok(());
        }
    }

    conn.execute(
        "UPDATE user_queue SET position = ? WHERE id = ?",
        params![new_pos, item.0],
    )?;
    Ok(())
}

fn renumber_queue(
    conn: &Conn,
    others: &[(i64, f64)],
    moved_id: i64,
    insert_at: usize,
) -> rusqlite::Result<()> {
    let mut ids: Vec<i64> = others.iter().map(|(id, _)| *id).collect();
    ids.insert(insert_at, moved_id);
    // Wrap all per-row UPDATEs in a single transaction so they are atomic and WAL mode can batch them into one fsync instead of N.
    let tx = conn.unchecked_transaction()?;
    for (i, id) in ids.iter().enumerate() {
        tx.execute(
            "UPDATE user_queue SET position = ? WHERE id = ?",
            params![(i as f64) + 1.0, id],
        )?;
    }
    tx.commit()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Playing,
    Paused,
    Stopped,
}

#[derive(Debug, Clone)]
pub struct PlaybackState {
    pub state: PlayerState,
    pub current_track_id: Option<i64>,
    pub position_sec: f64,
    pub volume: f32,
    pub shuffle: bool,
    pub repeat_mode: RepeatMode,
}

impl PlaybackState {
    pub fn new() -> Self {
        Self {
            state: PlayerState::Stopped,
            current_track_id: None,
            position_sec: 0.0,
            volume: 1.0,
            shuffle: false,
            repeat_mode: RepeatMode::Off,
        }
    }

    pub fn play(&mut self) {
        match self.state {
            PlayerState::Stopped | PlayerState::Paused => self.state = PlayerState::Playing,
            PlayerState::Playing => {}
        }
    }

    pub fn pause(&mut self) {
        if self.state == PlayerState::Playing {
            self.state = PlayerState::Paused;
        }
    }

    pub fn stop(&mut self) {
        self.state = PlayerState::Stopped;
        self.position_sec = 0.0;
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.0);
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
    }

    pub fn cycle_repeat(&mut self) {
        self.repeat_mode = self.repeat_mode.cycle();
    }

    pub fn set_position(&mut self, pos: f64) {
        self.position_sec = pos.max(0.0);
    }

    pub fn advance_position(&mut self, delta: f64) {
        self.position_sec = (self.position_sec + delta).max(0.0);
    }

    pub fn is_playing(&self) -> bool {
        self.state == PlayerState::Playing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> PlaybackState {
        PlaybackState::new()
    }

    // ---------- state transitions ----------

    #[test]
    fn test_new_is_stopped() {
        let s = make_state();
        assert_eq!(s.state, PlayerState::Stopped);
        assert!(!s.is_playing());
        assert_eq!(s.position_sec, 0.0);
        assert_eq!(s.volume, 1.0);
        assert!(!s.shuffle);
        assert_eq!(s.repeat_mode, RepeatMode::Off);
    }

    #[test]
    fn test_stopped_to_playing() {
        let mut s = make_state();
        s.play();
        assert_eq!(s.state, PlayerState::Playing);
        assert!(s.is_playing());
    }

    #[test]
    fn test_playing_to_paused() {
        let mut s = make_state();
        s.play();
        s.pause();
        assert_eq!(s.state, PlayerState::Paused);
        assert!(!s.is_playing());
    }

    #[test]
    fn test_paused_to_playing() {
        let mut s = make_state();
        s.play();
        s.pause();
        s.play();
        assert_eq!(s.state, PlayerState::Playing);
    }

    #[test]
    fn test_playing_to_stopped() {
        let mut s = make_state();
        s.play();
        s.stop();
        assert_eq!(s.state, PlayerState::Stopped);
        assert!(!s.is_playing());
    }

    #[test]
    fn test_paused_to_stopped() {
        let mut s = make_state();
        s.play();
        s.pause();
        s.stop();
        assert_eq!(s.state, PlayerState::Stopped);
    }

    #[test]
    fn test_stopped_to_stopped_is_noop() {
        let mut s = make_state();
        s.stop();
        assert_eq!(s.state, PlayerState::Stopped);
    }

    // ---------- idempotent transitions ----------

    #[test]
    fn test_double_play_stays_playing() {
        let mut s = make_state();
        s.play();
        s.play();
        assert_eq!(s.state, PlayerState::Playing);
    }

    #[test]
    fn test_double_pause_stays_paused() {
        let mut s = make_state();
        s.play();
        s.pause();
        s.pause();
        assert_eq!(s.state, PlayerState::Paused);
    }

    // ---------- volume clamping ----------

    #[test]
    fn test_volume_clamps_to_max() {
        let mut s = make_state();
        s.set_volume(1.5);
        assert_eq!(s.volume, 1.0);
    }

    #[test]
    fn test_volume_clamps_to_min() {
        let mut s = make_state();
        s.set_volume(-0.5);
        assert_eq!(s.volume, 0.0);
    }

    #[test]
    fn test_volume_zero() {
        let mut s = make_state();
        s.set_volume(0.0);
        assert_eq!(s.volume, 0.0);
    }

    #[test]
    fn test_volume_one() {
        let mut s = make_state();
        s.set_volume(1.0);
        assert_eq!(s.volume, 1.0);
    }

    // ---------- repeat mode cycling ----------

    #[test]
    fn test_repeat_cycles_off_to_all() {
        let mut s = make_state();
        assert_eq!(s.repeat_mode, RepeatMode::Off);
        s.cycle_repeat();
        assert_eq!(s.repeat_mode, RepeatMode::All);
    }

    #[test]
    fn test_repeat_cycles_all_to_one() {
        let mut s = make_state();
        s.repeat_mode = RepeatMode::All;
        s.cycle_repeat();
        assert_eq!(s.repeat_mode, RepeatMode::One);
    }

    #[test]
    fn test_repeat_cycles_one_to_off() {
        let mut s = make_state();
        s.repeat_mode = RepeatMode::One;
        s.cycle_repeat();
        assert_eq!(s.repeat_mode, RepeatMode::Off);
    }

    #[test]
    fn test_shuffle_default_off() {
        let s = make_state();
        assert!(!s.shuffle);
    }

    #[test]
    fn test_toggle_shuffle_on() {
        let mut s = make_state();
        s.toggle_shuffle();
        assert!(s.shuffle);
    }

    #[test]
    fn test_toggle_shuffle_off() {
        let mut s = make_state();
        s.toggle_shuffle();
        s.toggle_shuffle();
        assert!(!s.shuffle);
    }

    #[test]
    fn test_toggle_shuffle_multiple() {
        let mut s = make_state();
        s.toggle_shuffle();
        s.toggle_shuffle();
        s.toggle_shuffle();
        assert!(s.shuffle);
    }

    // ---------- position tracking ----------

    #[test]
    fn test_set_position() {
        let mut s = make_state();
        s.set_position(42.5);
        assert_eq!(s.position_sec, 42.5);
    }

    #[test]
    fn test_advance_position_positive() {
        let mut s = make_state();
        s.set_position(10.0);
        s.advance_position(5.0);
        assert_eq!(s.position_sec, 15.0);
    }

    #[test]
    fn test_stop_resets_position() {
        let mut s = make_state();
        s.play();
        s.set_position(30.0);
        s.stop();
        assert_eq!(s.position_sec, 0.0);
    }

    // ---------- edge cases: negative values ----------

    #[test]
    fn test_set_position_negative_clamps_to_zero() {
        let mut s = make_state();
        s.set_position(-10.0);
        assert_eq!(s.position_sec, 0.0);
    }

    #[test]
    fn test_advance_position_negative_does_not_go_below_zero() {
        let mut s = make_state();
        s.set_position(5.0);
        s.advance_position(-10.0);
        assert_eq!(s.position_sec, 0.0);
    }

    #[test]
    fn test_advance_position_negative_partial() {
        let mut s = make_state();
        s.set_position(10.0);
        s.advance_position(-3.0);
        assert_eq!(s.position_sec, 7.0);
    }

    // ---------- combined behaviour ----------

    #[test]
    fn test_play_then_stop_preserves_volume_and_shuffle() {
        let mut s = make_state();
        s.set_volume(0.3);
        s.toggle_shuffle();
        s.play();
        s.stop();
        assert_eq!(s.volume, 0.3);
        assert!(s.shuffle);
        assert_eq!(s.state, PlayerState::Stopped);
    }
}
