<script lang="ts">
  import type { TopArtist } from "$lib/commands.svelte";
  import { getImageUrl, formatDurationShort } from "$lib/utils";

  let { artists }: { artists: TopArtist[] } = $props();
</script>

<div class="bg-card/50 backdrop-blur-lg border border-border rounded-3xl shadow-lg overflow-hidden">
  <div class="px-5 py-4 border-b border-neutral-800">
    <h3 class="text-lg font-bold text-white">Top Artists</h3>
  </div>
  {#if artists.length === 0}
    <div class="p-8 text-center text-gray-500">No listening history yet</div>
  {:else}
    <div class="divide-y divide-neutral-800/50 max-h-[500px] overflow-y-auto">
      {#each artists as a, i}
        <div class="flex items-center gap-3 px-5 py-3 hover:bg-neutral-800/30 transition-colors">
          <span class="text-sm font-mono text-gray-500 w-6 shrink-0 text-right">{i + 1}</span>
          <div class="size-9 rounded-full bg-neutral-800 shrink-0 overflow-hidden">
            {#await getImageUrl(a.artist.profile_image, "artist") then url}
              {#if url}
                <img src={url} alt="" class="size-full object-cover" />
              {:else}
                <div class="size-full flex items-center justify-center text-xs text-gray-500">
                  {a.artist.name.charAt(0)}
                </div>
              {/if}
            {/await}
          </div>
          <div class="min-w-0 flex-1">
            <p class="text-sm font-medium text-white truncate">{a.artist.name}</p>
            <p class="text-xs text-gray-400">{a.tracks_played} tracks</p>
          </div>
          <div class="text-right shrink-0">
            <p class="text-sm font-medium text-white tabular-nums">{a.play_count}x</p>
            <p class="text-xs text-gray-400">{formatDurationShort(a.total_listening_time_sec)}</p>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
