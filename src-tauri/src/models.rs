use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackDetails {
    pub id: i64,
    pub path: String,
    pub title: String,
    pub artists: Vec<Artist>,
    pub album: Album,
    pub duration_seconds: u32,
    pub is_favorite: bool,
    pub mtime: i64,
    pub play_count: i64,
    pub last_played_at: Option<DateTime<Utc>>,
    pub skipped_count: i64,
    pub last_skipped_at: Option<DateTime<Utc>>,
    pub cover_art: Option<String>,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Track {
    pub id: i64,
    pub title: String,
    pub artists: Vec<Artist>,
    pub album: Album,
    pub duration_seconds: u32,
    pub is_favorite: bool,
    pub cover_art: Option<String>,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaybackState {
    pub current_track_id: Option<i64>,
    pub position_ms: u32,
    pub shuffle_enabled: bool,
    pub repeat_mode: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Artist {
    pub id: i64,
    pub name: String,
    pub profile_image: Option<String>,
    pub banner_image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Album {
    pub id: i64,
    pub name: String,
    pub cover_art: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepeatMode {
    Off = 0,
    Track = 1,
    All = 2,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Album = 0,
    Playlist = 1,
    Artist = 2,
    Favorites = 3,
    Other = 4,
}

impl SourceType {
    pub fn to_db_string(&self) -> &'static str {
        match self {
            SourceType::Album => "ALBUM",
            SourceType::Playlist => "PLAYLIST",
            SourceType::Artist => "ARTIST",
            SourceType::Favorites => "FAVORITES",
            SourceType::Other => "OTHER",
        }
    }
}

// ---------------------------------------------------------------------------
// Stats models
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatsOverview {
    pub total_tracks: i64,
    pub total_artists: i64,
    pub total_albums: i64,
    pub total_plays: i64,
    pub total_listening_time_sec: i64,
    pub avg_daily_listening_min: f64,
    pub total_file_size_bytes: i64,
    pub avg_file_size_mb: f64,
    pub largest_file_mb: f64,
    pub format_distribution: Vec<FormatStat>,
    pub pct_library_played: f64,
    pub unplayed_tracks: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FormatStat {
    pub format: String,
    pub count: i64,
    pub percentage: f64,
    pub total_bytes: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopTrack {
    pub track: Track,
    pub play_count: i64,
    pub total_listening_time_sec: i64,
    pub last_played_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopArtist {
    pub artist: Artist,
    pub play_count: i64,
    pub total_listening_time_sec: i64,
    pub tracks_played: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopAlbum {
    pub album: Album,
    pub play_count: i64,
    pub total_listening_time_sec: i64,
    pub tracks_played: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeSeriesPoint {
    pub date: String,
    pub value: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreakData {
    pub current_streak: i32,
    pub longest_streak: i32,
    pub streak_dates: Vec<String>,
    pub daily_counts: HashMap<String, i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GrowthPoint {
    pub period: String,
    pub tracks_added: i64,
    pub artists_added: i64,
    pub albums_added: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeatmapCell {
    pub label: String,
    pub value: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaybackEvent {
    pub played_at: DateTime<Utc>,
    pub track: Track,
    pub completion_percent: f64,
    pub source_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FavoriteTrend {
    pub period: String,
    pub top_track_id: Option<i64>,
    pub top_track_name: Option<String>,
    pub top_artist_id: Option<i64>,
    pub top_artist_name: Option<String>,
    pub top_album_id: Option<i64>,
    pub top_album_name: Option<String>,
}
