<script lang="ts">
  import type { GrowthPoint } from "$lib/commands.svelte";

  let { data, title = "Tracks Added" }: { data: GrowthPoint[]; title?: string } = $props();

  const w = 300;
  const h = 120;
  const pad = 5;

  const maxVal = $derived(
    Math.max(...data.map((d) => d.tracks_added), 1)
  );

  const path = $derived.by(() => {
    if (data.length === 0) return "";
    return data
      .map((d, i) => {
        const x = (i / Math.max(data.length - 1, 1)) * (w - pad * 2) + pad;
        const y = h - pad - (d.tracks_added / maxVal) * (h - pad * 2);
        return `${i === 0 ? "M" : "L"}${x.toFixed(1)},${y.toFixed(1)}`;
      })
      .join("");
  });
</script>

<div class="bg-card/50 backdrop-blur-lg border border-border rounded-3xl shadow-lg p-5">
  <h3 class="text-lg font-bold text-white mb-4">{title}</h3>
  {#if data.length === 0}
    <div class="h-40 flex items-center justify-center text-gray-500 text-sm">No data</div>
  {:else}
    <div class="h-40">
      <svg viewBox="0 0 {w} {h}" class="w-full h-full overflow-visible">
        <path d={path} fill="none" stroke="currentColor" class="text-secondary" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </div>
  {/if}
</div>
