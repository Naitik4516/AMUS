<script lang="ts">
  import type { PlaybackEvent } from "$lib/commands.svelte";
  import { getImageUrl } from "$lib/utils";

  let { events }: { events: PlaybackEvent[] } = $props();

  function formatTime(iso: string): string {
    const d = new Date(iso);
    return d.toLocaleDateString(undefined, { month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" });
  }
</script>

<div class="bg-card/50 backdrop-blur-lg border border-border rounded-3xl shadow-lg p-5">
  <h3 class="text-lg font-bold text-white mb-4">Playback History</h3>
  {#if events.length === 0}
    <div class="flex items-center justify-center text-gray-500 text-sm py-4">No history</div>
  {:else}
    <div class="space-y-1 max-h-[400px] overflow-y-auto">
      {#each events as e}
        <div class="flex items-center gap-3 px-2 py-2 hover:bg-neutral-800/30 rounded-lg transition-colors">
          <div class="size-8 rounded bg-neutral-800 shrink-0 overflow-hidden">
            {#await getImageUrl(e.track.cover_art) then url}
              {#if url}
                <img src={url} alt="" class="size-full object-cover" />
              {/if}
            {/await}
          </div>
          <div class="min-w-0 flex-1">
            <p class="text-sm font-medium text-white truncate">{e.track.title}</p>
            <p class="text-xs text-gray-400 truncate">
              {e.track.artists.map((a) => a.name).join(", ")}
            </p>
          </div>
          <div class="text-right shrink-0">
            <p class="text-xs text-gray-400">{formatTime(e.played_at)}</p>
            <div class="flex items-center gap-1 justify-end">
              <span class="text-[10px] text-gray-500">{(e.completion_percent).toFixed(0)}%</span>
              <div class="w-12 h-1 bg-neutral-800 rounded-full overflow-hidden">
                <div class="h-full bg-accent rounded-full" style="width: {e.completion_percent}%"></div>
              </div>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
