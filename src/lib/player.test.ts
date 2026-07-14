import { describe, it, expect, vi, beforeEach, afterEach, type Mock } from "vitest";
import { player } from "./player.svelte";
import type { Track, PlaybackSource, RepeatMode } from "./types";
import { mockTrack, mockSource } from "./test-utils";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const mockInvoke = vi.mocked(invoke);
const mockListen = vi.mocked(listen);

// Track the handler that init() passes to listen()
let playerEventHandler: ((event: { payload: unknown }) => void) | null = null;

function resetPlayer() {
  player.destroy();
  player.currentTrack = null;
  player.source = null;
  player.isPlaying = false;
  player.duration = 0;
  player.position = 0;
  player.volume = 1;
  player.repeatMode = "OFF";
  player.shuffleEnabled = false;
  player.userQueue = [];
  player.contextLength = 0;
  player.contextPosition = null;
  player.errorMessage = null;
  player.isReady = false;
  player.contextSourceType = "OTHER";
  player.contextLabel = null;
  player.playNext = [];
  playerEventHandler = null;
}

function emitEvent(payload: unknown) {
  if (playerEventHandler) {
    playerEventHandler({ payload });
  }
}

function mockSnapshot(overrides?: Record<string, unknown>) {
  return {
    current_track: mockTrack(1),
    is_playing: false,
    position_sec: 0,
    duration_sec: 200,
    repeat: "OFF" as RepeatMode,
    shuffle: false,
    volume: 1,
    user_queue: [],
    queue_view: {
      context_source_type: "OTHER",
      context_label: null,
      upcoming_context: [],
    },
    ...overrides,
  };
}

beforeEach(async () => {
  resetPlayer();
  vi.clearAllMocks();
  mockInvoke.mockResolvedValue(undefined);
  playerEventHandler = null;
  mockListen.mockImplementation((_event: string, handler: any) => {
    playerEventHandler = handler;
    return Promise.resolve(() => {
      playerEventHandler = null;
    });
  });
});

afterEach(() => {
  resetPlayer();
});

describe("init / lifecycle", () => {
  it("subscribes to player://event and hydrates state", async () => {
    mockInvoke.mockResolvedValueOnce(
      mockSnapshot({ current_track: mockTrack(42), is_playing: true }),
    );
    await player.init();

    expect(mockListen).toHaveBeenCalledWith("player://event", expect.any(Function));
    expect(mockInvoke).toHaveBeenCalledWith("get_current_state");
    expect(player.currentTrack?.id).toBe(42);
    expect(player.isPlaying).toBe(true);
    expect(player.isReady).toBe(true);
  });

  it("is idempotent — second init does not double-subscribe", async () => {
    mockInvoke.mockResolvedValue(mockSnapshot());
    await player.init();
    mockListen.mockClear();
    mockInvoke.mockClear();

    await player.init();
    expect(mockListen).not.toHaveBeenCalled();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("handles hydration error gracefully", async () => {
    mockInvoke.mockRejectedValueOnce(new Error("db error"));
    await player.init();

    expect(player.isReady).toBe(true);
  });

  it("destroy cleans up listener", async () => {
    let unlistenCalled = false;
    mockListen.mockImplementationOnce(() =>
      Promise.resolve(() => {
        unlistenCalled = true;
      }),
    );
    mockInvoke.mockResolvedValueOnce(mockSnapshot());
    await player.init();

    player.destroy();
    expect(unlistenCalled).toBe(true);
  });

  it("sets up beforeunload listener", async () => {
    mockInvoke.mockResolvedValueOnce(mockSnapshot());
    await player.init();
    expect(window.addEventListener).toHaveBeenCalledWith("beforeunload", expect.any(Function));
  });
});

describe("event handling — state machine", () => {
  beforeEach(() => {
    mockInvoke.mockResolvedValueOnce(mockSnapshot());
    player.init();
  });

  it("TrackChanged updates track, duration, source, resets position", () => {
    emitEvent({
      event: "TrackChanged",
      payload: {
        track: mockTrack(5, { title: "New Song" }),
        duration_sec: 300,
        source: mockSource("Album", 1),
      },
    });

    expect(player.currentTrack?.id).toBe(5);
    expect(player.currentTrack?.title).toBe("New Song");
    expect(player.duration).toBe(300);
    expect(player.source).toEqual(mockSource("Album", 1));
    expect(player.position).toBe(0);
    expect(player.errorMessage).toBeNull();
  });

  it("StateChanged toggles isPlaying and RAF", () => {
    emitEvent({ event: "StateChanged", payload: { is_playing: true } });
    expect(player.isPlaying).toBe(true);

    emitEvent({ event: "StateChanged", payload: { is_playing: false } });
    expect(player.isPlaying).toBe(false);
  });

  it("Position updates position with correct math", () => {
    emitEvent({ event: "Position", payload: { pos_sec: 42.5, at_epoch_ms: Date.now() } });
    expect(player.position).toBe(42.5);
  });

  it("QueueChanged updates queue state", () => {
    const upcoming = [mockTrack(2), mockTrack(3)];
    emitEvent({
      event: "QueueChanged",
      payload: {
        user_queue: [mockTrack(10)],
        context_len: 5,
        context_position: 1,
        queue_view: {
          context_source_type: "ALBUM",
          context_label: "My Album",
          upcoming_context: upcoming,
        },
      },
    });

    expect(player.userQueue).toHaveLength(1);
    expect(player.userQueue[0].id).toBe(10);
    expect(player.contextLength).toBe(5);
    expect(player.contextPosition).toBe(1);
    expect(player.contextSourceType).toBe("ALBUM");
    expect(player.contextLabel).toBe("My Album");
    expect(player.playNext).toEqual(upcoming);
  });

  it("RepeatShuffleChanged updates repeat and shuffle", () => {
    emitEvent({ event: "RepeatShuffleChanged", payload: { repeat: "ONE", shuffle: true } });
    expect(player.repeatMode).toBe("ONE");
    expect(player.shuffleEnabled).toBe(true);
  });

  it("VolumeChanged updates volume", () => {
    emitEvent({ event: "VolumeChanged", payload: { volume: 0.5 } });
    expect(player.volume).toBe(0.5);
  });

  it("PlaybackEnded clears state and stops playback", () => {
    // First simulate a playing state
    player.currentTrack = mockTrack(1);
    player.isPlaying = true;
    player.duration = 200;

    emitEvent({ event: "PlaybackEnded", payload: {} });

    expect(player.currentTrack).toBeNull();
    expect(player.duration).toBe(0);
    expect(player.source).toBeNull();
    expect(player.isPlaying).toBe(false);
    expect(player.position).toBe(0);
  });

  it("Error sets errorMessage", () => {
    emitEvent({ event: "Error", payload: { message: "track not found", track_id: 42 } });
    expect(player.errorMessage).toBe("track not found");
  });
});

describe("public command methods", () => {
  it("play() invokes play_context with correct args", async () => {
    const tracks = [mockTrack(1), mockTrack(2)];
    await player.play(tracks, mockSource("Playlist", 5), 1, "My Playlist");

    expect(mockInvoke).toHaveBeenCalledWith("play_context", {
      tracks,
      sourceType: "PLAYLIST",
      sourceId: 5,
      startIndex: 1,
      contextLabel: "My Playlist",
    });
  });

  it("play() uses defaults for source and index", async () => {
    await player.play([mockTrack(1)]);
    expect(mockInvoke).toHaveBeenCalledWith("play_context", {
      tracks: [mockTrack(1)],
      sourceType: "DIRECT",
      sourceId: null,
      startIndex: 0,
      contextLabel: null,
    });
  });

  it("playPause invokes play_pause", async () => {
    await player.playPause();
    expect(mockInvoke).toHaveBeenCalledWith("play_pause");
  });

  it("stop invokes stop", async () => {
    await player.stop();
    expect(mockInvoke).toHaveBeenCalledWith("stop");
  });

  it("next invokes next", async () => {
    await player.next();
    expect(mockInvoke).toHaveBeenCalledWith("next");
  });

  it("previous invokes previous", async () => {
    await player.previous();
    expect(mockInvoke).toHaveBeenCalledWith("previous");
  });

  it("seek does optimistic update then invokes", async () => {
    player.currentTrack = mockTrack(1);
    player.position = 0;
    await player.seek(120.5);

    expect(player.position).toBe(120.5);
    expect(mockInvoke).toHaveBeenCalledWith("seek", { positionSec: 120.5 });
  });

  it("setVolume does optimistic update then invokes", async () => {
    player.volume = 1;
    await player.setVolume(0.3);

    expect(player.volume).toBe(0.3);
    expect(mockInvoke).toHaveBeenCalledWith("set_volume", { volume: 0.3 });
  });

  it("cycleRepeat cycles OFF -> ALL -> ONE -> OFF", async () => {
    player.repeatMode = "OFF";
    await player.cycleRepeat();
    expect(mockInvoke).toHaveBeenCalledWith("set_repeat", { mode: "ALL" });

    player.repeatMode = "ALL";
    await player.cycleRepeat();
    expect(mockInvoke).toHaveBeenCalledWith("set_repeat", { mode: "ONE" });

    player.repeatMode = "ONE";
    await player.cycleRepeat();
    expect(mockInvoke).toHaveBeenCalledWith("set_repeat", { mode: "OFF" });
  });

  it("toggleShuffle invokes toggle_shuffle", async () => {
    await player.toggleShuffle();
    expect(mockInvoke).toHaveBeenCalledWith("toggle_shuffle");
  });

  it("enqueueNext / enqueueEnd invoke correct commands", async () => {
    const track = mockTrack(7);
    await player.enqueueNext(track);
    expect(mockInvoke).toHaveBeenCalledWith("enqueue_next", { track });

    await player.enqueueEnd(track);
    expect(mockInvoke).toHaveBeenCalledWith("enqueue_end", { track });
  });

  it("enqueueEndMany invokes with array", async () => {
    const tracks = [mockTrack(1), mockTrack(2)];
    await player.enqueueEndMany(tracks);
    expect(mockInvoke).toHaveBeenCalledWith("enqueue_end_many", { tracks });
  });

  it("removeFromQueue invokes with queueId", async () => {
    await player.removeFromQueue(99);
    expect(mockInvoke).toHaveBeenCalledWith("remove_from_queue", { queueId: 99 });
  });

  it("clearQueue invokes clear_queue", async () => {
    await player.clearQueue();
    expect(mockInvoke).toHaveBeenCalledWith("clear_queue");
  });

  it("reorderQueue invokes with queueId and newIndex", async () => {
    await player.reorderQueue(5, 2);
    expect(mockInvoke).toHaveBeenCalledWith("reorder_queue", { queueId: 5, newIndex: 2 });
  });

  it("setAutoplay invokes with enabled flag", async () => {
    await player.setAutoplay(true);
    expect(mockInvoke).toHaveBeenCalledWith("set_autoplay", { enabled: true });
  });

  it("playFromContextIndex plays track from playNext", async () => {
    player.playNext = [mockTrack(1), mockTrack(2)];
    await player.playFromContextIndex(1);
    expect(mockInvoke).toHaveBeenCalledWith("play_track_from_context", { trackId: 2 });
  });

  it("playFromContextIndex does nothing for out-of-bounds index", async () => {
    player.playNext = [mockTrack(1)];
    await player.playFromContextIndex(5);
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("toggleFavorite delegates to store and updates currentTrack", async () => {
    const track = mockTrack(1, { is_favorite: false });
    const updated = mockTrack(1, { is_favorite: true });
    mockInvoke.mockResolvedValueOnce(updated);
    player.currentTrack = track;

    const result = await player.toggleFavorite(track);
    expect(result).toBe(true);
    expect(mockInvoke).toHaveBeenCalledWith("toggle_favorite", { id: 1 });
    expect(player.currentTrack?.is_favorite).toBe(true);
  });
});

describe("derived state", () => {
  it("progress returns ratio capped at 1", () => {
    player.duration = 200;
    player.position = 50;
    expect(player.progress).toBe(0.25);

    player.position = 300;
    expect(player.progress).toBe(1);

    player.duration = 0;
    expect(player.progress).toBe(0);
  });

  it("nextSectionTitle returns context label for named types", () => {
    player.contextSourceType = "ALBUM";
    player.contextLabel = "Greatest Hits";
    expect(player.nextSectionTitle).toBe("Next from: Greatest Hits");

    player.contextSourceType = "PLAYLIST";
    expect(player.nextSectionTitle).toBe("Next from: Greatest Hits");

    player.contextSourceType = "ARTIST";
    expect(player.nextSectionTitle).toBe("Next from: Greatest Hits");

    player.contextSourceType = "FAVORITES";
    expect(player.nextSectionTitle).toBe("Next from: Greatest Hits");
  });

  it("nextSectionTitle returns generic for unnamed types", () => {
    player.contextSourceType = "OTHER";
    player.contextLabel = null;
    expect(player.nextSectionTitle).toBe("Next up");
  });
});

describe("position interpolation (RAF)", () => {
  beforeEach(async () => {
    vi.useFakeTimers();
    resetPlayer();
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue(
      mockSnapshot({ is_playing: true, current_track: mockTrack(1), duration_sec: 300 }),
    );
    playerEventHandler = null;
    mockListen.mockImplementation((_event: string, handler: any) => {
      playerEventHandler = handler;
      return Promise.resolve(() => {
        playerEventHandler = null;
      });
    });
    await player.init();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("advances position as time passes", () => {
    vi.advanceTimersByTime(32);
    expect(player.position).toBeGreaterThan(0);
  });

  it("caps position at duration", () => {
    player.duration = 5;
    vi.advanceTimersByTime(10000);
    expect(player.position).toBeLessThanOrEqual(5);
  });
});

describe("toBackendSource mapping", () => {
  it("maps Album -> ALBUM with id", async () => {
    player.currentTrack = mockTrack(1);
    await player.play([mockTrack(1)], mockSource("Album", 10));
    expect(mockInvoke).toHaveBeenCalledWith(
      "play_context",
      expect.objectContaining({
        sourceType: "ALBUM",
        sourceId: 10,
      }),
    );
  });

  it("maps Favorites -> FAVORITES with null id", async () => {
    await player.play([mockTrack(1)], mockSource("Favorites"));
    expect(mockInvoke).toHaveBeenCalledWith(
      "play_context",
      expect.objectContaining({
        sourceType: "FAVORITES",
        sourceId: null,
      }),
    );
  });

  it("maps default -> OTHER with null id", async () => {
    await player.play([mockTrack(1)], { type: "Direct" });
    expect(mockInvoke).toHaveBeenCalledWith(
      "play_context",
      expect.objectContaining({
        sourceType: "DIRECT",
        sourceId: null,
      }),
    );
  });
});
