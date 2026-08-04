import type { PageLoad } from "./$types";
import type { TrackDetails } from "$lib/types";
import { invoke } from "@tauri-apps/api/core";
import { error } from "@sveltejs/kit";

export const load: PageLoad = async ({ params, depends }) => {
  depends("app:track-details");

  const id = Number(params.id);
  if (!Number.isInteger(id) || id <= 0) {
    error(404, "Track not found");
  }

  try {
    const result = await invoke<TrackDetails>("get_track_details", { id });
    return {
      trackDetails: result,
    };
  } catch (e) {
    console.error("Failed to load track details:", e);
    error(500, "Failed to load track details");
  }
};
