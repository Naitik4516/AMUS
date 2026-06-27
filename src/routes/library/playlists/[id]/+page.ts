import type { PageLoad } from "./$types";
import { invoke } from "@tauri-apps/api/core";
import type { Track, SortBy } from "$lib/types";

export const load: PageLoad = async ({ params, url }) => {
  const sortBy = (url.searchParams.get("sortBy") as SortBy) || "title";
  const id = Number(params.id);
  const name = url.searchParams.get("name") || "All Tracks";

  try {
    if (id === 0) {
      const data = await invoke("get_all_tracks", {
        sortBy: sortBy,
      });
      return { data: data || [], name, coverArtFilename: null };
    }

    const [data, playlist] = await Promise.all([
      invoke<Track[]>("get_tracks_by_playlist", {
        playlistId: id,
        sortBy: sortBy,
      }),
      invoke<[number, string, string | null]>("get_playlist", {
        playlistId: id,
      }).catch(() => null),
    ]);

    return {
      data: data || [],
      name: playlist?.[1] ?? name,
      coverArtFilename: playlist?.[2] ?? null,
    };
  } catch (e) {
    console.error("Failed to load playlist", e);
    return { data: [], name, coverArtFilename: null };
  }
};
