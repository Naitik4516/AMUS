import {
  getStatsOverview,
  getTopTracksWithStats,
  getTopArtistsWithStats,
  getTopAlbumsWithStats,
  getTopGenresWithStats,
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
  type TopSort,
  type StatsOverview,
  type TopTrack,
  type TopArtist,
  type TopAlbum,
  type TopGenre,
  type TimeSeriesPoint,
  type StreakData,
  type GrowthPoint,
  type FormatStat,
  type HeatmapCell,
  type PlaybackEvent,
  type FavoriteTrend,
  type DataAge,
} from "$lib/commands.svelte";

const ALL_TIMEFRAMES: Timeframe[] = [
  "today",
  "this_week",
  "this_month",
  "last_3_months",
  "last_6_months",
  "last_year",
  "last_5_years",
  "all_time",
];

const STREAK_HIDDEN: Timeframe[] = ["today"];
const FAVORITE_TREND_HIDDEN: Timeframe[] = ["today", "this_week", "this_month"];
const TIMELINE_HIDDEN: Timeframe[] = ["last_5_years", "all_time"];

const HISTORY_PAGE_SIZE = 100;

type TopSection = "tracks" | "artists" | "albums" | "genres";

class StatsState {
  timeframe = $state<Timeframe>("all_time");
  tracksSortBy = $state<TopSort>("plays");
  artistsSortBy = $state<TopSort>("plays");
  albumsSortBy = $state<TopSort>("plays");
  genresSortBy = $state<TopSort>("plays");
  historyLimit = $state(HISTORY_PAGE_SIZE);

  dataAge = $state<DataAge | null>(null);

  overview = $state<StatsOverview | null>(null);
  topTracks = $state<TopTrack[]>([]);
  topArtists = $state<TopArtist[]>([]);
  topAlbums = $state<TopAlbum[]>([]);
  topGenres = $state<TopGenre[]>([]);
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
    const count =
      days < 7
        ? 2
        : days < 30
          ? 3
          : days < 90
            ? 4
            : days < 180
              ? 5
              : days < 365
                ? 6
                : days < 1825
                  ? 7
                  : 8;
    const available = ALL_TIMEFRAMES.slice(0, count);
    // "All" is always selectable, regardless of data age.
    if (!available.includes("all_time")) available.push("all_time");
    return available;
  });

  #loadGeneration = 0;
  #growthLoaded = false;
  #formatLoaded = false;

  #showStreak() {
    return !STREAK_HIDDEN.includes(this.timeframe);
  }

  #showFavoriteTrends() {
    return !FAVORITE_TREND_HIDDEN.includes(this.timeframe);
  }

  #showTimeline() {
    return !TIMELINE_HIDDEN.includes(this.timeframe);
  }

  async loadAll() {
    this.loading = true;
    this.error = null;
    const tf = this.timeframe;
    const generation = ++this.#loadGeneration;

    const needsGrowth = !this.#growthLoaded;
    const needsFormat = !this.#formatLoaded;

    const calls: Promise<unknown>[] = [
      getStatsOverview(tf),
      getTopTracksWithStats(tf, 20, this.tracksSortBy),
      getTopArtistsWithStats(tf, 20, this.artistsSortBy),
      getTopAlbumsWithStats(tf, 20, this.albumsSortBy),
      getTopGenresWithStats(tf, 20, this.genresSortBy),
      getListeningTimeTrend(tf),
      getHeatmapHourly(tf),
      getHeatmapWeekday(tf),
    ];
    if (this.#showStreak()) calls.push(getStreakData(tf));
    if (this.#showFavoriteTrends()) calls.push(getFavoriteTrends(tf));
    if (this.#showTimeline()) calls.push(getPlaybackHistoryTimeline(tf, this.historyLimit));
    if (needsGrowth) calls.push(getLibraryGrowth());
    if (needsFormat) calls.push(getFormatDistribution());

    try {
      const results = await Promise.allSettled(calls);

      if (generation !== this.#loadGeneration) return;

      let i = 0;
      const set = (target: (v: unknown) => void) => {
        const r = results[i++];
        if (r.status === "fulfilled") target(r.value);
      };

      set((v) => (this.overview = v as StatsOverview));
      set((v) => (this.topTracks = v as TopTrack[]));
      set((v) => (this.topArtists = v as TopArtist[]));
      set((v) => (this.topAlbums = v as TopAlbum[]));
      set((v) => (this.topGenres = v as TopGenre[]));
      set((v) => (this.listeningTrend = v as TimeSeriesPoint[]));
      set((v) => (this.heatmapHourly = v as HeatmapCell[]));
      set((v) => (this.heatmapWeekday = v as HeatmapCell[]));
      if (this.#showStreak()) set((v) => (this.streakData = v as StreakData));
      if (this.#showFavoriteTrends()) set((v) => (this.favoriteTrends = v as FavoriteTrend[]));
      if (this.#showTimeline()) set((v) => (this.playbackHistory = v as PlaybackEvent[]));
      if (needsGrowth) {
        set((v) => (this.libraryGrowth = v as GrowthPoint[]));
        this.#growthLoaded = true;
      }
      if (needsFormat) {
        set((v) => (this.formatDist = v as FormatStat[]));
        this.#formatLoaded = true;
      }

      const rejected = results.filter((r) => r.status === "rejected");
      if (rejected.length > 0) {
        console.error("Stats fetch errors:", rejected);
        if (rejected.length === results.length) {
          this.error = "Failed to load statistics";
        }
      }
    } catch (e) {
      if (generation !== this.#loadGeneration) return;
      this.error = e instanceof Error ? e.message : "Unknown error";
      console.error("Stats load error:", e);
    } finally {
      if (generation === this.#loadGeneration) {
        this.loading = false;
      }
    }
  }

  async loadDataAge() {
    try {
      this.dataAge = await getDataAge();
      const available = this.availableTimeframes;
      if (!available.includes(this.timeframe)) {
        this.timeframe = available[available.length - 1];
        await this.loadAll();
      }
    } catch (e) {
      console.error("Failed to load data age:", e);
    }
  }

  async setTimeframe(tf: Timeframe) {
    const available = this.availableTimeframes;
    if (!available.includes(tf)) tf = available[available.length - 1];
    if (tf === this.timeframe) return;
    this.timeframe = tf;
    this.#resetTimeframeData();
    await this.loadAll();
  }

  async setSortBy(section: TopSection, sort: TopSort) {
    const sortKey = `${section}SortBy` as const;
    if (this[sortKey] === sort) return;
    this[sortKey] = sort;
    const tf = this.timeframe;
    const generation = ++this.#loadGeneration;
    const fetch: Record<
      TopSection,
      (tf: Timeframe, limit: number, sort: TopSort) => Promise<unknown>
    > = {
      tracks: getTopTracksWithStats,
      artists: getTopArtistsWithStats,
      albums: getTopAlbumsWithStats,
      genres: getTopGenresWithStats,
    };
    const assign: Record<TopSection, (v: unknown) => void> = {
      tracks: (v) => (this.topTracks = v as TopTrack[]),
      artists: (v) => (this.topArtists = v as TopArtist[]),
      albums: (v) => (this.topAlbums = v as TopAlbum[]),
      genres: (v) => (this.topGenres = v as TopGenre[]),
    };
    try {
      const result = await fetch[section](tf, 20, sort);
      if (generation !== this.#loadGeneration) return;
      assign[section](result);
    } catch (e) {
      if (generation !== this.#loadGeneration) return;
      console.error(`Failed to sort ${section}:`, e);
    } finally {
      if (generation === this.#loadGeneration) {
        this.loading = false;
      }
    }
  }

  async loadMoreHistory() {
    if (!this.#showTimeline()) return;
    this.historyLimit += HISTORY_PAGE_SIZE;
    const tf = this.timeframe;
    try {
      const events = await getPlaybackHistoryTimeline(tf, this.historyLimit);
      if (this.timeframe === tf) {
        this.playbackHistory = events;
      }
    } catch (e) {
      console.error("Failed to load more history:", e);
    }
  }

  #resetTimeframeData() {
    this.overview = null;
    this.topTracks = [];
    this.topArtists = [];
    this.topAlbums = [];
    this.topGenres = [];
    this.listeningTrend = [];
    this.streakData = null;
    this.heatmapHourly = [];
    this.heatmapWeekday = [];
    this.favoriteTrends = [];
    this.playbackHistory = [];
  }

  /** @internal Resets all state and cached flags (used by tests). */
  reset() {
    this.#loadGeneration++;
    this.#growthLoaded = false;
    this.#formatLoaded = false;
    this.timeframe = "all_time";
    this.tracksSortBy = "plays";
    this.artistsSortBy = "plays";
    this.albumsSortBy = "plays";
    this.genresSortBy = "plays";
    this.historyLimit = HISTORY_PAGE_SIZE;
    this.dataAge = null;
    this.overview = null;
    this.topTracks = [];
    this.topArtists = [];
    this.topAlbums = [];
    this.topGenres = [];
    this.listeningTrend = [];
    this.streakData = null;
    this.libraryGrowth = [];
    this.formatDist = [];
    this.heatmapHourly = [];
    this.heatmapWeekday = [];
    this.favoriteTrends = [];
    this.playbackHistory = [];
    this.loading = false;
    this.error = null;
  }
}

export const stats = new StatsState();
