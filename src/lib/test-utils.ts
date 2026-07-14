import type { Track, PlaybackSource, Album, Artist } from "./types";

export function mockTrack(id: number, overrides?: Partial<Track>): Track {
  return {
    id,
    title: `Track ${id}`,
    duration_seconds: 200,
    is_favorite: false,
    added_at: "2024-01-01T00:00:00Z",
    track_number: 1,
    playlist_ids: [],
    cover_art: undefined,
    artists: [{ id: 1, name: "Artist 1" }],
    album: { id: 1, name: "Album 1" },
    ...overrides,
  };
}

export function mockAlbum(id: number, overrides?: Partial<Album>): Album {
  return { id, name: `Album ${id}`, ...overrides };
}

export function mockArtist(id: number, overrides?: Partial<Artist>): Artist {
  return { id, name: `Artist ${id}`, ...overrides };
}

export function mockSource(type: PlaybackSource["type"] = "Direct", id?: number): PlaybackSource {
  if (type === "Album" && id) return { type, id };
  if (type === "Playlist" && id) return { type, id };
  if (type === "Artist" && id) return { type, id };
  return { type };
}
