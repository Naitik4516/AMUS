import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import type { SortBy, Track } from "$lib/types";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, "child"> : T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
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

export function formatPercentage(value: number): string {
  return `${value.toFixed(1)}%`;
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
