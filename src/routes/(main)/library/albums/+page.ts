import type { PageLoad } from "./$types";
import { invoke } from "@tauri-apps/api/core";
import type { Album } from "$lib/types";

export const load: PageLoad = async () => {
  try {
    const albums = await invoke<Album[]>("get_all_albums");
    return { albums };
  } catch (e) {
    console.error("Failed to load albums", e);
    return { albums: [] };
  }
};
