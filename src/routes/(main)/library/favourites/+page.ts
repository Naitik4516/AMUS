import type { PageLoad } from "./$types";
import { invoke } from "@tauri-apps/api/core";
import type { Track } from "$lib/types";

export const load: PageLoad = async () => {
  try {
    const tracks = await invoke<Track[]>("get_favorite_tracks");
    return { tracks };
  } catch (e) {
    console.error("Failed to load albums", e);
    return { tracks: [] };
  }
};
