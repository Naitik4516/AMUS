import {
  getStatsOverview,
  getTopTracksWithStats,
  getTopArtistsWithStats,
  getTopAlbumsWithStats,
  getListeningTimeTrend,
  getStreakData,
  getLibraryGrowth,
  getFormatDistribution,
  getHeatmapHourly,
  getHeatmapWeekday,
  getFavoriteTrends,
  getPlaybackHistoryTimeline,
  getDataAge,
  type Timeframe,
  type StatsOverview,
  type TopTrack,
  type TopArtist,
  type TopAlbum,
  type TimeSeriesPoint,
  type StreakData,
  type GrowthPoint,
  type FormatStat,
  type HeatmapCell,
  type PlaybackEvent,
  type FavoriteTrend,
  type DataAge,
} from "$lib/commands.svelte";

class StatsState {
  timeframe = $state<Timeframe>("all_time");

  dataAge = $state<DataAge | null>(null);

  overview = $state<StatsOverview | null>(null);
  topTracks = $state<TopTrack[]>([]);
  topArtists = $state<TopArtist[]>([]);
  topAlbums = $state<TopAlbum[]>([]);
  listeningTrend = $state<TimeSeriesPoint[]>([]);
  streakData = $state<StreakData | null>(null);
  libraryGrowth = $state<GrowthPoint[]>([]);
  formatDist = $state<FormatStat[]>([]);
  heatmapHourly = $state<HeatmapCell[]>([]);
  heatmapWeekday = $state<HeatmapCell[]>([]);
  favoriteTrends = $state<FavoriteTrend[]>([]);
  playbackHistory = $state<PlaybackEvent[]>([]);

  loading = $state(false);
  error = $state<string | null>(null);

  availableTimeframes = $derived.by<Timeframe[]>(() => {
    const days = this.dataAge?.data_age_days ?? 0;
    const all: Timeframe[] = [
      "today",
      "this_week",
      "this_month",
      "last_3_months",
      "last_6_months",
      "last_year",
      "last_5_years",
      "all_time",
    ];
    if (days < 7) return all.slice(0, 2);
    if (days < 30) return all.slice(0, 3);
    if (days < 90) return all.slice(0, 4);
    if (days < 180) return all.slice(0, 5);
    if (days < 365) return all.slice(0, 6);
    if (days < 1825) return all.slice(0, 7);
    return all;
  });

  async loadAll() {
    this.loading = true;
    this.error = null;
    const tf = this.timeframe;

    try {
      const results = await Promise.allSettled([
        getStatsOverview(tf),
        getTopTracksWithStats(tf, 20),
        getTopArtistsWithStats(tf, 20),
        getTopAlbumsWithStats(tf, 20),
        getListeningTimeTrend(tf),
        getStreakData(tf),
        getLibraryGrowth(tf),
        getFormatDistribution(),
        getHeatmapHourly(tf),
        getHeatmapWeekday(tf),
        getFavoriteTrends(tf),
        getPlaybackHistoryTimeline(tf, 100),
      ]);

      if (results[0].status === "fulfilled") this.overview = results[0].value;
      if (results[1].status === "fulfilled") this.topTracks = results[1].value;
      if (results[2].status === "fulfilled") this.topArtists = results[2].value;
      if (results[3].status === "fulfilled") this.topAlbums = results[3].value;
      if (results[4].status === "fulfilled") this.listeningTrend = results[4].value;
      if (results[5].status === "fulfilled") this.streakData = results[5].value;
      if (results[6].status === "fulfilled") this.libraryGrowth = results[6].value;
      if (results[7].status === "fulfilled") this.formatDist = results[7].value;
      if (results[8].status === "fulfilled") this.heatmapHourly = results[8].value;
      if (results[9].status === "fulfilled") this.heatmapWeekday = results[9].value;
      if (results[10].status === "fulfilled") this.favoriteTrends = results[10].value;
      if (results[11].status === "fulfilled") this.playbackHistory = results[11].value;

      const rejected = results.filter((r) => r.status === "rejected");
      if (rejected.length > 0) {
        console.error("Stats fetch errors:", rejected);
        if (rejected.length === results.length) {
          this.error = "Failed to load statistics";
        }
      }
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Unknown error";
      console.error("Stats load error:", e);
    } finally {
      this.loading = false;
    }
  }

  async loadDataAge() {
    try {
      this.dataAge = await getDataAge();
    } catch (e) {
      console.error("Failed to load data age:", e);
    }
  }

  async setTimeframe(tf: Timeframe) {
    this.timeframe = tf;
    await this.loadAll();
  }
}

export const stats = new StatsState();
