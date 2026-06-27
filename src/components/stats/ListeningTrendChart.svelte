<script lang="ts">
  import type { TimeSeriesPoint } from "$lib/commands.svelte";

  let { data, title = "Listening Time" }: { data: TimeSeriesPoint[]; title?: string } = $props();

  const max = $derived(Math.max(...data.map((d) => d.value), 1));
  const points = $derived(
    data.map((d, i) => ({
      x: i,
      y: ((max - d.value) / max) * 120,
      value: d.value,
      label: d.date,
    }))
  );
  const path = $derived(
    points.length > 1
      ? "M" + points.map((p, i) => `${(i / (points.length - 1)) * 300},${p.y}`).join(" L")
      : ""
  );
  const areaPath = $derived(
    points.length > 1
      ? path + ` L300,120 L0,120 Z`
      : ""
  );
</script>

<div class="bg-card/50 backdrop-blur-lg border border-border rounded-3xl shadow-lg p-5">
  <h3 class="text-lg font-bold text-white mb-4">{title}</h3>
  {#if data.length === 0}
    <div class="h-40 flex items-center justify-center text-gray-500 text-sm">No data</div>
  {:else}
    <div class="relative h-40">
      <svg viewBox="0 0 300 120" class="w-full h-full overflow-visible">
        {#if areaPath}
          <path d={areaPath} fill="currentColor" class="text-secondary/20" />
        {/if}
        {#if path}
          <path d={path} fill="none" stroke="currentColor" class="text-secondary" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
        {/if}
      </svg>
      <div class="absolute bottom-0 left-0 right-0 flex justify-between text-xs text-gray-500 pt-1">
        <span>{data[0]?.date ?? ""}</span>
        <span>{data[data.length - 1]?.date ?? ""}</span>
      </div>
    </div>
    <p class="text-sm text-gray-400 mt-3">
      Total: <span class="text-white font-medium">{data.reduce((a, b) => a + b.value, 0).toFixed(0)} min</span>
      · Avg: <span class="text-white font-medium">{(data.reduce((a, b) => a + b.value, 0) / Math.max(data.length, 1)).toFixed(1)} min/day</span>
    </p>
  {/if}
</div>
