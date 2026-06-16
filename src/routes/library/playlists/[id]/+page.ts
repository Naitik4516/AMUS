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
      return { data: data || [], name };
    }

    const data = await invoke("get_tracks_by_playlist", {
      playlistId: id,
      sortBy: sortBy,
    });

    return { data: data || [], name };
  } catch (e) {
    console.error("Failed to load playlist", e);
    return { data: [], name };
  }
};
