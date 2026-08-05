export type WithRest<T> = T & Record<string, unknown>;

export interface Artist {
  id: number;
  name: string;
  profile_image?: string;
  banner_image?: string;
  track_count?: number;
  total_plays?: number;
  last_played_at?: string;
  added_at?: string;
}

export interface Album {
  id: number;
  name: string;
  cover_art?: string;
  album_artist?: Artist[];
  year?: number;
  track_count?: number;
  total_plays?: number;
  last_played_at?: string;
  added_at?: string;
}

export interface Playlist {
  id: number;
  name: string;
  cover_art?: string | null;
  track_count?: number;
  total_plays?: number;
  last_played_at?: string;
  added_at?: string;
}

export interface Genre {
  id: number;
  name: string;
  thumbnail?: string;
  track_count?: number;
  total_plays?: number;
  last_played_at?: string;
  added_at?: string;
}

export interface Lyrics {
  plain_lyrics?: string;
  synced_lyrics?: string;
  source: string;
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
  genre_ids?: number[];
  queue_id?: number;
}

export interface TrackDetails extends Track {
  path: string;
  mtime: number;
  play_count: number;
  last_played_at?: string;
  skipped_count: number;
  last_skipped_at?: string;
  year: number;
  genres: Genre[];
  bitrate?: number;
  sample_rate: number;
  bit_depth?: number;
  channels: number;
  audio_format: string;
  codec?: string;
  bpm?: number;
  replaygain_track_gain?: number;
  replaygain_track_peak?: number;
  replaygain_album_gain?: number;
  replaygain_album_peak?: number;
  encoder?: string;
  lyrics?: Lyrics;
}

export type SortBy = "title" | "artist" | "album" | "duration" | "recently_added";

export type RepeatMode = "OFF" | "ALL" | "ONE";

export type PlaybackSource =
  | { type: "Album"; id: number }
  | { type: "Playlist"; id: number }
  | { type: "Artist"; id: number }
  | { type: "Genre"; id: number }
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
      profileImage: string | null | undefined;
      bannerImage: string | null | undefined;
    }
  | { type: "Favorites"; name: "Favorites" }
  | { type: "Genre"; id: number; name: string; thumbnail: string | null }
  | null;

type MenuPosition =
  | { type: "anchor"; anchor: HTMLElement }
  | { type: "coordinates"; x: number; y: number };

// Stats types
export interface BlacklistedEntry {
  path: string;
  mtime: number;
  reason: string;
  created_at: string;
}

export type Timeframe =
  | "today"
  | "this_week"
  | "this_month"
  | "last_3_months"
  | "last_6_months"
  | "last_year"
  | "last_5_years"
  | "all_time";

export type TopSort = "plays" | "time";

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
  avg_bitrate_kbps: number | null;
  avg_sample_rate: number | null;
  avg_bit_depth: number | null;
  avg_completion_pct: number | null;
  skip_rate: number | null;
}

export interface FormatStat {
  format: string;
  count: number;
  percentage: number;
  total_bytes: number;
  avg_bitrate_kbps: number | null;
  avg_sample_rate: number | null;
  avg_bit_depth: number | null;
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

export interface TopGenre {
  genre: Genre;
  play_count: number;
  total_listening_time_sec: number;
  tracks_played: number;
}

export type TopRankItem = TopTrack | TopArtist | TopAlbum | TopGenre;

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
