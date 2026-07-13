use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{OptionalExtension, params};

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

pub fn queue_insert_back_many(
    conn: &mut Conn,
    tracks: &[Track],
) -> rusqlite::Result<Vec<i64>> {
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
