import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("$lib/commands.svelte", () => ({
  getStatsOverview: vi.fn(),
  getTopTracksWithStats: vi.fn(),
  getTopArtistsWithStats: vi.fn(),
  getTopAlbumsWithStats: vi.fn(),
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
  m.getStatsOverview.mockResolvedValue({ total_plays: 100, total_listening_time_seconds: 7200 });
  m.getTopTracksWithStats.mockResolvedValue([]);
  m.getTopArtistsWithStats.mockResolvedValue([]);
  m.getTopAlbumsWithStats.mockResolvedValue([]);
  m.getListeningTimeTrend.mockResolvedValue([]);
  m.getStreakData.mockResolvedValue({ current_streak_days: 5, longest_streak_days: 30 });
  m.getLibraryGrowth.mockResolvedValue([]);
  m.getFormatDistribution.mockResolvedValue([]);
  m.getHeatmapHourly.mockResolvedValue([]);
  m.getHeatmapWeekday.mockResolvedValue([]);
  m.getFavoriteTrends.mockResolvedValue([]);
  m.getPlaybackHistoryTimeline.mockResolvedValue([]);
}

beforeEach(() => {
  vi.clearAllMocks();
  statsMod.stats.overview = null;
  statsMod.stats.topTracks = [];
  statsMod.stats.error = null;
  statsMod.stats.loading = false;
  statsMod.stats.dataAge = null;
  statsMod.stats.timeframe = "all_time";
});

describe("StatsState", () => {
  describe("availableTimeframes", () => {
    it("returns all when data age > 5 years", () => {
      statsMod.stats.dataAge = { data_age_days: 2000 };
      expect(statsMod.stats.availableTimeframes).toHaveLength(8);
    });

    it("returns [today, this_week] when data age < 7 days", () => {
      statsMod.stats.dataAge = { data_age_days: 3 };
      expect(statsMod.stats.availableTimeframes).toEqual(["today", "this_week"]);
    });

    it("includes this_month when data age < 30 days", () => {
      statsMod.stats.dataAge = { data_age_days: 15 };
      expect(statsMod.stats.availableTimeframes).toEqual(["today", "this_week", "this_month"]);
    });

    it("returns [today, this_week] when data age is 0", () => {
      statsMod.stats.dataAge = { data_age_days: 0 };
      expect(statsMod.stats.availableTimeframes).toEqual(["today", "this_week"]);
    });
  });

  describe("loadAll", () => {
    it("calls all 12 fetchers with current timeframe", async () => {
      mockAllFulfilled();
      statsMod.stats.timeframe = "this_month";
      await statsMod.stats.loadAll();

      expect(m.getStatsOverview).toHaveBeenCalledWith("this_month");
      expect(m.getTopTracksWithStats).toHaveBeenCalledWith("this_month", 20);
      expect(m.getTopArtistsWithStats).toHaveBeenCalledWith("this_month", 20);
      expect(m.getTopAlbumsWithStats).toHaveBeenCalledWith("this_month", 20);
      expect(m.getListeningTimeTrend).toHaveBeenCalledWith("this_month");
      expect(m.getStreakData).toHaveBeenCalledWith("this_month");
      expect(m.getLibraryGrowth).toHaveBeenCalledWith("this_month");
      expect(m.getFormatDistribution).toHaveBeenCalledWith();
      expect(m.getHeatmapHourly).toHaveBeenCalledWith("this_month");
      expect(m.getHeatmapWeekday).toHaveBeenCalledWith("this_month");
      expect(m.getFavoriteTrends).toHaveBeenCalledWith("this_month");
      expect(m.getPlaybackHistoryTimeline).toHaveBeenCalledWith("this_month", 100);
    });

    it("populates state on full success", async () => {
      mockAllFulfilled();
      m.getStatsOverview.mockResolvedValue({ total_plays: 50, total_listening_time_seconds: 3600 });
      m.getTopTracksWithStats.mockResolvedValue([{ track_id: 1, play_count: 10 } as any]);
      m.getStreakData.mockResolvedValue({ current_streak_days: 3, longest_streak_days: 10 } as any);

      await statsMod.stats.loadAll();

      expect(statsMod.stats.loading).toBe(false);
      expect(statsMod.stats.overview).toEqual({
        total_plays: 50,
        total_listening_time_seconds: 3600,
      });
      expect(statsMod.stats.topTracks).toHaveLength(1);
      expect(statsMod.stats.streakData).toEqual({
        current_streak_days: 3,
        longest_streak_days: 10,
      });
    });

    it("handles partial failure without setting error", async () => {
      mockAllFulfilled();
      m.getStatsOverview.mockResolvedValue({ total_plays: 10, total_listening_time_seconds: 600 });
      m.getTopTracksWithStats.mockRejectedValue(new Error("tracks fail"));

      await statsMod.stats.loadAll();

      expect(statsMod.stats.overview).toEqual({
        total_plays: 10,
        total_listening_time_seconds: 600,
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
