// src/lib/player.svelte.ts
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Track, PlaybackSource, RepeatMode } from "./types";
import { store } from "./stores.svelte";
import { saveSession } from "./session.svelte";

// discriminated union matching the #[serde(tag = "event", content = "payload")] enum
type PlayerEvent =
  | {
      event: "TrackChanged";
      payload: { track: Track; duration_sec: number; source: PlaybackSource };
    }
  | { event: "StateChanged"; payload: { is_playing: boolean } }
  | { event: "Position"; payload: { pos_sec: number; at_epoch_ms: number } }
  | {
      event: "QueueChanged";
      payload: {
        user_queue: Track[];
        context_len: number;
        context_position: number | null;
        queue_view: QueueView;
      };
    }
  | {
      event: "RepeatShuffleChanged";
      payload: { repeat: RepeatMode; shuffle: boolean };
    }
  | { event: "VolumeChanged"; payload: { volume: number } }
  | { event: "PlaybackEnded"; payload: Record<string, never> }
  | { event: "Error"; payload: { message: string; track_id: number | null } };

interface StateSnapshot {
  current_track: Track | null;
  is_playing: boolean;
  position_sec: number;
  duration_sec: number;
  repeat: RepeatMode;
  shuffle: boolean;
  volume: number;
  user_queue: Track[];
  queue_view: QueueView;
}

export interface QueueView {
  context_source_type: string;
  context_label: string | null;
  upcoming_context: Track[];
}

const EVENT_NAME = "player://event";

function toBackendSource(source: PlaybackSource): {
  sourceType: string;
  sourceId: number | null;
} {
  switch (source.type) {
    case "Album":
      return { sourceType: "ALBUM", sourceId: source.id };
    case "Playlist":
      return { sourceType: "PLAYLIST", sourceId: source.id };
    case "Artist":
      return { sourceType: "ARTIST", sourceId: source.id };
    case "Favorites":
      return { sourceType: "FAVORITES", sourceId: null };
    case "Direct":
      return { sourceType: "DIRECT", sourceId: null };
    case "Queue":
      return { sourceType: "QUEUE", sourceId: null };
    default:
      return { sourceType: "OTHER", sourceId: null };
  }
}

class PlayerStore {
  currentTrack = $state<Track | null>(null);
  source = $state<PlaybackSource | null>(null);
  isPlaying = $state(false);
  duration = $state(0);

  position = $state(0);
  volume = $state(1);
  repeatMode = $state<RepeatMode>("OFF");
  shuffleEnabled = $state(false);
  userQueue = $state<Track[]>([]);
  contextLength = $state(0);
  contextPosition = $state<number | null>(null);
  errorMessage = $state<string | null>(null);

  isReady = $state(false);
  contextSourceType = $state<string>("OTHER");
  contextLabel = $state<string | null>(null);
  playNext = $state<Track[]>([]);

  get progress(): number {
    return this.duration > 0 ? Math.min(this.position / this.duration, 1) : 0;
  }

  get nextSectionTitle(): string {
    const namedContextTypes = ["ALBUM", "PLAYLIST", "ARTIST", "FAVORITES"];
    if (namedContextTypes.includes(this.contextSourceType) && this.contextLabel) {
      return `Next from: ${this.contextLabel}`;
    }
    return "Next up";
  }

  #lastKnownPos = 0;
  #lastUpdateAtMs = Date.now();
  #rafHandle: number | null = null;
  #unlisten: UnlistenFn | null = null;

  async init() {
    if (this.#unlisten) return; // already initialized
    this.#unlisten = await listen<PlayerEvent>(EVENT_NAME, (e) => this.#handleEvent(e.payload));
    await this.#hydrate();
    this.#setupBeforeUnload();
  }

  #setupBeforeUnload() {
    const save = () => {
      if (this.#debounceTimer) {
        clearTimeout(this.#debounceTimer);
        this.#debounceTimer = null;
      }
      this.#saveSession();
    };
    window.addEventListener("beforeunload", save);
  }

  destroy() {
    this.#unlisten?.();
    this.#unlisten = null;
    this.#stopTicking();
  }

  async #hydrate() {
    try {
      const snapshot = await invoke<StateSnapshot>("get_current_state");
      this.currentTrack = snapshot.current_track;
      this.isPlaying = snapshot.is_playing;
      this.duration = snapshot.duration_sec;
      this.repeatMode = snapshot.repeat;
      this.shuffleEnabled = snapshot.shuffle;
      this.volume = snapshot.volume;
      this.userQueue = snapshot.user_queue;
      this.#applyQueueView(snapshot.queue_view);
      this.#setPosition(snapshot.position_sec);
      if (this.isPlaying) {
        this.#startTicking();
      }
    } catch (err) {
      console.error("failed to hydrate player state", err);
    } finally {
      this.isReady = true;
    }
  }

  #applyQueueView(view: QueueView) {
    this.contextSourceType = view.context_source_type;
    this.contextLabel = view.context_label;
    this.playNext = view.upcoming_context;
  }

  #debounceTimer: ReturnType<typeof setTimeout> | null = null;

  #debouncedSave(fn: () => void) {
    if (this.#debounceTimer) clearTimeout(this.#debounceTimer);
    this.#debounceTimer = setTimeout(fn, 500);
  }

  #saveSession() {
    saveSession({
      current_track_id: this.currentTrack?.id ?? null,
      context_type: this.contextSourceType,
      context_id: this.source && "id" in this.source ? (this.source as any).id : null,
      context_label: this.contextLabel,
      context_position: this.contextPosition ?? 0,
      user_queue_ids: this.userQueue.map((t) => t.id),
      position_sec: this.#lastKnownPos,
      volume: this.volume,
      repeat: this.repeatMode,
      shuffle: this.shuffleEnabled,
      saved_at: Date.now(),
    });
  }

  #handleEvent(evt: PlayerEvent) {
    switch (evt.event) {
      case "TrackChanged":
        this.currentTrack = evt.payload.track;
        this.duration = evt.payload.duration_sec;
        this.source = evt.payload.source;
        this.errorMessage = null;
        this.#setPosition(0);
        this.#debouncedSave(() => this.#saveSession());
        if (this.isPlaying) {
          this.#startTicking();
        }
        break;
      case "StateChanged":
        this.isPlaying = evt.payload.is_playing;
        if (this.isPlaying) {
          this.#startTicking();
        } else {
          this.#stopTicking();
        }
        break;
      case "Position":
        this.#setPosition(evt.payload.pos_sec, evt.payload.at_epoch_ms);
        break;
      case "QueueChanged":
        this.userQueue = evt.payload.user_queue;
        this.contextLength = evt.payload.context_len;
        this.contextPosition = evt.payload.context_position;
        this.#applyQueueView(evt.payload.queue_view);
        this.#debouncedSave(() => this.#saveSession());
        break;
      case "RepeatShuffleChanged":
        this.repeatMode = evt.payload.repeat;
        this.shuffleEnabled = evt.payload.shuffle;
        this.#debouncedSave(() => this.#saveSession());
        break;
      case "VolumeChanged":
        this.volume = evt.payload.volume;
        this.#debouncedSave(() => this.#saveSession());
        break;
      case "PlaybackEnded":
        this.currentTrack = null;
        this.duration = 0;
        this.source = null;
        this.isPlaying = false;
        this.#setPosition(0);
        this.#stopTicking();
        break;
      case "Error":
        this.errorMessage = evt.payload.message;
        break;
    }
  }

  #setPosition(posSec: number, atEpochMs = Date.now()) {
    if (!this.currentTrack) {
      this.position = 0;
      return;
    }
    this.#lastKnownPos = posSec;
    this.#lastUpdateAtMs = atEpochMs;
    this.position = posSec;
  }

  #startTicking() {
    if (this.#rafHandle !== null) return;
    const tick = () => {
      if (!this.isPlaying || !this.currentTrack) {
        this.#rafHandle = null;
        return;
      }
      const elapsedSec = (Date.now() - this.#lastUpdateAtMs) / 1000;
      const cap = this.duration > 0 ? this.duration : Infinity;
      this.position = Math.min(this.#lastKnownPos + elapsedSec, cap);
      this.#rafHandle = requestAnimationFrame(tick);
    };
    this.#rafHandle = requestAnimationFrame(tick);
  }

  #stopTicking() {
    if (this.#rafHandle !== null) {
      cancelAnimationFrame(this.#rafHandle);
      this.#rafHandle = null;
    }
  }

  async play(
    tracks: Track[],
    source: PlaybackSource = { type: "Direct" },
    startIndex: number = 0,
    label?: string,
  ) {
    const { sourceType, sourceId } = toBackendSource(source);
    await invoke("play_context", {
      tracks,
      sourceType,
      sourceId,
      startIndex,
      contextLabel: label ?? null,
    });
  }

  async playPause() {
    await invoke("play_pause");
  }

  async stop() {
    await invoke("stop");
  }

  async next() {
    await invoke("next");
  }

  async previous() {
    await invoke("previous");
  }

  async seek(positionSec: number) {
    // optimistic update so the slider feels instant; the next Position
    // event will correct any drift from symphonia's keyframe seeking
    this.#setPosition(positionSec);
    await invoke("seek", { positionSec });
  }

  async setVolume(volume: number) {
    this.volume = volume; // optimistic
    await invoke("set_volume", { volume });
  }

  async cycleRepeat() {
    const nextMode: RepeatMode =
      this.repeatMode === "OFF" ? "ALL" : this.repeatMode === "ALL" ? "ONE" : "OFF";
    await invoke("set_repeat", { mode: nextMode });
  }

  async toggleShuffle() {
    await invoke("toggle_shuffle");
  }

  async enqueueNext(track: Track) {
    await invoke("enqueue_next", { track });
  }

  async enqueueEnd(track: Track) {
    await invoke("enqueue_end", { track });
  }

  async enqueueEndMany(tracks: Track[]) {
    await invoke("enqueue_end_many", { tracks });
  }

  async removeFromQueue(queueId: number) {
    await invoke("remove_from_queue", { queueId });
  }

  async clearQueue() {
    await invoke("clear_queue");
  }

  async reorderQueue(queueId: number, newIndex: number) {
    await invoke("reorder_queue", { queueId, newIndex });
  }

  async setAutoplay(enabled: boolean) {
    await invoke("set_autoplay", { enabled });
  }

  async playFromContextIndex(index: number) {
    const track = this.playNext[index];
    if (track) {
      await invoke("play_track_from_context", { trackId: track.id });
    }
  }

  async toggleFavorite(track: Track) {
    try {
      const updated = await store.toggleFavorite(track.id);
      if (this.currentTrack && this.currentTrack.id === track.id) {
        this.currentTrack.is_favorite = updated.is_favorite;
      }
      return updated.is_favorite;
    } catch (e) {
      console.error("Failed to toggle favorite", e);
      return track.is_favorite;
    }
  }
}

export const player = new PlayerStore();
