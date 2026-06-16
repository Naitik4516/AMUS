import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Track } from "./types";

export type RepeatMode = "none" | "one" | "all";

class PlayerState {
  currentTrack = $state<Track | null>(null);
  isPlaying = $state(false);
  progress = $state(0);
  volume = $state(100);
  shuffle = $state(false);
  repeat = $state<RepeatMode>("none");
  queue = $state<Track[]>([]);
  currentIndex = $state(-1);

  constructor() {
    this.setupListeners();
  }

  async setupListeners() {
    await listen("playback-state", (event: any) => {
      this.progress = event.payload.position_ms;
      this.isPlaying = event.payload.is_playing;
      if (event.payload.current_track) {
        const dt = event.payload.current_track;
        if (!this.currentTrack || this.currentTrack.id !== dt.id) {
          this.currentTrack = {
            id: dt.id,
            title: dt.title,
            artist: dt.artist,
            album: dt.album,
            genre: dt.genre,
            duration_seconds: dt.duration_seconds,
            is_favorite: dt.is_favorite,
            cover_art: dt.cover_art,
          };
        }
        
        // Update currentIndex if it's in the queue
        const index = this.queue.findIndex(t => t.id === dt.id);
        if (index !== -1) {
          this.currentIndex = index;
        }
      }
    });

    await listen("playback-completed", () => {
      this.next();
    });
  }

  async play(track: Track, tracks?: Track[]) {
    if (tracks) {
      this.queue = [...tracks];
      this.currentIndex = this.queue.findIndex((t) => t.id === track.id);
    } else {
      // If track is already in queue, just jump to it
      const index = this.queue.findIndex((t) => t.id === track.id);
      if (index !== -1) {
        this.currentIndex = index;
      } else {
        // Otherwise, play single and get similar
        await this.playWithSimilar(track);
        return;
      }
    }

    this.currentTrack = track;
    this.isPlaying = true;
    await invoke("play_track", { id: track.id });
  }

  async playWithSimilar(track: Track) {
    this.currentTrack = track;
    this.isPlaying = true;
    this.queue = [track];
    this.currentIndex = 0;

    await invoke("play_track", { id: track.id });

    try {
      const similar = await invoke<Track[]>("get_similar_songs", {
        id: track.id,
      });
      this.queue = [track, ...similar];
    } catch (e) {
      console.error("Failed to fetch similar songs", e);
    }
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
    this.progress = seconds;
    await invoke("seek", { seconds });
  }

  async next() {
    if (this.queue.length === 0) return;

    if (this.repeat === "one" && this.currentTrack) {
      await this.play(this.currentTrack, this.queue);
      return;
    }

    let nextIndex = this.currentIndex + 1;
    if (this.shuffle) {
      nextIndex = Math.floor(Math.random() * this.queue.length);
    }

    if (nextIndex >= this.queue.length) {
      if (this.repeat === "all") {
        nextIndex = 0;
      } else {
        this.isPlaying = false;
        this.currentTrack = null;
        return;
      }
    }

    this.currentIndex = nextIndex;
    await this.play(this.queue[nextIndex], this.queue);
  }

  async previous() {
    if (this.queue.length === 0) return;

    let prevIndex = this.currentIndex - 1;
    if (prevIndex < 0) {
      if (this.repeat === "all") {
        prevIndex = this.queue.length - 1;
      } else {
        prevIndex = 0;
      }
    }

    this.currentIndex = prevIndex;
    await this.play(this.queue[prevIndex], this.queue);
  }

  toggleShuffle() {
    this.shuffle = !this.shuffle;
  }

  toggleRepeat() {
    const modes: RepeatMode[] = ["none", "all", "one"];
    const currentIndex = modes.indexOf(this.repeat);
    this.repeat = modes[(currentIndex + 1) % modes.length];
  }

  addToQueue(track: Track) {
    if (!this.queue.find((t) => t.id === track.id)) {
      this.queue = [...this.queue, track];
    }
  }

  removeFromQueue(trackId: number) {
    const index = this.queue.findIndex((t) => t.id === trackId);
    if (index === -1) return;

    if (index === this.currentIndex) {
      this.next();
    }

    this.queue = this.queue.filter((t) => t.id !== trackId);

    // Adjust currentIndex if necessary
    if (index < this.currentIndex) {
      this.currentIndex--;
    }
  }

  async toggleFavorite(track: Track) {
    try {
      const isFav = await invoke<boolean>("toggle_favorite", { id: track.id });

      // Update current track
      if (this.currentTrack && this.currentTrack.id === track.id) {
        this.currentTrack.is_favorite = isFav;
      }

      // Update queue
      this.queue = this.queue.map((t) => {
        if (t.id === track.id) {
          return { ...t, is_favorite: isFav };
        }
        return t;
      });

      return isFav;
    } catch (e) {
      console.error("Failed to toggle favorite", e);
      return track.is_favorite;
    }
  }

  setQueue(tracks: Track[]) {
    this.queue = [...tracks];
    this.currentIndex = -1;
  }
}

export const player = new PlayerState();
