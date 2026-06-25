import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Track } from "./types";

export type RepeatMode = "none" | "one" | "all";
export type SourceType = "album" | "playlist" | "artist" | "favorites" | "other";

function toTrack(dt: any): Track {
    return {
        id: dt.id,
        title: dt.title,
        artists: dt.artists || [],
        album: dt.album || { id: 0, name: "Unknown Album" },
        duration_seconds: dt.duration_seconds,
        is_favorite: dt.is_favorite,
        cover_art: dt.cover_art,
        added_at: dt.added_at,
    };
}

class PlayerState {
    currentTrack = $state<Track | null>(null);
    isPlaying = $state(false);
    progress = $state(0);
    volume = $state(100);
    shuffle = $state(false);
    repeat = $state<RepeatMode>("none");
    userQueue = $state<Track[]>([]);
    playNext = $state<Track[]>([]);

    fullQueue = $derived([...this.userQueue, ...this.playNext]);

    currentIsFromQueue = $derived(
        this.userQueue.some((t) => t.id === this.currentTrack?.id),
    );

    constructor() {
        this.setupListeners();
    }

    async setupListeners() {
        await listen("playback-state", (event: any) => {
            this.progress = event.payload.position_ms;
            this.isPlaying = event.payload.is_playing;
        });

        await listen("track-changed", (event: any) => {
            this.updateFromState(event.payload);
        });

        await this.loadQueue();
        this.syncState();
    }

    async loadQueue() {
        try {
            const tracks = await invoke<Track[]>("load_queue");
            if (tracks.length > 0) {
                for (const t of tracks) {
                    await invoke("add_to_queue", { trackId: t.id });
                }
            }
        } catch (e) {
            console.error("Failed to load persisted queue", e);
        }
    }

    async persistQueue() {
        try {
            const ids = this.userQueue.map((t) => t.id);
            await invoke("save_queue", { trackIds: ids });
        } catch (e) {
            console.error("Failed to persist queue", e);
        }
    }

    async syncState() {
        try {
            const state = await invoke<any>("get_queue_state");
            this.updateFromState(state);
        } catch (e) {
            console.error("Failed to sync state", e);
        }
    }

    private updateFromState(payload: any) {
        this.isPlaying = payload.is_playing;
        this.progress = payload.position_ms;
        this.volume = Math.round(payload.volume * 100);
        this.shuffle = payload.shuffle;
        const modes: RepeatMode[] = ["none", "all", "one"];
        this.repeat = modes[payload.repeat] ?? "none";

        if (payload.current_track) {
            this.currentTrack = toTrack(payload.current_track);
        }

        this.userQueue = (payload.user_queue || []).map(toTrack);
        this.playNext = (payload.play_next || []).map(toTrack);
    }

    async play(track: Track, tracks?: Track[], sourceType?: SourceType, sourceId?: number | null) {
        const st = sourceType ?? "other";
        const idx = tracks ? tracks.findIndex((t) => t.id === track.id) : -1;
        const playNextIds: number[] = idx >= 0 && tracks ? tracks.slice(idx + 1).map((t) => t.id) : [];

        await invoke("play_from_source", {
            trackId: track.id,
            sourceType: st,
            sourceId: sourceId ?? null,
            playNextIds,
        });
    }

    async togglePlay() {
        this.isPlaying = !this.isPlaying;
        await invoke("toggle_playback", { playing: this.isPlaying });
    }

    async setVolume(vol: number) {
        this.volume = vol;
        await invoke("set_volume", { volume: vol });
    }

    async seek(percent: number) {
        if (!this.currentTrack) return;
        const seconds = Math.floor(
            (percent / 100) * this.currentTrack.duration_seconds,
        );
        this.progress = seconds * 1000;
        await invoke("seek", { seconds });
    }

    async next() {
        await invoke("play_next_track");
    }

    async previous() {
        await invoke("play_previous_track");
    }

    async toggleShuffle() {
        this.shuffle = !this.shuffle;
        await invoke("set_shuffle", { enabled: this.shuffle });
    }

    async toggleRepeat() {
        const modes: RepeatMode[] = ["none", "all", "one"];
        const i = modes.indexOf(this.repeat);
        this.repeat = modes[(i + 1) % modes.length];
        const modeMap: Record<RepeatMode, number> = { none: 0, one: 1, all: 2 };
        await invoke("set_repeat", { mode: modeMap[this.repeat] });
    }

    async addToQueue(track: Track) {
        await invoke("add_to_queue", { trackId: track.id });
        await this.syncState();
        await this.persistQueue();
    }

    async playNextInQueue(track: Track) {
        await invoke("insert_play_next_queue", { trackId: track.id });
        await this.syncState();
        await this.persistQueue();
    }

    async removeFromQueue(trackId: number) {
        const idx = this.userQueue.findIndex((t) => t.id === trackId);
        if (idx === -1) return;
        await invoke("remove_from_queue", { index: idx });
        await this.syncState();
        await this.persistQueue();
    }

    async reorderQueue(from: number, to: number) {
        await invoke("reorder_queue", { from, to });
        await this.syncState();
        await this.persistQueue();
    }

    async clearQueue() {
        this.userQueue = [];
        await invoke("clear_queue");
        await this.persistQueue();
    }

    async toggleFavorite(track: Track) {
        try {
            const isFav = await invoke<boolean>("toggle_favorite", { id: track.id });
            if (this.currentTrack && this.currentTrack.id === track.id) {
                this.currentTrack.is_favorite = isFav;
            }
            return isFav;
        } catch (e) {
            console.error("Failed to toggle favorite", e);
            return track.is_favorite;
        }
    }
}

export const player = new PlayerState();
