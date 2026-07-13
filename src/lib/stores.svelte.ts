import { initSettings } from "$lib/settings.svelte";
import type { Album, Artist, Playlist, Track } from "$lib/types";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { appDataDir } from "@tauri-apps/api/path";

class LibraryStore {
  tracks = $state<Track[]>([]);
  albums = $state<Album[]>([]);
  artists = $state<Artist[]>([]);
  playlists = $state<Playlist[]>([]);
  ready = $state(false);
  loading = $state(false);
  error = $state<string | null>(null);

  /** Reactive so cover/artist images re-render when the path becomes available. */
  appDataDirPath = $state<string | null>(null);
  #appDataDirPromise: Promise<string> | null = null;

  tracksById = new Map<number, Track>();
  albumsById = new Map<number, Album>();
  artistsById = new Map<number, Artist>();
  playlistsById = new Map<number, Playlist>();

  // Derived data
  favoriteTracks = $derived(this.tracks.filter((t) => t.is_favorite));

  recentlyAddedTracks = $derived(
    [...this.tracks].sort(
      (a, b) => new Date(b.added_at).getTime() - new Date(a.added_at).getTime(),
    ),
  );

  tracksByAlbum(albumId: number): Track[] {
    return this.tracks.filter((t) => t.album.id === albumId);
  }

  tracksByArtist(artistId: number): Track[] {
    return this.tracks.filter((t) => t.artists.some((a) => a.id === artistId));
  }

  tracksByPlaylist(playlistId: number): Track[] {
    return this.tracks.filter((t) => t.playlist_ids.includes(playlistId));
  }

  albumsByArtist(artistId: number): Album[] {
    const albumIds = new Set(
      this.tracks.filter((t) => t.artists.some((a) => a.id === artistId)).map((t) => t.album.id),
    );
    return this.albums.filter((a) => albumIds.has(a.id));
  }

  /**
   * Resolve app data dir once. Safe to call from layout and init concurrently.
   */
  ensureAppDataDir(): Promise<string> {
    if (this.appDataDirPath) {
      return Promise.resolve(this.appDataDirPath);
    }
    if (!this.#appDataDirPromise) {
      this.#appDataDirPromise = appDataDir()
        .then((dir) => {
          this.appDataDirPath = dir;
          return dir;
        })
        .catch((e) => {
          this.#appDataDirPromise = null;
          throw e;
        });
    }
    return this.#appDataDirPromise;
  }

  getImageSrc(
    filename: string | undefined | null,
    type: "cover" | "artist" | "banner" = "cover",
  ): string | null {
    if (!filename) {
      return null;
    }
    // Path not ready yet — callers re-render when appDataDirPath becomes available.
    if (!this.appDataDirPath) {
      return null;
    }
    const subdir = type === "artist" ? "artists" : type === "banner" ? "artist_banner" : "covers";
    return convertFileSrc(`${this.appDataDirPath}/${subdir}/${filename}`);
  }

  async init() {
    if (this.loading) return;
    this.loading = true;
    this.error = null;

    try {
      await initSettings();

      await Promise.all([
        this.ensureAppDataDir(),
        this.#loadTracks(),
        this.#loadAlbums(),
        this.#loadArtists(),
        this.#loadPlaylists(),
      ]);
      this.#preprocess();

      this.ready = true;
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load library";
      console.error("LibraryStore.init failed:", e);
    } finally {
      this.loading = false;
    }

    // Listen for library-updated events from scanner/watcher
    listen("library-updated", () => {
      this.#reloadAll();
    });
  }

  async #reloadAll() {
    try {
      await Promise.all([
        this.#loadTracks(),
        this.#loadAlbums(),
        this.#loadArtists(),
        this.#loadPlaylists(),
      ]);
      this.#preprocess();
    } catch (e) {
      console.error("LibraryStore reload failed:", e);
    }
  }

  async #loadTracks() {
    const tracks = await invoke<Track[]>("get_all_tracks");
    this.tracks = tracks || [];
  }

  async #loadAlbums() {
    const albums = await invoke<Album[]>("get_all_albums");
    this.albums = albums || [];
  }

  async #loadArtists() {
    const artists = await invoke<Artist[]>("get_artists");
    this.artists = artists || [];
  }

  async #loadPlaylists() {
    const result = await invoke<(Playlist & { coverArts: string[] })[]>("get_playlists");
    this.playlists = result || [];
  }

  #preprocess() {
    this.tracksById = new Map();
    this.albumsById = new Map();
    this.artistsById = new Map();
    this.playlistsById = new Map();

    for (const track of this.tracks) {
      this.tracksById.set(track.id, track);
    }

    for (const album of this.albums) {
      this.albumsById.set(album.id, album);
    }

    for (const artist of this.artists) {
      this.artistsById.set(artist.id, artist);
    }

    for (const playlist of this.playlists) {
      this.playlistsById.set(playlist.id, playlist);
    }
  }

  async reloadArtists() {
    await this.#loadArtists();
    for (const artist of this.artists) {
      this.artistsById.set(artist.id, artist);
    }
  }

  applyTrackUpdate(track: Track) {
    const idx = this.tracks.findIndex((t) => t.id === track.id);
    if (idx !== -1) {
      this.tracks[idx] = track;
    }
    this.tracksById.set(track.id, track);
  }

  applyAlbumUpdate(album: Album) {
    const idx = this.albums.findIndex((a) => a.id === album.id);
    if (idx !== -1) {
      this.albums[idx] = album;
    }
    this.albumsById.set(album.id, album);
  }

  applyArtistUpdate(artist: Artist) {
    const idx = this.artists.findIndex((a) => a.id === artist.id);
    if (idx !== -1) {
      this.artists[idx] = artist;
    }
    this.artistsById.set(artist.id, artist);
  }

  applyPlaylistUpdate(playlist: Playlist & { coverArts: string[] }) {
    const idx = this.playlists.findIndex((p) => p.id === playlist.id);
    if (idx !== -1) {
      this.playlists[idx] = playlist;
    }
    this.playlistsById.set(playlist.id, playlist);
  }

  async toggleFavorite(trackId: number): Promise<Track> {
    const updated = await invoke<Track>("toggle_favorite", { id: trackId });
    this.applyTrackUpdate(updated);
    return updated;
  }

  async saveAlbum(id: number, name?: string, cover_art?: string): Promise<Album> {
    const updated = await invoke<Album>("update_album", {
      id,
      name: name ?? null,
      coverArt: cover_art ?? null,
    });
    this.applyAlbumUpdate(updated);
    return updated;
  }

  async saveArtist(
    id: number,
    name?: string,
    profile_image?: string,
    banner_image?: string,
  ): Promise<Artist> {
    const updated = await invoke<Artist>("update_artist", {
      id,
      name: name ?? null,
      profileImage: profile_image ?? null,
      bannerImage: banner_image ?? null,
    });
    this.applyArtistUpdate(updated);
    return updated;
  }

  async savePlaylist(
    id: number,
    name?: string,
    cover_art?: string,
  ): Promise<Playlist & { coverArts: string[] }> {
    const updated = await invoke<Playlist & { coverArts: string[] }>("update_playlist", {
      id,
      name: name ?? null,
      coverArt: cover_art ?? null,
    });
    this.applyPlaylistUpdate(updated);
    return updated;
  }

  async createPlaylist(name: string): Promise<Playlist & { coverArts: string[] }> {
    const created = await invoke<Playlist & { coverArts: string[] }>("create_playlist", { name });
    this.applyPlaylistUpdate(created);
    return created;
  }

  async deletePlaylist(id: number): Promise<void> {
    await invoke("delete_playlist", { playlistId: id });
    this.playlists = this.playlists.filter((p) => p.id !== id);
    this.playlistsById.delete(id);
  }

  async addTrackToPlaylist(trackId: number, playlistId: number): Promise<void> {
    await invoke("add_track_to_playlist", { trackId, playlistId });
    const track = this.tracks.find((t) => t.id === trackId);
    if (track && !track.playlist_ids.includes(playlistId)) {
      track.playlist_ids.push(playlistId);
    }
  }

  async removeTrackFromPlaylist(trackId: number, playlistId: number): Promise<void> {
    await invoke("remove_track_from_playlist", { trackId, playlistId });
    const track = this.tracks.find((t) => t.id === trackId);
    if (track) {
      track.playlist_ids = track.playlist_ids.filter((id) => id !== playlistId);
      this.tracks = this.tracks.map((t) => (t.id === track.id ? track : t));
    }
  }

  async getTrackPlaylistIds(trackId: number): Promise<number[]> {
    return invoke<number[]>("get_track_playlist_ids", { trackId });
  }
}

export const store = new LibraryStore();
