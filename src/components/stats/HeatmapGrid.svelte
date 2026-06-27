<script lang="ts">
  import type { HeatmapCell } from "$lib/commands.svelte";
  import { cn } from "$lib/utils";

  let {
    data,
    title,
    type,
  }: {
    data: HeatmapCell[];
    title: string;
    type: "hourly" | "weekday";
  } = $props();

  const labels = $derived(
    type === "hourly"
      ? Array.from({ length: 24 }, (_, i) => `${i}`)
      : ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]
  );

  const maxVal = $derived(Math.max(...data.map((d) => d.value), 1));

  function opacity(val: number): string {
    if (maxVal === 0) return "bg-neutral-800";
    const pct = val / maxVal;
    if (pct === 0) return "bg-neutral-800";
    if (pct < 0.25) return "bg-accent/30";
    if (pct < 0.5) return "bg-accent/50";
    if (pct < 0.75) return "bg-accent/70";
    return "bg-accent";
  }
</script>

<div class="bg-card/50 backdrop-blur-lg border border-border rounded-3xl shadow-lg p-5">
  <h3 class="text-lg font-bold text-white mb-4">{title}</h3>
  {#if data.length === 0}
    <div class="flex items-center justify-center text-gray-500 text-sm py-4">No data</div>
  {:else}
    <div class="flex flex-wrap gap-1">
      {#each data as cell}
        <div
          class={cn("size-6 rounded tooltip", opacity(cell.value))}
          title="{cell.label}: {cell.value} plays"
        ></div>
      {/each}
    </div>
    {#if type === "hourly"}
      <div class="flex flex-wrap gap-1 mt-1 text-[10px] text-gray-500">
        {#each labels as l}
          <span class="size-6 text-center">{l}</span>
        {/each}
      </div>
    {:else}
      <div class="flex gap-2 mt-2 text-xs text-gray-500">
        {#each labels as l}
          <span>{l}</span>
        {/each}
      </div>
    {/if}
  {/if}
</div>
