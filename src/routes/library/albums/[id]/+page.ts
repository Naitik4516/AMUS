import type { PageLoad } from "./$types";
import { invoke } from "@tauri-apps/api/core";
import type { Track, SortBy, Album } from "$lib/types";
import { getImageUrl } from "$lib/utils";

export const load: PageLoad = async ({ params, url }) => {
  const sortBy = (url.searchParams.get("sortBy") as SortBy) || "title";
  const id = Number(params.id);

  try {
    const [album, tracks] = await Promise.all([
      invoke<Album>("get_album", { id }),
      invoke<Track[]>("get_tracks_by_album", {
        albumId: id,
        sort_by: sortBy,
      }),
    ]);

    const coverArtUrl = album.cover_art
      ? await getImageUrl(album.cover_art)
      : null;

    return {
      data: tracks || [],
      name: album.name || "Album",
      cover_art: coverArtUrl,
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
