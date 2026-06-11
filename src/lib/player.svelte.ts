import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface Track {
    id: number;
    path: string;
    title: string;
    artist: string;
    album: string;
    genre: string;
    duration_seconds: number;
    is_favorite: boolean;
}

export type RepeatMode = 'none' | 'one' | 'all';

class PlayerState {
    currentTrack = $state<Track | null>(null);
    isPlaying = $state(false);
    progress = $state(0);
    volume = $state(100);
    shuffle = $state(false);
    repeat = $state<RepeatMode>('none');
    queue = $state<Track[]>([]);
    currentIndex = $state(-1);

    constructor() {
        this.setupListeners();
    }

    async setupListeners() {
        await listen("playback-status", (event: any) => {
            this.progress = event.payload.position;
            this.isPlaying = event.payload.is_playing;
        });

        await listen("playback-completed", () => {
            this.next();
        });
    }

    async play(track: Track, tracks?: Track[]) {
        if (tracks) {
            this.queue = tracks;
            this.currentIndex = tracks.findIndex(t => t.id === track.id);
        } else if (!this.queue.includes(track)) {
            this.queue = [track];
            this.currentIndex = 0;
        }

        this.currentTrack = track;
        this.isPlaying = true;
        await invoke("play_track", { id: track.id });
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
        const seconds = Math.floor((percent / 100) * this.currentTrack.duration_seconds);
        this.progress = seconds;
        await invoke("seek", { seconds });
    }

    async next() {
        if (this.queue.length === 0) return;

        if (this.repeat === 'one' && this.currentTrack) {
            await this.play(this.currentTrack);
            return;
        }

        let nextIndex = this.currentIndex + 1;
        if (this.shuffle) {
            nextIndex = Math.floor(Math.random() * this.queue.length);
        }

        if (nextIndex >= this.queue.length) {
            if (this.repeat === 'all') {
                nextIndex = 0;
            } else {
                this.isPlaying = false;
                this.currentTrack = null;
                return;
            }
        }

        this.currentIndex = nextIndex;
        await this.play(this.queue[nextIndex]);
    }

    async previous() {
        if (this.queue.length === 0) return;

        let prevIndex = this.currentIndex - 1;
        if (prevIndex < 0) {
            if (this.repeat === 'all') {
                prevIndex = this.queue.length - 1;
            } else {
                prevIndex = 0;
            }
        }

        this.currentIndex = prevIndex;
        await this.play(this.queue[prevIndex]);
    }

    toggleShuffle() {
        this.shuffle = !this.shuffle;
    }

    toggleRepeat() {
        const modes: RepeatMode[] = ['none', 'all', 'one'];
        const currentIndex = modes.indexOf(this.repeat);
        this.repeat = modes[(currentIndex + 1) % modes.length];
    }
}

export const player = new PlayerState();
