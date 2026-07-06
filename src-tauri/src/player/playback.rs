use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{OptionalExtension, params};

use crate::player::source::RepeatMode;

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

// ---------- user_queue persistence (fractional positions) ----------

pub fn queue_load_all(conn: &Conn) -> rusqlite::Result<Vec<(i64, i64)>> {
    // returns (user_queue.id, track_id) ordered by position
    let mut stmt = conn.prepare("SELECT id, track_id FROM user_queue ORDER BY position ASC")?;
    let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
    rows.collect()
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

pub fn queue_remove(conn: &Conn, queue_id: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM user_queue WHERE id = ?", params![queue_id])?;
    Ok(())
}

pub fn queue_pop_front(conn: &Conn) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM user_queue WHERE id = (SELECT id FROM user_queue ORDER BY position ASC LIMIT 1)",
        [],
    )?;
    Ok(())
}

/// Reorders by placing `queue_id` between its new neighbors; renumbers the
/// whole table on the rare occasion fractional gaps get too tight.
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

    // gap collapsed too far -> renumber everything cleanly
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
    for (i, id) in ids.iter().enumerate() {
        conn.execute(
            "UPDATE user_queue SET position = ? WHERE id = ?",
            params![(i as f64) + 1.0, id],
        )?;
    }
    Ok(())
}

// ---------- session persistence ----------

pub struct SessionState {
    pub context_type: Option<String>,
    pub context_id: Option<i64>,
    pub current_track_id: Option<i64>,
    pub context_position: Option<i64>,
    pub position_sec: f64,
    pub repeat_mode: RepeatMode,
    pub shuffle: bool,
    pub shuffle_order: Option<Vec<usize>>,
}

pub fn session_load(conn: &Conn) -> rusqlite::Result<Option<SessionState>> {
    conn.query_row(
        "SELECT context_type, context_id, current_track_id, context_position,
                position_sec, repeat_mode, shuffle, shuffle_order
         FROM playback_session WHERE id = 1",
        [],
        |r| {
            let shuffle_order_json: Option<String> = r.get(7)?;
            Ok(SessionState {
                context_type: r.get(0)?,
                context_id: r.get(1)?,
                current_track_id: r.get(2)?,
                context_position: r.get(3)?,
                position_sec: r.get(4)?,
                repeat_mode: RepeatMode::from_str(&r.get::<_, String>(5)?),
                shuffle: r.get(6)?,
                shuffle_order: shuffle_order_json.and_then(|s| serde_json::from_str(&s).ok()),
            })
        },
    )
    .optional()
}

pub fn session_save(conn: &Conn, s: &SessionState) -> rusqlite::Result<()> {
    let shuffle_order_json = s
        .shuffle_order
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap());
    conn.execute(
        "INSERT INTO playback_session
            (id, context_type, context_id, current_track_id, context_position,
             position_sec, repeat_mode, shuffle, shuffle_order, updated_at)
         VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
         ON CONFLICT(id) DO UPDATE SET
            context_type=excluded.context_type, context_id=excluded.context_id,
            current_track_id=excluded.current_track_id, context_position=excluded.context_position,
            position_sec=excluded.position_sec, repeat_mode=excluded.repeat_mode,
            shuffle=excluded.shuffle, shuffle_order=excluded.shuffle_order,
            updated_at=CURRENT_TIMESTAMP",
        params![
            s.context_type,
            s.context_id,
            s.current_track_id,
            s.context_position,
            s.position_sec,
            s.repeat_mode.as_str(),
            s.shuffle,
            shuffle_order_json,
        ],
    )?;
    Ok(())
}
