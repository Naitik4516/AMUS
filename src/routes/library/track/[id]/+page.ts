import type { PageLoad } from "./$types";
import type { TrackDetails } from "$lib/types";
import { invoke } from "@tauri-apps/api/core";

export const load: PageLoad = async ({ params, depends }) => {
  const id = Number(params.id);
  const result = await invoke<TrackDetails>("get_track_details", { id });

  depends("app:track-details");
  return {
    trackDetails: result,
  };
};
