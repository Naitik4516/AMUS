export interface Artist {
  id: number;
  name: string;
  profile_image?: string;
  banner_image?: string;
}

export interface Album {
  id: number;
  name: string;
  cover_art?: string;
  album_artist?: Artist[];
  year?: number;
}

export interface Playlist {
  id: number;
  name: string;
  cover_art?: string | null;
}

export interface Track {
  id: number;
  title: string;
  artists: Artist[];
  album: Album;
  duration_seconds: number;
  is_favorite: boolean;
  cover_art?: string;
  added_at: string;
  track_number?: number;
  playlist_ids: number[];
}

export interface TrackDetails extends Track {
  path: string;
  mtime: number;
  play_count: number;
  last_played_at?: string;
  skipped_count: number;
  last_skipped_at?: string;
  year: number;
}

export type SortBy = "title" | "artist" | "album" | "duration" | "recently_added";

export type RepeatMode = "OFF" | "ALL" | "ONE";

export type PlaybackSource =
  | { type: "Album"; id: number }
  | { type: "Playlist"; id: number }
  | { type: "Artist"; id: number }
  | { type: "Favorites" }
  | { type: "Direct" }
  | { type: "Queue" }
  | { type: "Other" };

export type Context =
  | { type: "Playlist"; id: number; name: string; coverArt: string | null | undefined }
  | { type: "Album"; id: number; name: string; coverArt: string | null }
  | {
      type: "Artist";
      id: number;
      name: string;
      profileImage: string | null;
      bannerImage: string | null;
    }
  | { type: "Favorites"; name: "Favorites" };

type MenuPosition =
  | { type: "anchor"; anchor: HTMLElement }
  | { type: "coordinates"; x: number; y: number };

// Stats types
export type Timeframe =
  | "today"
  | "this_week"
  | "this_month"
  | "last_3_months"
  | "last_6_months"
  | "last_year"
  | "last_5_years"
  | "all_time";

export interface DataAge {
  min_track_added_at: string | null;
  min_played_at: string | null;
  data_age_days: number;
}

export interface StatsOverview {
  total_tracks: number;
  total_artists: number;
  total_albums: number;
  total_plays: number;
  total_listening_time_sec: number;
  avg_daily_listening_min: number;
  total_file_size_bytes: number;
  avg_file_size_mb: number;
  largest_file_mb: number;
  format_distribution: FormatStat[];
  pct_library_played: number;
  unplayed_tracks: number;
}

export interface FormatStat {
  format: string;
  count: number;
  percentage: number;
  total_bytes: number;
}

export interface TopTrack {
  track: Track;
  play_count: number;
  total_listening_time_sec: number;
  last_played_at: string | null;
}

export interface TopArtist {
  artist: Artist;
  play_count: number;
  total_listening_time_sec: number;
  tracks_played: number;
}

export interface TopAlbum {
  album: Album;
  play_count: number;
  total_listening_time_sec: number;
  tracks_played: number;
}

export interface TimeSeriesPoint {
  date: string;
  value: number;
}

export interface StreakData {
  current_streak: number;
  longest_streak: number;
  streak_dates: string[];
  daily_counts: Record<string, number>;
}

export interface GrowthPoint {
  period: string;
  tracks_added: number;
  artists_added: number;
  albums_added: number;
}

export interface HeatmapCell {
  label: string;
  value: number;
}

export interface PlaybackEvent {
  played_at: string;
  track: Track;
  completion_percent: number;
  source_type: string;
}

export interface FavoriteTrend {
  period: string;
  top_track_id: number | null;
  top_track_name: string | null;
  top_artist_id: number | null;
  top_artist_name: string | null;
  top_album_id: number | null;
  top_album_name: string | null;
}
