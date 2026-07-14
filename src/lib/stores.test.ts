import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import type { Track, Album, Artist, Playlist } from "$lib/types";
import { store } from "$lib/stores.svelte";

describe("LibraryStore derived data and methods", () => {
  beforeEach(() => {
    store.tracks = [
      {
        id: 1,
        title: "Song A",
        duration_seconds: 200,
        is_favorite: true,
        added_at: "2024-06-01T00:00:00Z",
        track_number: 1,
        playlist_ids: [10],
        artists: [{ id: 1, name: "Artist X" }],
        album: { id: 1, name: "Album 1" },
      },
      {
        id: 2,
        title: "Song B",
        duration_seconds: 300,
        is_favorite: false,
        added_at: "2024-05-01T00:00:00Z",
        track_number: 2,
        playlist_ids: [],
        artists: [
          { id: 1, name: "Artist X" },
          { id: 2, name: "Artist Y" },
        ],
        album: { id: 1, name: "Album 1" },
      },
      {
        id: 3,
        title: "Song C",
        duration_seconds: 150,
        is_favorite: true,
        added_at: "2024-07-01T00:00:00Z",
        track_number: 1,
        playlist_ids: [11],
        artists: [{ id: 2, name: "Artist Y" }],
        album: { id: 2, name: "Album 2" },
      },
    ];
    store.albums = [
      { id: 1, name: "Album 1" },
      { id: 2, name: "Album 2" },
    ];
    store.artists = [
      { id: 1, name: "Artist X" },
      { id: 2, name: "Artist Y" },
    ];
  });

  describe("favoriteTracks", () => {
    it("returns only favorited tracks", () => {
      const result = store.favoriteTracks;
      expect(result).toHaveLength(2);
      expect(result.every((t) => t.is_favorite)).toBe(true);
    });
  });

  describe("tracksByAlbum", () => {
    it("filters tracks by album id", () => {
      const result = store.tracksByAlbum(1);
      expect(result).toHaveLength(2);
      expect(result.every((t) => t.album.id === 1)).toBe(true);
    });

    it("returns empty when no tracks match", () => {
      const result = store.tracksByAlbum(999);
      expect(result).toHaveLength(0);
    });
  });

  describe("tracksByArtist", () => {
    it("filters tracks by artist id", () => {
      const result = store.tracksByArtist(1);
      expect(result).toHaveLength(2);
    });

    it("handles artists with multiple tracks", () => {
      const result = store.tracksByArtist(2);
      expect(result).toHaveLength(2);
    });
  });

  describe("tracksByPlaylist", () => {
    it("filters tracks by playlist id", () => {
      const result = store.tracksByPlaylist(10);
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe(1);
    });

    it("returns empty for nonexistent playlist", () => {
      const result = store.tracksByPlaylist(999);
      expect(result).toHaveLength(0);
    });
  });

  describe("albumsByArtist", () => {
    it("returns albums containing tracks by the artist", () => {
      const result = store.albumsByArtist(1);
      expect(result).toHaveLength(1);
      expect(result[0].name).toBe("Album 1");
    });

    it("includes all albums an artist appears on", () => {
      const result = store.albumsByArtist(2);
      expect(result).toHaveLength(2);
    });
  });

  describe("applyTrackUpdate", () => {
    it("replaces track at index when id matches", () => {
      const updated: Track = { ...store.tracks[0], title: "Song A Updated" };
      store.applyTrackUpdate(updated);
      expect(store.tracks.find((t) => t.id === 1)?.title).toBe("Song A Updated");
      expect(store.tracksById.get(1)?.title).toBe("Song A Updated");
    });
  });

  describe("recentlyAddedTracks", () => {
    it("sorts tracks by added_at descending", () => {
      const result = store.recentlyAddedTracks;
      expect(result[0].id).toBe(3);
      expect(result[1].id).toBe(1);
      expect(result[2].id).toBe(2);
    });
  });
});

// ---------------------------------------------------------------------------
// Integration-style tests using the singleton store + mocked invoke
// ---------------------------------------------------------------------------

const testTrack: Track = {
  id: 1,
  title: "Test",
  duration_seconds: 200,
  is_favorite: false,
  added_at: "2025-01-01T00:00:00Z",
  track_number: 1,
  playlist_ids: [],
  artists: [{ id: 1, name: "Artist" }],
  album: { id: 1, name: "Album" },
};

const testAlbum: Album = { id: 1, name: "Album" };
const testArtist: Artist = { id: 1, name: "Artist" };
const testPlaylist: Playlist & { coverArts: string[] } = { id: 1, name: "Playlist", coverArts: [] };
const invokeMock = vi.mocked(invoke);

beforeEach(() => {
  vi.clearAllMocks();
  store.tracks = [];
  store.albums = [];
  store.artists = [];
  store.playlists = [];
  store.ready = false;
  store.error = null;
});

describe("LibraryStore async operations", () => {
  describe("init", () => {
    it("loads data from Tauri and sets ready", async () => {
      invokeMock.mockImplementation((cmd: string) => {
        if (cmd === "get_all_tracks") return Promise.resolve([testTrack]);
        if (cmd === "get_all_albums") return Promise.resolve([testAlbum]);
        if (cmd === "get_artists") return Promise.resolve([testArtist]);
        if (cmd === "get_playlists") return Promise.resolve([testPlaylist]);
        return Promise.resolve();
      });

      await store.init();

      expect(store.tracks).toHaveLength(1);
      expect(store.albums).toHaveLength(1);
      expect(store.artists).toHaveLength(1);
      expect(store.playlists).toHaveLength(1);
      expect(store.ready).toBe(true);
      // Maps should be populated
      expect(store.tracksById.get(1)?.title).toBe("Test");
      expect(store.albumsById.get(1)?.name).toBe("Album");
    });

    it("sets error on failure", async () => {
      invokeMock.mockRejectedValue(new Error("db error"));
      await store.init();
      expect(store.error).toBe("db error");
      expect(store.ready).toBe(false);
    });
  });

  describe("toggleFavorite", () => {
    it("invokes toggle_favorite and updates state", async () => {
      store.tracks = [testTrack];
      store.tracksById.set(1, testTrack);
      const updated = { ...testTrack, is_favorite: true };
      invokeMock.mockResolvedValue(updated);

      const result = await store.toggleFavorite(1);

      expect(invokeMock).toHaveBeenCalledWith("toggle_favorite", { id: 1 });
      expect(result.is_favorite).toBe(true);
      expect(store.tracks[0].is_favorite).toBe(true);
    });
  });

  describe("saveAlbum", () => {
    it("updates album name and cover via invoke", async () => {
      store.albums = [testAlbum];
      store.albumsById.set(1, testAlbum);
      const updated = { ...testAlbum, name: "Updated Album" };
      invokeMock.mockResolvedValue(updated);

      const result = await store.saveAlbum(1, "Updated Album");
      expect(result.name).toBe("Updated Album");
      expect(store.albums[0].name).toBe("Updated Album");
    });
  });

  describe("saveArtist", () => {
    it("updates artist fields via invoke", async () => {
      store.artists = [testArtist];
      store.artistsById.set(1, testArtist);
      const updated = { ...testArtist, name: "Updated Artist" };
      invokeMock.mockResolvedValue(updated);

      const result = await store.saveArtist(1, "Updated Artist");
      expect(result.name).toBe("Updated Artist");
      expect(store.artists[0].name).toBe("Updated Artist");
    });
  });

  describe("createPlaylist", () => {
    it("invokes and updates playlistsById", async () => {
      const created = { id: 2, name: "New Playlist", coverArts: [] };
      invokeMock.mockResolvedValue(created);

      const result = await store.createPlaylist("New Playlist");
      expect(result.name).toBe("New Playlist");
      expect(store.playlistsById.get(2)?.name).toBe("New Playlist");
    });
  });

  describe("deletePlaylist", () => {
    it("invokes and removes from local state", async () => {
      const pl = { id: 1, name: "To Delete", coverArts: [] };
      store.playlists = [pl];
      store.playlistsById.set(1, pl);

      await store.deletePlaylist(1);
      expect(invokeMock).toHaveBeenCalledWith("delete_playlist", { playlistId: 1 });
      expect(store.playlists).toHaveLength(0);
      expect(store.playlistsById.has(1)).toBe(false);
    });
  });

  describe("addTrackToPlaylist", () => {
    it("appends playlist_id to track", async () => {
      store.tracks = [testTrack];
      await store.addTrackToPlaylist(1, 10);
      expect(store.tracks[0].playlist_ids).toContain(10);
    });
  });

  describe("removeTrackFromPlaylist", () => {
    it("removes playlist_id from track", async () => {
      store.tracks = [{ ...testTrack, playlist_ids: [10] }];
      await store.removeTrackFromPlaylist(1, 10);
      expect(store.tracks[0].playlist_ids).not.toContain(10);
    });
  });

  describe("getImageSrc", () => {
    it("returns null when filename is null", () => {
      expect(store.getImageSrc(null, "cover")).toBeNull();
    });

    it("returns null when appDataDirPath is not set", () => {
      store.appDataDirPath = null;
      expect(store.getImageSrc("art.jpg", "cover")).toBeNull();
    });

    it("passes correct cover path to convertFileSrc", () => {
      store.appDataDirPath = "/app/data";
      store.getImageSrc("art.jpg", "cover");
      expect(vi.mocked(convertFileSrc)).toHaveBeenCalledWith("/app/data/covers/art.jpg");
    });

    it("passes correct artist path to convertFileSrc", () => {
      store.appDataDirPath = "/app/data";
      store.getImageSrc("pic.jpg", "artist");
      expect(vi.mocked(convertFileSrc)).toHaveBeenCalledWith("/app/data/artists/pic.jpg");
    });

    it("passes correct banner path to convertFileSrc", () => {
      store.appDataDirPath = "/app/data";
      store.getImageSrc("banner.jpg", "banner");
      expect(vi.mocked(convertFileSrc)).toHaveBeenCalledWith("/app/data/artist_banner/banner.jpg");
    });
  });

  describe("ensureAppDataDir", () => {
    it("returns cached path after first call", async () => {
      store.appDataDirPath = null;
      const dir = await store.ensureAppDataDir();
      expect(dir).toBeTruthy();
      // Second call returns same value
      const dir2 = await store.ensureAppDataDir();
      expect(dir2).toBe(dir);
    });
  });
});
