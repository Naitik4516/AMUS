import { invoke } from "@tauri-apps/api/core";
import { getImageUrl } from "./utils";
import type { Album, Track, Artist } from "./types";

export async function getPlaylistCoverArt(playlistId: number) {
  try {
    const coverArt = await invoke<string[]>("get_playlist_cover_arts", {
      playlistId,
    });

    return await Promise.all(coverArt.map((art) => getImageUrl(art)));
  } catch (error) {
    console.error("Error fetching playlist cover art:", error);
    return null;
  }
}

export async function getPlaylists() {
  try {
    const playlists =
      await invoke<{ id: number; name: string }[]>("get_playlists");

    const coverArts = await Promise.all(
      playlists.map(async (p: { id: number }) => {
        return await invoke<string[]>("get_playlist_cover_arts", {
          playlistId: p.id,
        });
      }),
    );

    return {
      playlists: playlists.map((p, i) => ({
        ...p,
        coverArts: coverArts[i],
      })),
    };
  } catch (error) {
    console.error("Failed to load playlists:", error);
    return { playlists: [] };
  }
}

export async function getAllAlbums(): Promise<Album[]> {
  try {
    const albums = await invoke<Album[]>("get_all_albums");
    return albums || [];
  } catch (error) {
    console.error("Failed to load albums:", error);
    return [];
  }
}

export async function getArtists(): Promise<Artist[]> {
  try {
    const artists = await invoke<Artist[]>("get_artists");
    return artists || [];
  } catch (error) {
    console.error("Failed to load artists:", error);
    return [];
  }
}

export async function getArtistDetails(artistId: number) {
  try {
    const [artist, tracks, albums] = await Promise.all([
      invoke<{
        id: number;
        name: string;
        profile_image: string | null;
        banner_image: string | null;
      }>("get_artist", { id: artistId }),
      invoke<Track[]>("get_tracks_by_artist", {
        artistId: artistId,
        sort_by: "title",
      }),
      invoke<Album[]>("get_albums", { artistId: artistId }),
    ]);

    return {
      artist,
      tracks: tracks || [],
      albums: albums || [],
    };
  } catch (error) {
    console.error("Failed to load artist details:", error);
    return null;
  }
}

export async function fetchArtistImages(artistId: number): Promise<void> {
  try {
    await invoke("fetch_artist_images", { artistId });
  } catch (error) {
    console.error("Failed to fetch artist images:", error);
  }
}
