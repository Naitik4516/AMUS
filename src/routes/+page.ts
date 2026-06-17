import { invoke } from "@tauri-apps/api/core";
import type { PageLoad } from "./$types";

export const load: PageLoad = async () => {
  try {
    const hasMusic = await invoke<boolean>("has_music");
    return { hasMusic };
  } catch (e) {
    console.error("Failed to check if library has music", e);
    return { hasMusic: false };
  }
};
