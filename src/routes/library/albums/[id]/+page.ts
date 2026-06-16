import type { PageLoad } from "./$types";
import { invoke } from "@tauri-apps/api/core";
import type { Track, SortBy, Album } from "$lib/types";

export const load: PageLoad = async ({ params, url }) => {
  const sortBy = (url.searchParams.get("sortBy") as SortBy) || "title";
  const id = Number(params.id);

  try {
    const [album, tracks] = await Promise.all([
      invoke<Album>("get_album", { id }),
      invoke<Track[]>("get_tracks_by_album", {
        album_id: id,
        sort_by: sortBy,
      }),
    ]);

    return {
      data: tracks || [],
      name: album.name || "Album",
      cover_art: album.cover_art || null,
    };
  } catch (e) {
    console.error("Failed to load album", e);
    return {
      data: [],
      name: "Album",
      cover_art: null,
    };
  }
};
