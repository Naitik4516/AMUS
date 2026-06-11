use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackDetails {
    pub id: i64,
    pub path: String,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub duration_seconds: u32,
    pub is_favorite: bool,
    pub mtime: i64,
    pub play_count: i64,
    pub last_played_at: Option<DateTime<Utc>>,
    pub skipped_count: i64,
    pub last_skipped_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Track {
    pub id: i64,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_seconds: u32,
    pub is_favorite: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaybackState {
    pub current_track_id: Option<i64>,
    pub position_ms: u32,
    pub shuffle_enabled: bool,
    pub repeat_mode: u8, // 0: Off, 1: Track, 2: All
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Artist {
    pub id: i64,
    pub name: String,
    pub profile_picture: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Album {
    pub id: i64,
    pub name: String,
    pub cover_art: Option<String>,
}
