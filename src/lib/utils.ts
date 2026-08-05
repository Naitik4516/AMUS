import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import type { SortBy, Track } from "$lib/types";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export type WithoutChild<T> = T extends { child?: any } ? Omit<T, "child"> : T;
export type WithoutChildren<T> = T extends { children?: any } ? Omit<T, "children"> : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & {
  ref?: U | null;
};

export function formatDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
}

export function formatDurationColon(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  if (h > 0) return `${h}:${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
  return `${m}:${s.toString().padStart(2, "0")}`;
}

export function formatDurationShort(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`;
}

export function sumDuration(tracks: { duration_seconds: number }[]): number {
  return tracks.reduce((sum, track) => sum + track.duration_seconds, 0);
}

export function formatPercentage(value: number): string {
  return `${value.toFixed(1)}%`;
}

export function toLocalDateKey(date: Date): string {
  const y = date.getFullYear();
  const m = `${date.getMonth() + 1}`.padStart(2, "0");
  const d = `${date.getDate()}`.padStart(2, "0");
  return `${y}-${m}-${d}`;
}

export function formatDateShort(iso: string): string {
  const d = new Date(iso);
  return d.toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function sortTracks(tracks: Track[], sortBy: SortBy): Track[] {
  const sortedTracks = [...tracks];
  switch (sortBy) {
    case "title":
      sortedTracks.sort((a, b) => a.title.localeCompare(b.title));
      break;
    case "artist":
      sortedTracks.sort((a, b) => a.artists[0]?.name.localeCompare(b.artists[0]?.name || ""));
      break;
    case "album":
      sortedTracks.sort((a, b) => a.album.name.localeCompare(b.album.name));
      break;
    case "duration":
      sortedTracks.sort((a, b) => a.duration_seconds - b.duration_seconds);
      break;
    case "recently_added":
      sortedTracks.sort((a, b) => new Date(b.added_at).getTime() - new Date(a.added_at).getTime());
      break;
  }
  return sortedTracks;
}

export type CollectionSortField =
  | "name"
  | "added_at"
  | "last_played_at"
  | "total_plays"
  | "track_count"
  | "year";

export type CollectionSortDir = "asc" | "desc";

export interface CollectionSortPref {
  field: CollectionSortField;
  dir: CollectionSortDir;
}

export function loadSortPref(key: string, fallback: CollectionSortPref): CollectionSortPref {
  try {
    const raw = localStorage.getItem(`amus.sortPrefs.${key}`);
    if (raw) {
      const parsed = JSON.parse(raw) as CollectionSortPref;
      if (
        parsed &&
        typeof parsed.field === "string" &&
        (parsed.dir === "asc" || parsed.dir === "desc")
      ) {
        return parsed;
      }
    }
  } catch {
    // ignore malformed prefs
  }
  return fallback;
}

export function saveSortPref(key: string, pref: CollectionSortPref): void {
  try {
    localStorage.setItem(`amus.sortPrefs.${key}`, JSON.stringify(pref));
  } catch {
    // storage unavailable; sort still works for this session
  }
}

interface SortableCollection {
  name: string;
  added_at?: string;
  last_played_at?: string;
  total_plays?: number;
  track_count?: number;
  year?: number;
}

/** Sort collections (albums/artists/playlists/genres) by the given field. */
export function sortCollectionItems<T extends SortableCollection>(
  items: T[],
  field: CollectionSortField,
  dir: CollectionSortDir,
): T[] {
  const factor = dir === "asc" ? 1 : -1;
  const itemsToSort = [...items];
  itemsToSort.sort((a, b) => {
    let cmp = 0;
    if (field === "name") {
      cmp = a.name.localeCompare(b.name, undefined, { sensitivity: "base" }) * factor;
    } else {
      cmp = compareSortField(a[field], b[field], factor);
    }
    return cmp === 0 ? a.name.localeCompare(b.name, undefined, { sensitivity: "base" }) : cmp;
  });
  return itemsToSort;
}

/** Values that are missing/null sort last regardless of direction. */
function compareSortField(
  a: string | number | undefined,
  b: string | number | undefined,
  factor: number,
): number {
  if (a === undefined && b === undefined) return 0;
  if (a === undefined) return 1;
  if (b === undefined) return -1;
  if (typeof a === "string" && typeof b === "string") {
    return (new Date(a).getTime() - new Date(b).getTime()) * factor;
  }
  return ((a as number) - (b as number)) * factor;
}
