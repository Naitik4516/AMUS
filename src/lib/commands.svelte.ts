import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { Track, Artist, Album } from "./types.d.ts";

export async function importAudioLibrary() {
  const selected = await open({
    directory: true,
    multiple: true,
    title: "Select Audio Library Folder",
  });

  if (selected) {
    for (const path of selected) {
      await invoke("add_source", { path });
    }
  }
  await invoke("scan_library");
}

export async function getSourceDirs(): Promise<string[]> {
  return invoke("get_source_dirs");
}

export async function removeSource(path: string): Promise<void> {
  await invoke("remove_source", { path });
}

export async function scanLibrary(): Promise<void> {
  await invoke("scan_library");
}

export async function refreshWatcher(): Promise<void> {
  await invoke("refresh_watcher");
}

// ---------------------------------------------------------------------------
// Stats commands
// ---------------------------------------------------------------------------

export type Timeframe =
  | "today"
  | "this_week"
  | "this_month"
  | "last_3_months"
  | "last_6_months"
  | "this_year"
  | "last_year"
  | "last_5_years"
  | "all_time";

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

export async function getStatsOverview(timeframe: Timeframe): Promise<StatsOverview> {
  return invoke("get_stats_overview", { timeframe });
}

export async function getTopTracksWithStats(
  timeframe: Timeframe,
  limit: number
): Promise<TopTrack[]> {
  return invoke("get_top_tracks_with_stats", { timeframe, limit });
}

export async function getTopArtistsWithStats(
  timeframe: Timeframe,
  limit: number
): Promise<TopArtist[]> {
  return invoke("get_top_artists_with_stats", { timeframe, limit });
}

export async function getTopAlbumsWithStats(
  timeframe: Timeframe,
  limit: number
): Promise<TopAlbum[]> {
  return invoke("get_top_albums_with_stats", { timeframe, limit });
}

export async function getListeningTimeTrend(timeframe: Timeframe): Promise<TimeSeriesPoint[]> {
  return invoke("get_listening_time_trend", { timeframe });
}

export async function getStreakData(timeframe: Timeframe): Promise<StreakData> {
  return invoke("get_streak_data", { timeframe });
}

export async function getLibraryGrowth(timeframe: Timeframe): Promise<GrowthPoint[]> {
  return invoke("get_library_growth", { timeframe });
}

export async function getFormatDistribution(): Promise<FormatStat[]> {
  return invoke("get_format_distribution");
}

export async function getHeatmapHourly(timeframe: Timeframe): Promise<HeatmapCell[]> {
  return invoke("get_heatmap_hourly", { timeframe });
}

export async function getHeatmapWeekday(timeframe: Timeframe): Promise<HeatmapCell[]> {
  return invoke("get_heatmap_weekday", { timeframe });
}

export async function getFavoriteTrends(timeframe: Timeframe): Promise<FavoriteTrend[]> {
  return invoke("get_favorite_trends", { timeframe });
}

export async function getPlaybackHistoryTimeline(
  timeframe: Timeframe,
  limit: number
): Promise<PlaybackEvent[]> {
  return invoke("get_playback_history_timeline", { timeframe, limit });
}
