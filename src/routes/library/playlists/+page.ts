import type { PageLoad } from "./$types";
import { getPlaylists } from "$lib/data.svelte";
export const load: PageLoad = async () => {
  return await getPlaylists();
};
