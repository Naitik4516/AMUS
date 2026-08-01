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
  BlacklistedEntry,
  Genre,
  Track,
  TrackDetails,
  Lyrics,
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
    const results = await Promise.allSettled(
      selected.map((path) => invoke("add_source", { path })),
    );
    const failures = results.filter((r) => r.status === "rejected");
    if (failures.length > 0) {
      console.warn(`${failures.length} source path(s) failed to add`);
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

export async function setOsMediaControls(enabled: boolean): Promise<void> {
  return invoke("set_os_media_controls", { enabled });
}

export async function deleteTrack(id: number): Promise<void> {
  await invoke("delete_track", { id });
}

export async function getScanBlacklist(): Promise<BlacklistedEntry[]> {
  return invoke("get_scan_blacklist");
}

export async function unblacklistPath(path: string): Promise<void> {
  await invoke("unblacklist_path", { path });
}

// ---------------------------------------------------------------------------
// Lyrics commands
// ---------------------------------------------------------------------------

export async function getTrackLyrics(id: number): Promise<Lyrics | null> {
  return invoke("get_track_lyrics", { id });
}

export async function updateTrackLyrics(
  trackId: number,
  plainLyrics: string | null,
  syncedLyrics: string | null,
  source: string,
): Promise<void> {
  await invoke("update_track_lyrics", {
    trackId,
    plainLyrics,
    syncedLyrics,
    source,
  });
}

export async function fetchLyricsFromLrclib(trackId: number): Promise<boolean> {
  return invoke("fetch_lyrics_from_lrclib", { trackId });
}

// ---------------------------------------------------------------------------
// Genre commands
// ---------------------------------------------------------------------------

export async function getGenres(): Promise<Genre[]> {
  return invoke("get_genres");
}

export async function getTracksByGenre(genreId: number): Promise<Track[]> {
  return invoke("get_tracks_by_genre", { genreId });
}

export async function updateGenre(
  id: number,
  name: string,
  thumbnail?: string | null,
): Promise<Genre> {
  return invoke("update_genre", { id, name, thumbnail });
}

export async function createGenre(name: string): Promise<Genre> {
  return invoke("create_genre", { name });
}

export async function setTrackGenre(trackId: number, genreName: string): Promise<void> {
  await invoke("set_track_genre", { trackId, genreName });
}

// ---------------------------------------------------------------------------
// Track editing commands
// ---------------------------------------------------------------------------

export async function setTrackCoverArt(trackId: number, coverArt: string | null): Promise<void> {
  await invoke("set_track_cover_art", { trackId, coverArt });
}

export async function setTrackArtists(trackId: number, artistIds: number[]): Promise<void> {
  await invoke("set_track_artists", { trackId, artistIds });
}

export async function setTrackAlbum(trackId: number, albumId: number): Promise<void> {
  await invoke("set_track_album", { trackId, albumId });
}
