ALTER TABLE track_lyrics ADD COLUMN fetch_attempts INTEGER NOT NULL DEFAULT 0;
ALTER TABLE track_lyrics ADD COLUMN last_fetch_attempt TEXT;
