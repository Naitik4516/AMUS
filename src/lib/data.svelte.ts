import { invoke } from "@tauri-apps/api/core";

export async function getPlaylists() {
  try {
    const playlists = await invoke<
      { id: number; name: string; cover_arts: string[]; coverArt?: string }[]
    >("get_playlists");
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

export async function fetchArtistImages(artistId: number): Promise<void> {
  try {
    await invoke("fetch_artist_images", { artistId });
  } catch (error) {
    console.error("Failed to fetch artist images:", error);
  }
}
