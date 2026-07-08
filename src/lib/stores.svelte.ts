import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { convertFileSrc } from "@tauri-apps/api/core";
import { appDataDir } from "@tauri-apps/api/path";
import { initSettings, settings } from "$lib/settings.svelte";
import type { Track, Album, Artist, Playlist, GlobalSearchResult } from "$lib/types";

// ---------------------------------------------------------------------------
// Precomputed image URL cache (avoids repeated async getImageUrl calls)
// ---------------------------------------------------------------------------
let appDataDirPath: string | null = null;
let appDirReady = false;

async function ensureAppDir() {
  if (!appDirReady) {
    appDataDirPath = await appDataDir();
    appDirReady = true;
  }
}

function precomputeImageUrl(filename: string | null | undefined, type: "cover" | "artist" | "banner"): string | null {
  if (!filename || !appDataDirPath) return null;
  const subdir =
    type === "artist" ? "artists" :
    type === "banner" ? "artist_banner" :
    "covers";
  return convertFileSrc(`${appDataDirPath}/${subdir}/${filename}`);
}

// ---------------------------------------------------------------------------
// Search normalization cache
// ---------------------------------------------------------------------------
interface NormalizedNames {
  tracks: Map<number, { title: string; artists: string[]; album: string }>;
  albums: Map<number, string>;
  artists: Map<number, string>;
  playlists: Map<number, string>;
}

// ---------------------------------------------------------------------------
// LibraryStore Singleton
// ---------------------------------------------------------------------------
class LibraryStore {
  // Reactive state
  tracks = $state<Track[]>([]);
  albums = $state<Album[]>([]);
  artists = $state<Artist[]>([]);
  playlists = $state<(Playlist & { coverArts: string[] })[]>([]);
  playlistTrackIds = $state<Map<number, number[]>>(new Map());
  ready = $state(false);
  loading = $state(false);
  error = $state<string | null>(null);

  // Non-reactive hash maps for O(1) lookup
  tracksById = new Map<number, Track>();
  albumsById = new Map<number, Album>();
  artistsById = new Map<number, Artist>();
  playlistsById = new Map<number, Playlist & { coverArts: string[] }>();

  // Precomputed cover URLs (non-reactive)
  trackCoverUrls = new Map<number, string | null>();
  albumCoverUrls = new Map<number, string | null>();
  artistProfileUrls = new Map<number, string | null>();
  artistBannerUrls = new Map<number, string | null>();
  playlistCoverUrls = new Map<number, string | null>();

  // Search normalization cache
  #normalized: NormalizedNames = {
    tracks: new Map(),
    albums: new Map(),
    artists: new Map(),
    playlists: new Map(),
  };

  // Derived data
  favoriteTracks = $derived(this.tracks.filter(t => t.is_favorite));
  recentlyAddedTracks = $derived(
    [...this.tracks].sort((a, b) => new Date(b.added_at).getTime() - new Date(a.added_at).getTime())
  );

  tracksByAlbum(albumId: number): Track[] {
    return this.tracks.filter(t => t.album.id === albumId);
  }

  tracksByArtist(artistId: number): Track[] {
    return this.tracks.filter(t => t.artists.some(a => a.id === artistId));
  }

  albumsByArtist(artistId: number): Album[] {
    const albumIds = new Set(
      this.tracks
        .filter(t => t.artists.some(a => a.id === artistId))
        .map(t => t.album.id)
    );
    return this.albums.filter(a => albumIds.has(a.id));
  }

  getTrackCoverUrl(track: Track): string | null {
    return this.trackCoverUrls.get(track.id) ?? null;
  }

  getAlbumCoverUrl(album: Album): string | null {
    return this.albumCoverUrls.get(album.id) ?? null;
  }

  getArtistProfileUrl(artist: Artist): string | null {
    return this.artistProfileUrls.get(artist.id) ?? null;
  }

  getArtistBannerUrl(artist: Artist): string | null {
    return this.artistBannerUrls.get(artist.id) ?? null;
  }

  // ---------------------------------------------------------------------------
  // Init
  // ---------------------------------------------------------------------------
  async init() {
    if (this.loading) return;
    this.loading = true;
    this.error = null;

    try {
      await initSettings();
      await ensureAppDir();

      await Promise.all([
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
    const pt = await invoke<[number, number[]][]>("get_all_playlist_tracks");
    this.playlistTrackIds = new Map(pt);
  }

  // ---------------------------------------------------------------------------
  // Preprocess data (normalize strings, precompute URLs, build maps)
  // ---------------------------------------------------------------------------
  #preprocess() {
    // Reset maps
    this.tracksById = new Map();
    this.albumsById = new Map();
    this.artistsById = new Map();
    this.playlistsById = new Map();
    this.trackCoverUrls = new Map();
    this.albumCoverUrls = new Map();
    this.artistProfileUrls = new Map();
    this.artistBannerUrls = new Map();
    this.playlistCoverUrls = new Map();
    this.#normalized.tracks = new Map();
    this.#normalized.albums = new Map();
    this.#normalized.artists = new Map();
    this.#normalized.playlists = new Map();

    for (const track of this.tracks) {
      this.tracksById.set(track.id, track);
      this.#normalized.tracks.set(track.id, {
        title: track.title.toLowerCase(),
        artists: track.artists.map(a => a.name.toLowerCase()),
        album: track.album.name.toLowerCase(),
      });
      this.trackCoverUrls.set(track.id, precomputeImageUrl(track.cover_art, "cover"));
    }

    for (const album of this.albums) {
      this.albumsById.set(album.id, album);
      this.#normalized.albums.set(album.id, album.name.toLowerCase());
      this.albumCoverUrls.set(album.id, precomputeImageUrl(album.cover_art, "cover"));
    }

    for (const artist of this.artists) {
      this.artistsById.set(artist.id, artist);
      this.#normalized.artists.set(artist.id, artist.name.toLowerCase());
      this.artistProfileUrls.set(artist.id, precomputeImageUrl(artist.profile_image, "artist"));
      this.artistBannerUrls.set(artist.id, precomputeImageUrl(artist.banner_image, "banner"));
    }

    for (const playlist of this.playlists) {
      this.playlistsById.set(playlist.id, playlist);
      this.#normalized.playlists.set(playlist.id, playlist.name.toLowerCase());
    }
  }

  // ---------------------------------------------------------------------------
  // Global Search (pure JS, no IPC)
  // ---------------------------------------------------------------------------
  globalSearch(query: string, limit: number = 20): GlobalSearchResult[] {
    if (query.length < 2) return [];

    const q = query.toLowerCase();
    const results: GlobalSearchResult[] = [];
    const perTypeLimit = limit * 2;

    // Tracks
    for (const track of this.tracks) {
      const n = this.#normalized.tracks.get(track.id);
      if (!n) continue;
      let bestScore = 0;
      const titleScore = this.#scoreEntity(n.title, q, 10);
      if (titleScore > bestScore) bestScore = titleScore;
      for (const artistName of n.artists) {
        const s = this.#scoreEntity(artistName, q, 6);
        if (s > bestScore) bestScore = s;
      }
      if (bestScore > 0) {
        results.push({ result_type: "track", score: bestScore, track, artist: undefined, album: undefined, playlist: undefined });
      }
    }

    // Artists
    for (const artist of this.artists) {
      const name = this.#normalized.artists.get(artist.id);
      if (!name) continue;
      const score = this.#scoreEntity(name, q, 10);
      if (score > 0) {
        results.push({ result_type: "artist", score, artist, track: undefined, album: undefined, playlist: undefined });
      }
    }

    // Albums
    for (const album of this.albums) {
      const name = this.#normalized.albums.get(album.id);
      if (!name) continue;
      const score = this.#scoreEntity(name, q, 8);
      if (score > 0) {
        results.push({ result_type: "album", score, album, track: undefined, artist: undefined, playlist: undefined });
      }
    }

    // Playlists
    for (const playlist of this.playlists) {
      const name = this.#normalized.playlists.get(playlist.id);
      if (!name) continue;
      const score = this.#scoreEntity(name, q, 7);
      if (score > 0) {
        results.push({ result_type: "playlist", score, playlist, track: undefined, artist: undefined, album: undefined });
      }
    }

    results.sort((a, b) => b.score - a.score);
    return results.slice(0, limit);
  }

  #scoreEntity(name: string, query: string, baseWeight: number): number {
    if (name === query) return baseWeight * 4;
    if (name.startsWith(query)) return Math.floor(baseWeight * 2.5);
    if (name.includes(query)) return baseWeight;
    return 0;
  }

  async reloadArtists() {
    await this.#loadArtists();
    for (const artist of this.artists) {
      this.artistsById.set(artist.id, artist);
      this.#normalized.artists.set(artist.id, artist.name.toLowerCase());
      this.artistProfileUrls.set(artist.id, precomputeImageUrl(artist.profile_image, "artist"));
      this.artistBannerUrls.set(artist.id, precomputeImageUrl(artist.banner_image, "banner"));
    }
    this.artists = this.artists;
  }
  applyTrackUpdate(track: Track) {
    const idx = this.tracks.findIndex(t => t.id === track.id);
    if (idx !== -1) {
      this.tracks[idx] = track;
    }
    this.tracksById.set(track.id, track);
    if (track.cover_art !== undefined) {
      this.trackCoverUrls.set(track.id, precomputeImageUrl(track.cover_art, "cover"));
    }
    // Force reactivity by reassigning
    this.tracks = this.tracks;
  }

  applyAlbumUpdate(album: Album) {
    const idx = this.albums.findIndex(a => a.id === album.id);
    if (idx !== -1) {
      this.albums[idx] = album;
    }
    this.albumsById.set(album.id, album);
    this.#normalized.albums.set(album.id, album.name.toLowerCase());
    if (album.cover_art !== undefined) {
      this.albumCoverUrls.set(album.id, precomputeImageUrl(album.cover_art, "cover"));
    }
    this.albums = this.albums;
  }

  applyArtistUpdate(artist: Artist) {
    const idx = this.artists.findIndex(a => a.id === artist.id);
    if (idx !== -1) {
      this.artists[idx] = artist;
    }
    this.artistsById.set(artist.id, artist);
    this.#normalized.artists.set(artist.id, artist.name.toLowerCase());
    if (artist.profile_image !== undefined) {
      this.artistProfileUrls.set(artist.id, precomputeImageUrl(artist.profile_image, "artist"));
    }
    if (artist.banner_image !== undefined) {
      this.artistBannerUrls.set(artist.id, precomputeImageUrl(artist.banner_image, "banner"));
    }
    this.artists = this.artists;
  }

  applyPlaylistUpdate(playlist: Playlist & { coverArts: string[] }) {
    const idx = this.playlists.findIndex(p => p.id === playlist.id);
    if (idx !== -1) {
      this.playlists[idx] = playlist;
    }
    this.playlistsById.set(playlist.id, playlist);
    this.#normalized.playlists.set(playlist.id, playlist.name.toLowerCase());
    this.playlists = this.playlists;
  }

  applyNewPlaylist(playlist: Playlist & { coverArts: string[] }) {
    this.playlists = [...this.playlists, playlist];
    this.playlistsById.set(playlist.id, playlist);
    this.#normalized.playlists.set(playlist.id, playlist.name.toLowerCase());
  }

  applyRemovePlaylist(id: number) {
    this.playlists = this.playlists.filter(p => p.id !== id);
    this.playlistsById.delete(id);
    this.#normalized.playlists.delete(id);
  }

  // ---------------------------------------------------------------------------
  // Mutation wrappers (invoke backend, then apply update)
  // ---------------------------------------------------------------------------
  async toggleFavorite(trackId: number): Promise<Track> {
    const updated = await invoke<Track>("toggle_favorite", { id: trackId });
    this.applyTrackUpdate(updated);
    return updated;
  }

  async saveAlbum(id: number, name?: string, cover_art?: string): Promise<Album> {
    const updated = await invoke<Album>("update_album", { id, name: name ?? null, coverArt: cover_art ?? null });
    this.applyAlbumUpdate(updated);
    return updated;
  }

  async saveArtist(id: number, name?: string, profile_image?: string, banner_image?: string): Promise<Artist> {
    const updated = await invoke<Artist>("update_artist", {
      id,
      name: name ?? null,
      profileImage: profile_image ?? null,
      bannerImage: banner_image ?? null,
    });
    this.applyArtistUpdate(updated);
    return updated;
  }

  async savePlaylist(id: number, name?: string, cover_art?: string): Promise<Playlist & { coverArts: string[] }> {
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
    this.applyNewPlaylist(created);
    return created;
  }

  async deletePlaylist(id: number): Promise<void> {
    await invoke("delete_playlist", { playlistId: id });
    this.applyRemovePlaylist(id);
  }

  async addTrackToPlaylist(trackId: number, playlistId: number): Promise<void> {
    await invoke("add_track_to_playlist", { trackId, playlistId });
    const existing = this.playlistTrackIds.get(playlistId) || [];
    this.playlistTrackIds.set(playlistId, [...existing, trackId]);
    this.playlistTrackIds = new Map(this.playlistTrackIds);
  }

  async removeTrackFromPlaylist(trackId: number, playlistId: number): Promise<void> {
    await invoke("remove_track_from_playlist", { trackId, playlistId });
    const existing = this.playlistTrackIds.get(playlistId) || [];
    this.playlistTrackIds.set(playlistId, existing.filter(id => id !== trackId));
    this.playlistTrackIds = new Map(this.playlistTrackIds);
  }

  async getTrackPlaylistIds(trackId: number): Promise<number[]> {
    return invoke<number[]>("get_track_playlist_ids", { trackId });
  }
}

export const store = new LibraryStore();
