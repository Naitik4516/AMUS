import { describe, it, expect } from "vitest";
import {
  cn,
  formatDuration,
  formatDurationColon,
  formatDurationShort,
  formatBytes,
  formatPercentage,
  sortTracks,
} from "./utils";

describe("cn", () => {
  it("merges class names", () => {
    expect(cn("foo", "bar")).toBe("foo bar");
  });

  it("handles conditional classes", () => {
    expect(cn("base", false && "hidden", "visible")).toBe("base visible");
  });

  it("returns empty string for no inputs", () => {
    expect(cn()).toBe("");
  });
});

describe("formatDuration", () => {
  it("formats seconds only", () => {
    expect(formatDuration(45)).toBe("45s");
  });

  it("formats minutes and seconds", () => {
    expect(formatDuration(125)).toBe("2m 5s");
  });

  it("formats hours and minutes", () => {
    expect(formatDuration(3661)).toBe("1h 1m");
  });

  it("formats exactly one hour", () => {
    expect(formatDuration(3600)).toBe("1h 0m");
  });

  it("handles zero", () => {
    expect(formatDuration(0)).toBe("0s");
  });
});

describe("formatDurationColon", () => {
  it("formats seconds with leading zero", () => {
    expect(formatDurationColon(65)).toBe("1:05");
  });

  it("formats with hours", () => {
    expect(formatDurationColon(3661)).toBe("1:01:01");
  });

  it("handles zero", () => {
    expect(formatDurationColon(0)).toBe("0:00");
  });
});

describe("formatDurationShort", () => {
  it("formats minutes", () => {
    expect(formatDurationShort(125)).toBe("2m");
  });

  it("formats hours and minutes", () => {
    expect(formatDurationShort(3661)).toBe("1h 1m");
  });

  it("handles zero", () => {
    expect(formatDurationShort(0)).toBe("0m");
  });
});

describe("formatBytes", () => {
  it("formats bytes", () => {
    expect(formatBytes(500)).toBe("500 B");
  });

  it("formats KB", () => {
    expect(formatBytes(2048)).toBe("2.0 KB");
  });

  it("formats MB", () => {
    expect(formatBytes(5 * 1024 * 1024)).toBe("5.0 MB");
  });

  it("formats GB", () => {
    expect(formatBytes(2 * 1024 * 1024 * 1024)).toBe("2.00 GB");
  });
});

describe("formatPercentage", () => {
  it("formats with one decimal", () => {
    expect(formatPercentage(75.5)).toBe("75.5%");
  });

  it("handles zero", () => {
    expect(formatPercentage(0)).toBe("0.0%");
  });
});

describe("sortTracks", () => {
  const tracks = [
    {
      id: 1,
      title: "Beta",
      artists: [{ id: 1, name: "Artist B" }],
      album: { id: 1, name: "Album Z" },
      duration_seconds: 300,
      is_favorite: false,
      added_at: "2024-01-01T00:00:00Z",
      cover_art: undefined,
      track_number: 1,
      playlist_ids: [],
    },
    {
      id: 2,
      title: "Alpha",
      artists: [{ id: 2, name: "Artist A" }],
      album: { id: 2, name: "Album A" },
      duration_seconds: 100,
      is_favorite: false,
      added_at: "2024-06-01T00:00:00Z",
      cover_art: undefined,
      track_number: 2,
      playlist_ids: [],
    },
    {
      id: 3,
      title: "Gamma",
      artists: [{ id: 3, name: "Artist C" }],
      album: { id: 3, name: "Album M" },
      duration_seconds: 600,
      is_favorite: false,
      added_at: "2024-03-01T00:00:00Z",
      cover_art: undefined,
      track_number: 3,
      playlist_ids: [],
    },
  ];

  it("sorts by title", () => {
    const sorted = sortTracks(tracks, "title");
    expect(sorted[0].title).toBe("Alpha");
    expect(sorted[1].title).toBe("Beta");
    expect(sorted[2].title).toBe("Gamma");
  });

  it("sorts by artist", () => {
    const sorted = sortTracks(tracks, "artist");
    expect(sorted[0].artists[0].name).toBe("Artist A");
    expect(sorted[1].artists[0].name).toBe("Artist B");
    expect(sorted[2].artists[0].name).toBe("Artist C");
  });

  it("sorts by album", () => {
    const sorted = sortTracks(tracks, "album");
    expect(sorted[0].album.name).toBe("Album A");
    expect(sorted[1].album.name).toBe("Album M");
    expect(sorted[2].album.name).toBe("Album Z");
  });

  it("sorts by duration", () => {
    const sorted = sortTracks(tracks, "duration");
    expect(sorted[0].duration_seconds).toBe(100);
    expect(sorted[1].duration_seconds).toBe(300);
    expect(sorted[2].duration_seconds).toBe(600);
  });

  it("sorts by recently added", () => {
    const sorted = sortTracks(tracks, "recently_added");
    expect(sorted[0].added_at).toBe("2024-06-01T00:00:00Z");
    expect(sorted[1].added_at).toBe("2024-03-01T00:00:00Z");
    expect(sorted[2].added_at).toBe("2024-01-01T00:00:00Z");
  });

  it("does not mutate original array", () => {
    const original = [...tracks];
    sortTracks(tracks, "title");
    expect(tracks).toEqual(original);
  });
});
