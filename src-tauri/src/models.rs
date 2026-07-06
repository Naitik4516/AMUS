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
    pub track_number: Option<u32>,
    pub year: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Track {
    pub id: i64,
    pub title: String,
    pub artists: Vec<Artist>,
    pub album: Album,
    pub duration_seconds: u32,
    pub is_favorite: bool,
    pub cover_art: Option<String>,
    pub added_at: DateTime<Utc>,
    pub track_number: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaybackState {
    pub current_track_id: Option<i64>,
    pub position_ms: u32,
    pub shuffle_enabled: bool,
    pub repeat_mode: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Artist {
    pub id: i64,
    pub name: String,
    pub profile_image: Option<String>,
    pub banner_image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Album {
    pub id: i64,
    pub name: String,
    pub cover_art: Option<String>,
    pub album_artist: Option<Vec<Artist>>,
    pub year: Option<u32>,
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
pub struct GlobalSearchResult {
    pub result_type: String,
    pub score: i32,
    pub track: Option<Track>,
    pub artist: Option<Artist>,
    pub album: Option<Album>,
    pub playlist: Option<Playlist>,
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
