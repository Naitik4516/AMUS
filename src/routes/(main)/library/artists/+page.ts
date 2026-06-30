import type { PageLoad } from "./$types";
import { getArtists } from "$lib/data.svelte";

export const load: PageLoad = async () => {
  const artists = await getArtists();
  return { artists };
};
