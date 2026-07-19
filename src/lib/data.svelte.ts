import { invoke } from "@tauri-apps/api/core";

export async function getPlaylists() {
  try {
    const playlists =
      await invoke<{ id: number; name: string; cover_arts: string[]; coverArt?: string }[]>(
        "get_playlists",
      );
    return {
      playlists: playlists.map((p) => ({
        ...p,
        coverArts: p.cover_arts,
      })),
    };
  } catch (error) {
    console.error("Failed to load playlists:", error);
    return { playlists: [] };
  }
}
