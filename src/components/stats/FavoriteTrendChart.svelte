<script lang="ts">
  import type { FavoriteTrend } from "$lib/commands.svelte";

  let { data, title = "Favorite Trends" }: { data: FavoriteTrend[]; title?: string } = $props();
</script>

<div class="bg-card/50 backdrop-blur-lg border border-border rounded-3xl shadow-lg p-5">
  <h3 class="text-lg font-bold text-white mb-4">{title}</h3>
  {#if data.length === 0}
    <div class="flex items-center justify-center text-gray-500 text-sm py-4">No trend data</div>
  {:else}
    <div class="space-y-3">
      {#each data as trend}
        <div class="text-sm">
          <span class="text-gray-400 font-mono text-xs">{trend.period}</span>
          <div class="mt-1 space-y-0.5">
            {#if trend.top_track_name}
              <div class="flex gap-2">
                <span class="text-gray-500 w-10 shrink-0">Track:</span>
                <span class="text-white truncate">{trend.top_track_name}</span>
              </div>
            {/if}
            {#if trend.top_artist_name}
              <div class="flex gap-2">
                <span class="text-gray-500 w-10 shrink-0">Artist:</span>
                <span class="text-white truncate">{trend.top_artist_name}</span>
              </div>
            {/if}
            {#if trend.top_album_name}
              <div class="flex gap-2">
                <span class="text-gray-500 w-10 shrink-0">Album:</span>
                <span class="text-white truncate">{trend.top_album_name}</span>
              </div>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
