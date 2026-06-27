CREATE TABLE IF NOT EXISTS track (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    duration_sec INTEGER NOT NULL,
    cover_art TEXT,
    year INTEGER,
    is_favorite BOOLEAN DEFAULT FALSE,
    added_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    mtime INTEGER NOT NULL,
    file_size INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS artist (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    profile_image TEXT,
    banner_image TEXT
);

CREATE TABLE IF NOT EXISTS album (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    cover_art TEXT,
    year INTEGER
);

CREATE TABLE IF NOT EXISTS playlist (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS track_artist (
    track_id INTEGER,
    artist_id INTEGER,
    PRIMARY KEY (track_id, artist_id),
    FOREIGN KEY (track_id) REFERENCES track(id) ON DELETE CASCADE,
    FOREIGN KEY (artist_id) REFERENCES artist(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS album_track (
    album_id INTEGER,
    track_id INTEGER,
    track_number INTEGER NOT NULL,
    PRIMARY KEY (album_id, track_id),
    FOREIGN KEY (album_id) REFERENCES album(id) ON DELETE CASCADE,
    FOREIGN KEY (track_id) REFERENCES track(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS playlist_track (
    playlist_id INTEGER,
    track_id INTEGER,
    position INTEGER NOT NULL,
    PRIMARY KEY (playlist_id, track_id),
    FOREIGN KEY (playlist_id) REFERENCES playlist(id) ON DELETE CASCADE,
    FOREIGN KEY (track_id) REFERENCES track(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS source_dirs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS playback_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    track_id INTEGER NOT NULL,
    played_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    source_type TEXT CHECK(source_type IN ('ALBUM', 'PLAYLIST', 'ARTIST', 'FAVORITES', 'SEARCH', 'RECOMMENDATION', 'OTHER')),
    source_id INTEGER,
    completion_percent REAL CHECK(completion_percent >= 0 AND completion_percent <= 100),
    FOREIGN KEY(track_id) REFERENCES track(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS user_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    track_id INTEGER NOT NULL,
    position INTEGER NOT NULL,
    added_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(track_id) REFERENCES track(id) ON DELETE CASCADE
);

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

CREATE INDEX IF NOT EXISTS idx_track_title ON track(title);
CREATE INDEX IF NOT EXISTS idx_track_added_at ON track(added_at);
CREATE INDEX IF NOT EXISTS idx_artist_name ON artist(name);
CREATE INDEX IF NOT EXISTS idx_album_name ON album(name);
CREATE INDEX IF NOT EXISTS idx_track_artist_artist_id ON track_artist(artist_id);
CREATE INDEX IF NOT EXISTS idx_album_track_album_id ON album_track(album_id);
CREATE INDEX IF NOT EXISTS idx_playlist_track_playlist_id ON playlist_track(playlist_id);
CREATE INDEX IF NOT EXISTS idx_playback_history_played_at ON playback_history(played_at);
CREATE INDEX IF NOT EXISTS idx_playback_history_track_id ON playback_history(track_id);
