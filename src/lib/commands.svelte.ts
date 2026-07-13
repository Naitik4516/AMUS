import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  Timeframe,
  DataAge,
  StatsOverview,
  FormatStat,
  TopTrack,
  TopArtist,
  TopAlbum,
  TimeSeriesPoint,
  StreakData,
  GrowthPoint,
  HeatmapCell,
  PlaybackEvent,
  FavoriteTrend,
} from "./types.d.ts";

export type {
  Timeframe,
  DataAge,
  StatsOverview,
  FormatStat,
  TopTrack,
  TopArtist,
  TopAlbum,
  TimeSeriesPoint,
  StreakData,
  GrowthPoint,
  HeatmapCell,
  PlaybackEvent,
  FavoriteTrend,
};

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

export async function getStatsOverview(timeframe: Timeframe): Promise<StatsOverview> {
  return invoke("get_stats_overview", { timeframe });
}

export async function getTopTracksWithStats(
  timeframe: Timeframe,
  limit: number,
): Promise<TopTrack[]> {
  return invoke("get_top_tracks_with_stats", { timeframe, limit });
}

export async function getTopArtistsWithStats(
  timeframe: Timeframe,
  limit: number,
): Promise<TopArtist[]> {
  return invoke("get_top_artists_with_stats", { timeframe, limit });
}

export async function getTopAlbumsWithStats(
  timeframe: Timeframe,
  limit: number,
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

export async function getDataAge(): Promise<DataAge> {
  return invoke("get_data_age");
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
  limit: number,
): Promise<PlaybackEvent[]> {
  return invoke("get_playback_history_timeline", { timeframe, limit });
}
