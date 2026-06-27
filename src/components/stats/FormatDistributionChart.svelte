<script lang="ts">
  import type { FormatStat } from "$lib/commands.svelte";
  import { formatBytes, cn } from "$lib/utils";

  let { data, title = "Format Distribution" }: { data: FormatStat[]; title?: string } = $props();

  const colors = ["bg-accent", "bg-green-400", "bg-yellow-400", "bg-red-400", "bg-blue-400", "bg-purple-400", "bg-pink-400", "bg-orange-400"];
  const total = $derived(data.reduce((a, b) => a + b.count, 0));
</script>

<div class="bg-card/50 backdrop-blur-lg border border-border rounded-3xl shadow-lg p-5">
  <h3 class="text-lg font-bold text-white mb-4">{title}</h3>
  {#if data.length === 0}
    <div class="flex items-center justify-center text-gray-500 text-sm py-4">No data</div>
  {:else}
    <div class="flex h-2 rounded-full overflow-hidden mb-4">
      {#each data as f, i}
        <div
          class={cn(colors[i % colors.length], "h-full transition-all")}
          style="width: {f.percentage}%"
        ></div>
      {/each}
    </div>
    <div class="space-y-2">
      {#each data as f, i}
        <div class="flex items-center gap-3 text-sm">
          <span class={cn("size-2.5 rounded-sm shrink-0", colors[i % colors.length])}></span>
          <span class="text-white font-medium uppercase min-w-10">.{f.format}</span>
          <span class="text-gray-400 tabular-nums">{f.count}</span>
          <span class="text-gray-500 text-xs">({f.percentage.toFixed(1)}%)</span>
          <span class="text-gray-500 text-xs ml-auto">{formatBytes(f.total_bytes)}</span>
        </div>
      {/each}
      <div class="flex items-center gap-3 text-sm pt-2 border-t border-neutral-800 mt-2">
        <span class="text-gray-400 font-medium">Total</span>
        <span class="text-white tabular-nums">{total}</span>
        <span class="text-gray-500 text-xs">(100%)</span>
      </div>
    </div>
  {/if}
</div>
