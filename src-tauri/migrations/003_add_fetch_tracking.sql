ALTER TABLE artist ADD COLUMN fetch_attempts INTEGER NOT NULL DEFAULT 0;
ALTER TABLE artist ADD COLUMN last_fetch_attempt TEXT;
