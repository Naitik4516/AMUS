<script lang="ts">
  import type { GrowthPoint } from "$lib/commands.svelte";

  let { data, title = "Library Growth" }: { data: GrowthPoint[]; title?: string } = $props();

  const w = 300;
  const h = 120;
  const pad = 5;

  const maxVal = $derived(
    Math.max(
      ...data.map((d) => Math.max(d.tracks_added, d.artists_added, d.albums_added)),
      1,
    ),
  );

  function seriesPath(key: "tracks_added" | "artists_added" | "albums_added"): string {
    if (data.length === 0) return "";
    return data
      .map((d, i) => {
        const x = pointX(i);
        const y = pointY(d[key]);
        return `${i === 0 ? "M" : "L"}${x.toFixed(1)},${y.toFixed(1)}`;
      })
      .join("");
  }

  function pointX(i: number): number {
    return (i / Math.max(data.length - 1, 1)) * (w - pad * 2) + pad;
  }

  function pointY(value: number): number {
    return h - pad - (value / maxVal) * (h - pad * 2);
  }

  function point(key: "tracks_added" | "artists_added" | "albums_added", i: number): string {
    return `${pointX(i).toFixed(1)},${pointY(data[i][key]).toFixed(1)}`;
  }

  const yTicks = $derived(
    [0.25, 0.5, 0.75, 1].map((f) => ({
      y: h - pad - f * (h - pad * 2),
      value: Math.round(maxVal * f),
    })),
  );
</script>

<div class="bg-card/50  border border-border rounded-3xl shadow-lg p-5">
  <div class="flex items-center justify-between mb-4">
    <h3 class="text-lg font-bold text-white">{title}</h3>
    <div class="flex items-center gap-3 text-xs text-gray-400">
      <span class="flex items-center gap-1">
        <span class="size-2 rounded-full bg-accent"></span>Tracks
      </span>
      <span class="flex items-center gap-1">
        <span class="size-2 rounded-full bg-green-400"></span>Artists
      </span>
      <span class="flex items-center gap-1">
        <span class="size-2 rounded-full bg-purple-400"></span>Albums
      </span>
    </div>
  </div>
  {#if data.length === 0}
    <div class="h-40 flex items-center justify-center text-gray-500 text-sm">No data</div>
  {:else}
    <div class="relative h-44">
      <svg viewBox="0 0 {w} {h}" class="w-full h-full overflow-visible">
        {#each yTicks as tick}
          <line
            x1={pad}
            x2={w - pad}
            y1={tick.y}
            y2={tick.y}
            stroke="currentColor"
            stroke-width="0.5"
            stroke-dasharray="3 3"
            class="text-neutral-700"
          />
        {/each}
        <path
          d={seriesPath("tracks_added")}
          fill="none"
          stroke="currentColor"
          class="text-accent"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
        <path
          d={seriesPath("artists_added")}
          fill="none"
          stroke="currentColor"
          class="text-green-400"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
        <path
          d={seriesPath("albums_added")}
          fill="none"
          stroke="currentColor"
          class="text-purple-400"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
        {#each data as _, i}
          <circle cx={pointX(i)} cy={pointY(data[i].tracks_added)} r="2" class="fill-accent" />
          <circle cx={pointX(i)} cy={pointY(data[i].artists_added)} r="2" class="fill-green-400" />
          <circle cx={pointX(i)} cy={pointY(data[i].albums_added)} r="2" class="fill-purple-400" />
        {/each}
      </svg>
      <div
        class="absolute bottom-0 left-0 right-0 flex justify-between text-xs text-gray-500 pt-1"
      >
        <span>{data[0]?.period ?? ""}</span>
        <span>{data[data.length - 1]?.period ?? ""}</span>
      </div>
    </div>
    <p class="text-sm text-gray-400 mt-3">
      Tracks:
      <span class="text-white font-medium tabular-nums">
        {data.reduce((a, b) => a + b.tracks_added, 0)}
      </span>
      · Artists:
      <span class="text-white font-medium tabular-nums">
        {data.reduce((a, b) => a + b.artists_added, 0)}
      </span>
      · Albums:
      <span class="text-white font-medium tabular-nums">
        {data.reduce((a, b) => a + b.albums_added, 0)}
      </span>
    </p>
  {/if}
</div>
