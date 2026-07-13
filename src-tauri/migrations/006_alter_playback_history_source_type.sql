-- The track_stats view depends on playback_history; drop it before the rename dance
-- so SQLite doesn't complain about a dangling view reference, then recreate it.
DROP VIEW IF EXISTS track_stats;

CREATE TABLE playback_history_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    track_id INTEGER NOT NULL,
    played_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    source_type TEXT CHECK(source_type IN ('ALBUM', 'PLAYLIST', 'ARTIST', 'FAVORITES', 'DIRECT', 'QUEUE', 'OTHER')),
    source_id INTEGER,
    completion_percent REAL CHECK(completion_percent >= 0 AND completion_percent <= 100),
    FOREIGN KEY(track_id) REFERENCES track(id) ON DELETE CASCADE
);

INSERT INTO playback_history_new (id, track_id, played_at, source_type, source_id, completion_percent)
SELECT id, track_id, played_at,
    CASE WHEN source_type IN ('ALBUM', 'PLAYLIST', 'ARTIST', 'FAVORITES', 'DIRECT', 'QUEUE', 'OTHER') THEN source_type ELSE 'OTHER' END,
    source_id, completion_percent
FROM playback_history;

DROP TABLE playback_history;

ALTER TABLE playback_history_new RENAME TO playback_history;

CREATE VIEW IF NOT EXISTS track_stats AS
SELECT
    t.id AS track_id,
    COUNT(ph.id) AS play_count,
    MAX(ph.played_at) AS last_played_at,
    SUM(CASE WHEN ph.completion_percent < 50 THEN 1 ELSE 0 END) AS skip_count,
    MAX(CASE WHEN ph.completion_percent < 50 THEN ph.played_at ELSE NULL END) AS last_skipped_at
FROM track t
LEFT JOIN playback_history ph ON t.id = ph.track_id
GROUP BY t.id;
