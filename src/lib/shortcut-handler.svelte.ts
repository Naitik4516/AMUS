import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { goto } from "$app/navigation";
import { player } from "./player.svelte";
import { handlerMap } from "./shortcuts.svelte";
import { scanLibrary } from "./commands.svelte";
import Clock_1 from "@lucide/svelte/icons/clock-1";

export const ui = $state({ queueVisible: false });
export function toggleQueue() {
  ui.queueVisible = !ui.queueVisible;
}

export const shortcutSettings = $state({ open: false });

export function installHandlers() {
  const seekSec = (sec: number) => {
    if (!player.currentTrack) return;
    const dur = player.currentTrack.duration_seconds;
    const currentSec = player.position;
    const newSec = Math.max(0, Math.min(dur, currentSec + sec));
    player.seek(newSec);
  };

  handlerMap.set("play_pause", () => player.playPause());
  handlerMap.set("next_track", () => player.next());
  handlerMap.set("prev_track", () => player.previous());
  handlerMap.set("stop", () => {
    if (player.isPlaying) player.playPause();
  });
  handlerMap.set("restart_track", () => {
    if (player.currentTrack) player.seek(0);
  });

  handlerMap.set("seek_forward_5", () => seekSec(5));
  handlerMap.set("seek_backward_5", () => seekSec(-5));
  handlerMap.set("seek_forward_10", () => seekSec(10));
  handlerMap.set("seek_backward_10", () => seekSec(-10));

  handlerMap.set("volume_up", () => {
    player.setVolume(Math.min(1, player.volume + 0.05));
  });
  handlerMap.set("volume_down", () => {
    player.setVolume(Math.max(0, player.volume - 0.05));
  });
  handlerMap.set("mute", () => {
    player.setVolume(player.volume > 0 ? 0 : 1);
  });

  handlerMap.set("focus_search", () => {
    const el = document.getElementById("global-search-input");
    el?.focus();
  });
  handlerMap.set("refresh_page", () => {
    window.location.reload();
  });
  handlerMap.set("rescan_music", () => {
    scanLibrary();
  });

  handlerMap.set("toggle_shuffle", () => player.toggleShuffle());
  handlerMap.set("toggle_repeat", () => player.cycleRepeat());
  handlerMap.set("toggle_queue", () => toggleQueue());
  handlerMap.set("clear_queue", () => player.clearQueue());

  handlerMap.set("navigate_home", () => goto("/"));
  handlerMap.set("navigate_library", () => goto("/library"));
  handlerMap.set("navigate_playlists", () => goto("/library/playlists"));
  handlerMap.set("navigate_albums", () => goto("/library/albums"));
  handlerMap.set("navigate_artists", () => goto("/library/artists"));
  handlerMap.set("navigate_settings", () => goto("/settings"));
  handlerMap.set("go_back", () => history.back());
  handlerMap.set("go_forward", () => history.forward());

  handlerMap.set("minimize", () => {
    getCurrentWindow().minimize();
  });
  handlerMap.set("close_window", () => {
    getCurrentWindow().close();
  });
  handlerMap.set("quit_amus", () => {
    invoke("quit_app");
  });

  // Global shortcut handlers
  handlerMap.set("global_play_pause", () => player.playPause());
  handlerMap.set("global_next_track", () => player.next());
  handlerMap.set("global_prev_track", () => player.previous());
  handlerMap.set("global_stop", () => {
    if (player.isPlaying) player.playPause();
  });
  handlerMap.set("global_volume_up", () => {
    player.setVolume(Math.min(1, player.volume + 0.05));
  });
  handlerMap.set("global_volume_down", () => {
    player.setVolume(Math.max(0, player.volume - 0.05));
  });
  handlerMap.set("global_toggle_mute", () => {
    player.setVolume(player.volume > 0 ? 0 : 1);
  });
  handlerMap.set("global_seek_forward", () => {
    if (player.currentTrack) seekSec(5);
  });
  handlerMap.set("global_seek_backward", () => {
    if (player.currentTrack) seekSec(-5);
  });
  handlerMap.set("global_toggle_shuffle", () => player.toggleShuffle());
  handlerMap.set("global_toggle_repeat", () => player.cycleRepeat());
  handlerMap.set("global_show_hide", () => {
    const w = getCurrentWindow();
    w.isVisible().then((visible) => {
      visible ? w.hide() : w.show();
    });
  });
  handlerMap.set("global_show_miniplayer", async () => {
    await invoke("toggle_mini_player");
  });
}
