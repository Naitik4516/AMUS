import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("$lib/commands.svelte", () => ({
  getStatsOverview: vi.fn(),
  getTopTracksWithStats: vi.fn(),
  getTopArtistsWithStats: vi.fn(),
  getTopAlbumsWithStats: vi.fn(),
  getTopGenresWithStats: vi.fn(),
  getListeningTimeTrend: vi.fn(),
  getStreakData: vi.fn(),
  getLibraryGrowth: vi.fn(),
  getFormatDistribution: vi.fn(),
  getHeatmapHourly: vi.fn(),
  getHeatmapWeekday: vi.fn(),
  getFavoriteTrends: vi.fn(),
  getPlaybackHistoryTimeline: vi.fn(),
  getDataAge: vi.fn(),
}));

const cmds = await import("$lib/commands.svelte");
const statsMod = await import("./stats.svelte");
const m = cmds as unknown as Record<string, ReturnType<typeof vi.fn>>;

function mockAllFulfilled() {
  m.getStatsOverview.mockResolvedValue({ total_plays: 100, total_listening_time_sec: 7200 });
  m.getTopTracksWithStats.mockResolvedValue([]);
  m.getTopArtistsWithStats.mockResolvedValue([]);
  m.getTopAlbumsWithStats.mockResolvedValue([]);
  m.getTopGenresWithStats.mockResolvedValue([]);
  m.getListeningTimeTrend.mockResolvedValue([]);
  m.getStreakData.mockResolvedValue({
    current_streak: 5,
    longest_streak: 30,
    streak_dates: [],
    daily_counts: {},
  });
  m.getLibraryGrowth.mockResolvedValue([]);
  m.getFormatDistribution.mockResolvedValue([]);
  m.getHeatmapHourly.mockResolvedValue([]);
  m.getHeatmapWeekday.mockResolvedValue([]);
  m.getFavoriteTrends.mockResolvedValue([]);
  m.getPlaybackHistoryTimeline.mockResolvedValue([]);
}

beforeEach(() => {
  vi.clearAllMocks();
  statsMod.stats.reset();
});

describe("StatsState", () => {
  describe("availableTimeframes", () => {
    it("returns all when data age > 5 years", () => {
      statsMod.stats.dataAge = {
        min_track_added_at: null,
        min_played_at: null,
        data_age_days: 2000,
      };
      expect(statsMod.stats.availableTimeframes).toHaveLength(8);
    });

    it("returns [today, this_week] plus all_time when data age < 7 days", () => {
      statsMod.stats.dataAge = {
        min_track_added_at: null,
        min_played_at: null,
        data_age_days: 3,
      };
      expect(statsMod.stats.availableTimeframes).toEqual(["today", "this_week", "all_time"]);
    });

    it("includes this_month when data age < 30 days", () => {
      statsMod.stats.dataAge = {
        min_track_added_at: null,
        min_played_at: null,
        data_age_days: 15,
      };
      expect(statsMod.stats.availableTimeframes).toEqual([
        "today",
        "this_week",
        "this_month",
        "all_time",
      ]);
    });

    it("returns [today, this_week, all_time] when data age is 0", () => {
      statsMod.stats.dataAge = {
        min_track_added_at: null,
        min_played_at: null,
        data_age_days: 0,
      };
      expect(statsMod.stats.availableTimeframes).toEqual(["today", "this_week", "all_time"]);
    });

    it("keeps all_time last even when fully available", () => {
      statsMod.stats.dataAge = {
        min_track_added_at: null,
        min_played_at: null,
        data_age_days: 2000,
      };
      expect(statsMod.stats.availableTimeframes).toEqual([
        "today",
        "this_week",
        "this_month",
        "last_3_months",
        "last_6_months",
        "last_year",
        "last_5_years",
        "all_time",
      ]);
    });
  });

  describe("loadAll", () => {
    it("calls all fetchers with current timeframe", async () => {
      mockAllFulfilled();
      statsMod.stats.timeframe = "last_year";
      await statsMod.stats.loadAll();

      expect(m.getStatsOverview).toHaveBeenCalledWith("last_year");
      expect(m.getTopTracksWithStats).toHaveBeenCalledWith("last_year", 20, "plays");
      expect(m.getTopArtistsWithStats).toHaveBeenCalledWith("last_year", 20, "plays");
      expect(m.getTopAlbumsWithStats).toHaveBeenCalledWith("last_year", 20, "plays");
      expect(m.getTopGenresWithStats).toHaveBeenCalledWith("last_year", 20, "plays");
      expect(m.getListeningTimeTrend).toHaveBeenCalledWith("last_year");
      expect(m.getStreakData).toHaveBeenCalledWith("last_year");
      expect(m.getLibraryGrowth).toHaveBeenCalledWith();
      expect(m.getFormatDistribution).toHaveBeenCalledWith();
      expect(m.getHeatmapHourly).toHaveBeenCalledWith("last_year");
      expect(m.getHeatmapWeekday).toHaveBeenCalledWith("last_year");
      expect(m.getFavoriteTrends).toHaveBeenCalledWith("last_year");
      expect(m.getPlaybackHistoryTimeline).toHaveBeenCalledWith("last_year", 100);
    });

    it("skips streak and favorite trends for today", async () => {
      mockAllFulfilled();
      statsMod.stats.timeframe = "today";
      await statsMod.stats.loadAll();

      expect(m.getStreakData).not.toHaveBeenCalled();
      expect(m.getFavoriteTrends).not.toHaveBeenCalled();
      expect(m.getPlaybackHistoryTimeline).toHaveBeenCalledWith("today", 100);
    });

    it("skips favorite trends for this_week and this_month", async () => {
      mockAllFulfilled();
      statsMod.stats.timeframe = "this_week";
      await statsMod.stats.loadAll();
      expect(m.getFavoriteTrends).not.toHaveBeenCalled();
      statsMod.stats.timeframe = "this_month";
      await statsMod.stats.loadAll();
      expect(m.getFavoriteTrends).not.toHaveBeenCalled();
    });

    it("skips timeline for all_time", async () => {
      mockAllFulfilled();
      statsMod.stats.timeframe = "all_time";
      await statsMod.stats.loadAll();

      expect(m.getPlaybackHistoryTimeline).not.toHaveBeenCalled();
      expect(m.getStreakData).toHaveBeenCalledWith("all_time");
      expect(m.getFavoriteTrends).toHaveBeenCalledWith("all_time");
    });

    it("fetches library growth and format distribution only once", async () => {
      mockAllFulfilled();
      await statsMod.stats.loadAll();
      await statsMod.stats.loadAll();

      expect(m.getLibraryGrowth).toHaveBeenCalledTimes(1);
      expect(m.getFormatDistribution).toHaveBeenCalledTimes(1);
    });

    it("populates state on full success", async () => {
      mockAllFulfilled();
      m.getStatsOverview.mockResolvedValue({ total_plays: 50, total_listening_time_sec: 3600 });
      m.getTopTracksWithStats.mockResolvedValue([{ track_id: 1, play_count: 10 } as any]);
      m.getTopGenresWithStats.mockResolvedValue([{ genre_id: 1, play_count: 5 } as any]);
      m.getStreakData.mockResolvedValue({ current_streak: 3, longest_streak: 10 } as any);

      await statsMod.stats.loadAll();

      expect(statsMod.stats.loading).toBe(false);
      expect(statsMod.stats.overview).toEqual({
        total_plays: 50,
        total_listening_time_sec: 3600,
      });
      expect(statsMod.stats.topTracks).toHaveLength(1);
      expect(statsMod.stats.topGenres).toHaveLength(1);
      expect(statsMod.stats.streakData).toEqual({ current_streak: 3, longest_streak: 10 });
    });

    it("handles partial failure without setting error", async () => {
      mockAllFulfilled();
      m.getStatsOverview.mockResolvedValue({ total_plays: 10, total_listening_time_sec: 600 });
      m.getTopTracksWithStats.mockRejectedValue(new Error("tracks fail"));

      await statsMod.stats.loadAll();

      expect(statsMod.stats.overview).toEqual({
        total_plays: 10,
        total_listening_time_sec: 600,
      });
      expect(statsMod.stats.topTracks).toEqual([]);
      expect(statsMod.stats.error).toBeNull();
    });

    it("sets error when ALL requests fail", async () => {
      mockAllFulfilled();
      for (const key of Object.keys(m)) {
        (m as any)[key]?.mockRejectedValue?.(new Error("fail"));
      }

      await statsMod.stats.loadAll();

      expect(statsMod.stats.error).toBe("Failed to load statistics");
    });

    it("sets loading flag during fetch", async () => {
      mockAllFulfilled();
      let resolvePromise!: () => void;
      m.getStatsOverview.mockReturnValue(
        new Promise<void>((r) => {
          resolvePromise = r;
        }),
      );

      const loadPromise = statsMod.stats.loadAll();
      expect(statsMod.stats.loading).toBe(true);
      resolvePromise();
      await loadPromise;
      expect(statsMod.stats.loading).toBe(false);
    });
  });

  describe("setTimeframe", () => {
    it("updates timeframe and reloads", async () => {
      mockAllFulfilled();
      await statsMod.stats.setTimeframe("this_week");
      expect(statsMod.stats.timeframe).toBe("this_week");
      expect(m.getStatsOverview).toHaveBeenCalledWith("this_week");
    });

    it("clamps to all_time when selection is unavailable", async () => {
      mockAllFulfilled();
      statsMod.stats.dataAge = {
        min_track_added_at: null,
        min_played_at: null,
        data_age_days: 60,
      };
      await statsMod.stats.setTimeframe("last_6_months");
      expect(statsMod.stats.timeframe).toBe("all_time");
    });

    it("allows all_time even with little data", async () => {
      mockAllFulfilled();
      statsMod.stats.dataAge = {
        min_track_added_at: null,
        min_played_at: null,
        data_age_days: 60,
      };
      statsMod.stats.timeframe = "this_week";
      await statsMod.stats.setTimeframe("all_time");
      expect(statsMod.stats.timeframe).toBe("all_time");
      expect(m.getStatsOverview).toHaveBeenCalledWith("all_time");
    });

    it("clears stale data before reloading", async () => {
      mockAllFulfilled();
      m.getStatsOverview.mockReturnValue(
        new Promise<void>(() => {
          // never resolves
        }),
      );
      statsMod.stats.setTimeframe("this_week");
      expect(statsMod.stats.overview).toBeNull();
    });
  });

  describe("setSortBy", () => {
    it("refetches only the selected section with the new sort", async () => {
      mockAllFulfilled();
      statsMod.stats.timeframe = "last_year";
      await statsMod.stats.loadAll();
      m.getStatsOverview.mockClear();
      m.getTopArtistsWithStats.mockClear();
      m.getTopAlbumsWithStats.mockClear();
      m.getTopGenresWithStats.mockClear();

      await statsMod.stats.setSortBy("tracks", "time");

      expect(statsMod.stats.tracksSortBy).toBe("time");
      expect(statsMod.stats.artistsSortBy).toBe("plays");
      expect(m.getTopTracksWithStats).toHaveBeenCalledWith("last_year", 20, "time");
      expect(m.getTopArtistsWithStats).not.toHaveBeenCalled();
      expect(m.getTopAlbumsWithStats).not.toHaveBeenCalled();
      expect(m.getTopGenresWithStats).not.toHaveBeenCalled();
      expect(m.getStatsOverview).not.toHaveBeenCalled();
    });

    it("keeps sort independent per section", async () => {
      mockAllFulfilled();
      statsMod.stats.timeframe = "this_month";
      await statsMod.stats.setSortBy("artists", "time");
      expect(statsMod.stats.artistsSortBy).toBe("time");
      expect(statsMod.stats.tracksSortBy).toBe("plays");

      await statsMod.stats.loadAll();
      // loadAll uses each section's own preference
      expect(m.getTopArtistsWithStats).toHaveBeenCalledWith("this_month", 20, "time");
      expect(m.getTopTracksWithStats).toHaveBeenCalledWith("this_month", 20, "plays");
      expect(m.getTopGenresWithStats).toHaveBeenCalledWith("this_month", 20, "plays");
    });

    it("does not refetch when sort is unchanged", async () => {
      mockAllFulfilled();
      statsMod.stats.timeframe = "last_year";
      await statsMod.stats.loadAll();
      m.getTopTracksWithStats.mockClear();
      await statsMod.stats.setSortBy("tracks", "plays");
      expect(m.getTopTracksWithStats).not.toHaveBeenCalled();
    });
  });

  describe("loadMoreHistory", () => {
    it("increases limit and refetches timeline", async () => {
      mockAllFulfilled();
      statsMod.stats.timeframe = "this_month";
      m.getPlaybackHistoryTimeline.mockResolvedValue([{ played_at: "x" } as any]);
      await statsMod.stats.loadMoreHistory();

      expect(m.getPlaybackHistoryTimeline).toHaveBeenCalledWith("this_month", 200);
      expect(statsMod.stats.playbackHistory).toHaveLength(1);
      expect(statsMod.stats.historyLimit).toBe(200);
    });
  });

  describe("loadDataAge", () => {
    it("stores data age on success", async () => {
      m.getDataAge.mockResolvedValue({ data_age_days: 100 });
      await statsMod.stats.loadDataAge();
      expect(statsMod.stats.dataAge).toEqual({ data_age_days: 100 });
    });

    it("does not set error on failure", async () => {
      m.getDataAge.mockRejectedValue(new Error("fail"));
      await statsMod.stats.loadDataAge();
      expect(statsMod.stats.dataAge).toBeNull();
      expect(statsMod.stats.error).toBeNull();
    });
  });
});
