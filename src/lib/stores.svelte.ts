import { initSettings } from "$lib/settings.svelte";
import type { Album, Artist, Genre, Playlist, Track } from "$lib/types";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { appDataDir } from "@tauri-apps/api/path";
import { player } from "$lib/player.svelte";
import * as commands from "$lib/commands.svelte";

class LibraryStore {
  tracks = $state<Track[]>([]);
  albums = $state<Album[]>([]);
  artists = $state<Artist[]>([]);
  playlists = $state<Playlist[]>([]);
  genres = $state<Genre[]>([]);
  ready = $state(false);
  loading = $state(false);
  error = $state<string | null>(null);

  appDataDirPath = $state<string | null>(null);
  #coverBaseUrl: string | null = null;
  #artistBaseUrl: string | null = null;
  #baseUrlDir: string | null = null;
  #appDataDirPromise: Promise<string> | null = null;
  #unlistenLibraryUpdate: UnlistenFn | null = null;

  tracksById = new Map<number, Track>();
  albumsById = new Map<number, Album>();
  artistsById = new Map<number, Artist>();
  playlistsById = new Map<number, Playlist>();
  genresById = new Map<number, Genre>();

  // Precomputed indexes (rebuilt in #preprocess / on mutations).
  // Plain objects keyed by id: `$state` proxies object property access,
  // so mutations and fresh-array assignment notify `$derived` consumers.
  tracksByPlaylistId = $state<Record<number, Track[]>>({});
  tracksByGenreId = $state<Record<number, Track[]>>({});
  tracksByAlbumId = $state<Record<number, Track[]>>({});
  tracksByArtistId = $state<Record<number, Track[]>>({});
  albumsByArtistId = $state<Record<number, Album[]>>({});

  // Playlist insertion order (track ids, playlist position order) from the DB,
  // used by #rebuildIndexes so playlist views keep "latest added last".
  #playlistTrackOrder = new Map<number, number[]>();

  // Derived data
  favoriteTracks = $derived(this.tracks.filter((t) => t.is_favorite));

  recentlyAddedTracks = $derived(
    [...this.tracks].sort(
      (a, b) => new Date(b.added_at).getTime() - new Date(a.added_at).getTime(),
    ),
  );

  tracksByAlbum(albumId: number): Track[] {
    this.#ensureIndexes();
    return this.tracksByAlbumId[albumId] ?? [];
  }

  tracksByArtist(artistId: number): Track[] {
    this.#ensureIndexes();
    return this.tracksByArtistId[artistId] ?? [];
  }

  tracksByPlaylist(playlistId: number): Track[] {
    this.#ensureIndexes();
    return this.tracksByPlaylistId[playlistId] ?? [];
  }

  tracksByGenre(genreId: number): Track[] {
    this.#ensureIndexes();
    return this.tracksByGenreId[genreId] ?? [];
  }

  albumsByArtist(artistId: number): Album[] {
    this.#ensureIndexes();
    return this.albumsByArtistId[artistId] ?? [];
  }

  #ensureIndexes() {
    if (this.tracks.length > 0 && Object.keys(this.tracksByAlbumId).length === 0) {
      this.#rebuildIndexes();
    }
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
    type: "cover" | "artist" = "cover",
  ): string | null {
    if (!filename || !this.appDataDirPath) {
      return null;
    }
    if (this.#baseUrlDir !== this.appDataDirPath) {
      this.#baseUrlDir = this.appDataDirPath;
      this.#coverBaseUrl = `${convertFileSrc(this.appDataDirPath)}/covers/`;
      this.#artistBaseUrl = `${convertFileSrc(this.appDataDirPath)}/artists/`;
    }
    const base = type === "artist" ? this.#artistBaseUrl : this.#coverBaseUrl;
    return `${base}${filename}`;
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
        this.#loadGenres(),
      ]);
      this.#preprocess();
      await this.#loadPlaylistTrackOrder();
      this.#rebuildIndexes();

      this.ready = true;
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load library";
      console.error("LibraryStore.init failed:", e);
    } finally {
      this.loading = false;
    }

    this.#unlistenLibraryUpdate?.();
    this.#unlistenLibraryUpdate = await listen("library-updated", async () => {
      await this.#reloadAll();
    });
  }

  async #reloadAll() {
    try {
      await Promise.all([
        this.#loadTracks(),
        this.#loadAlbums(),
        this.#loadArtists(),
        this.#loadPlaylists(),
        this.#loadGenres(),
      ]);
      this.#preprocess();
      await this.#loadPlaylistTrackOrder();
      this.#rebuildIndexes();
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
    const result = await invoke<Playlist[]>("get_playlists");
    this.playlists = result || [];
  }

  async #loadGenres() {
    const result = await commands.getGenres();
    this.genres = result || [];
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

    this.genresById = new Map();
    for (const genre of this.genres) {
      this.genresById.set(genre.id, genre);
    }

    this.#rebuildIndexes();
  }

  #rebuildIndexes() {
    this.tracksByPlaylistId = {};
    this.tracksByGenreId = {};
    this.tracksByAlbumId = {};
    this.tracksByArtistId = {};
    this.albumsByArtistId = {};

    const byAlbum = this.tracksByAlbumId;
    const byArtist = this.tracksByArtistId;
    const byPlaylist = this.tracksByPlaylistId;
    const byGenre = this.tracksByGenreId;

    for (const track of this.tracks) {
      let albumTracks = byAlbum[track.album.id];
      if (!albumTracks) {
        albumTracks = [];
        byAlbum[track.album.id] = albumTracks;
      }
      albumTracks.push(track);

      for (const artist of track.artists) {
        let artistTracks = byArtist[artist.id];
        if (!artistTracks) {
          artistTracks = [];
          byArtist[artist.id] = artistTracks;
        }
        artistTracks.push(track);
      }

      for (const genreId of track.genre_ids ?? []) {
        let genreTracks = byGenre[genreId];
        if (!genreTracks) {
          genreTracks = [];
          byGenre[genreId] = genreTracks;
        }
        genreTracks.push(track);
      }
    }

    // Playlist tracks keep DB insertion order (latest added last); tracks not
    // present in the cached order (e.g. just added) are appended at the end.
    for (const track of this.tracks) {
      for (const playlistId of track.playlist_ids) {
        let playlistTracks = byPlaylist[playlistId];
        if (!playlistTracks) {
          const orderedIds = this.#playlistTrackOrder.get(playlistId) ?? [];
          playlistTracks = orderedIds
            .map((id) => this.tracksById.get(id))
            .filter((t): t is Track => t !== undefined);
          byPlaylist[playlistId] = playlistTracks;
        }
        if (!playlistTracks.some((t) => t.id === track.id)) {
          playlistTracks.push(track);
        }
      }
    }

    for (const [artistId, tracks] of Object.entries(this.tracksByArtistId)) {
      const albumIds = new Set(tracks.map((t) => t.album.id));
      this.albumsByArtistId[Number(artistId)] = this.albums.filter((a) => albumIds.has(a.id));
    }
  }

  async #loadPlaylistTrackOrder() {
    try {
      const ordered = new Map<number, number[]>();
      for (const playlist of this.playlists) {
        const tracks = await invoke<Track[]>("get_tracks_by_playlist", {
          playlistId: playlist.id,
        });
        ordered.set(
          playlist.id,
          (tracks ?? []).map((t) => t.id),
        );
      }
      this.#playlistTrackOrder = ordered;
    } catch (e) {
      console.error("Failed to load playlist track order:", e);
    }
  }

  async reloadArtists() {
    await this.#loadArtists();
    for (const artist of this.artists) {
      this.artistsById.set(artist.id, artist);
    }
    this.#rebuildIndexes();
  }

  applyTrackUpdate(track: Track) {
    const idx = this.tracks.findIndex((t) => t.id === track.id);
    if (idx !== -1) {
      this.tracks[idx] = track;
    }
    this.tracksById.set(track.id, track);
    this.#rebuildIndexes();
  }

  applyAlbumUpdate(album: Album) {
    const idx = this.albums.findIndex((a) => a.id === album.id);
    if (idx !== -1) {
      this.albums[idx] = album;
    }
    this.albumsById.set(album.id, album);
    this.#rebuildIndexes();
  }

  applyArtistUpdate(artist: Artist) {
    const idx = this.artists.findIndex((a) => a.id === artist.id);
    if (idx !== -1) {
      this.artists[idx] = artist;
    }
    this.artistsById.set(artist.id, artist);
    this.#rebuildIndexes();
  }

  applyPlaylistUpdate(playlist: Playlist) {
    const idx = this.playlists.findIndex((p) => p.id === playlist.id);
    if (idx !== -1) {
      this.playlists[idx] = playlist;
    } else {
      this.playlists = [...this.playlists, playlist];
    }
    this.playlistsById.set(playlist.id, playlist);
    this.#rebuildIndexes();
  }

  async toggleFavorite(trackId: number): Promise<Track> {
    const updated = await invoke<Track>("toggle_favorite", { id: trackId });
    this.applyTrackUpdate(updated);
    return updated;
  }

  async saveAlbum(id: number, name: string, cover_art?: string | null): Promise<Album> {
    if (name.trim().length === 0) {
      throw new Error("Album name cannot be empty");
    }
    const updated = await invoke<Album>("update_album", {
      id,
      name,
      ...(cover_art !== undefined ? { cover_art: cover_art ?? "" } : {}),
    });
    this.applyAlbumUpdate(updated);
    return updated;
  }

  async saveArtist(
    id: number,
    name: string,
    profile_image?: string | null,
    banner_image?: string | null,
  ): Promise<Artist> {
    if (name.trim().length === 0) {
      throw new Error("Artist name cannot be empty");
    }
    const updated = await invoke<Artist>("update_artist", {
      artist: {
        id,
        name,
        profile_image,
        banner_image,
      },
    });
    this.applyArtistUpdate(updated);
    return updated;
  }

  async savePlaylist(
    id: number,
    name?: string | null,
    cover_art?: string | null,
  ): Promise<Playlist> {
    const updated = await invoke<Playlist>("update_playlist", {
      playlist: { id, name, cover_art },
    });
    this.applyPlaylistUpdate(updated);
    return updated;
  }

  async saveGenre(id: number, name: string, thumbnail?: string | null): Promise<Genre> {
    const updated = await commands.updateGenre(id, name, thumbnail);
    this.genresById = new Map(this.genresById).set(updated.id, updated);
    const idx = this.genres.findIndex((g) => g.id === id);
    if (idx !== -1) {
      this.genres[idx] = updated;
    }
    return updated;
  }

  async updateTrackMetadata(
    trackId: number,
    title: string,
    year?: number | null,
    genre?: string | null,
  ): Promise<Track> {
    if (title.trim().length === 0) {
      throw new Error("Track title cannot be empty");
    }
    const updated = await invoke<Track>("update_track_metadata", {
      id: trackId,
      title: title,
      year: year ?? null,
      genre: genre ?? null,
    });
    this.applyTrackUpdate(updated);
    return updated;
  }

  async createPlaylist(name: string): Promise<Playlist> {
    const created = await invoke<Playlist>("create_playlist", { name });
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
      const playlistTracks = this.tracksByPlaylistId[playlistId] ?? [];
      this.tracksByPlaylistId[playlistId] = [...playlistTracks, track];
      const order = this.#playlistTrackOrder.get(playlistId) ?? [];
      this.#playlistTrackOrder.set(playlistId, [...order, track.id]);
    }
  }

  async addTracksToPlaylist(trackIds: number[], playlistId: number): Promise<void> {
    const targets = this.tracks.filter(
      (t) => trackIds.includes(t.id) && !t.playlist_ids.includes(playlistId),
    );
    if (targets.length === 0) return;
    await Promise.all(
      targets.map((t) => invoke("add_track_to_playlist", { trackId: t.id, playlistId })),
    );
    const playlistTracks = this.tracksByPlaylistId[playlistId] ?? [];
    this.tracksByPlaylistId[playlistId] = [
      ...playlistTracks,
      ...targets.map((t) => {
        t.playlist_ids.push(playlistId);
        return t;
      }),
    ];
    const order = this.#playlistTrackOrder.get(playlistId) ?? [];
    this.#playlistTrackOrder.set(playlistId, [...order, ...targets.map((t) => t.id)]);
  }

  async removeTrackFromPlaylist(trackId: number, playlistId: number): Promise<void> {
    await invoke("remove_track_from_playlist", { trackId, playlistId });
    const track = this.tracks.find((t) => t.id === trackId);
    if (track) {
      track.playlist_ids = track.playlist_ids.filter((id) => id !== playlistId);
      this.tracks = this.tracks.map((t) => (t.id === track.id ? track : t));
    }
    const order = this.#playlistTrackOrder.get(playlistId);
    if (order) {
      this.#playlistTrackOrder.set(
        playlistId,
        order.filter((id) => id !== trackId),
      );
    }
    this.#rebuildIndexes();
  }

  async removeTracksFromPlaylist(trackIds: number[], playlistId: number): Promise<void> {
    const targets = this.tracks.filter(
      (t) => trackIds.includes(t.id) && t.playlist_ids.includes(playlistId),
    );
    if (targets.length === 0) return;
    await Promise.all(
      targets.map((t) => invoke("remove_track_from_playlist", { trackId: t.id, playlistId })),
    );
    for (const track of targets) {
      track.playlist_ids = track.playlist_ids.filter((id) => id !== playlistId);
      this.tracks = this.tracks.map((t) => (t.id === track.id ? track : t));
    }
    const removed = new Set(trackIds);
    const order = this.#playlistTrackOrder.get(playlistId);
    if (order) {
      this.#playlistTrackOrder.set(
        playlistId,
        order.filter((id) => !removed.has(id)),
      );
    }
    this.#rebuildIndexes();
  }

  async setTracksFavorite(trackIds: number[], favorite: boolean): Promise<void> {
    const targets = this.tracks.filter(
      (t) => trackIds.includes(t.id) && t.is_favorite !== favorite,
    );
    await Promise.all(
      targets.map((t) =>
        invoke<Track>("toggle_favorite", { id: t.id }).then((updated) =>
          this.applyTrackUpdate(updated),
        ),
      ),
    );
  }

  async deleteTracks(trackIds: number[]): Promise<void> {
    for (const id of trackIds) {
      await this.deleteTrack(id);
    }
  }

  async deleteTrack(trackId: number): Promise<void> {
    if (player.currentTrack?.id === trackId) {
      await player.next();
    }
    await commands.deleteTrack(trackId);
    this.tracks = this.tracks.filter((t) => t.id !== trackId);
    this.tracksById.delete(trackId);
    this.#rebuildIndexes();

    for (const t of player.userQueue.filter((t) => t.id === trackId)) {
      if (t.queue_id != null) {
        await player.removeFromQueue(t.queue_id, "user");
      }
    }
    if (player.playNext.some((t) => t.id === trackId)) {
      await player.removeFromQueue(trackId, "context");
    }
  }

  async getTrackPlaylistIds(trackId: number): Promise<number[]> {
    return invoke<number[]>("get_track_playlist_ids", { trackId });
  }
}

export const store = new LibraryStore();
