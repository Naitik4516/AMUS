import type { PageLoad } from "./$types";
import { getArtistDetails } from "$lib/data.svelte";

export const load: PageLoad = async ({ params }) => {
  const id = Number(params.id);
  const result = await getArtistDetails(id);

  if (!result) {
    return {
      artist: { id, name: "Unknown Artist", profile_picture: null },
      tracks: [],
      albums: [],
    };
  }

  return {
    artist: result.artist,
    tracks: result.tracks,
    albums: result.albums,
  };
};
