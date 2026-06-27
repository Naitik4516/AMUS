<script lang="ts">
  import type { StreakData } from "$lib/commands.svelte";
  import { cn } from "$lib/utils";

  let { data }: { data: StreakData | null } = $props();

  const weeks = $derived.by(() => {
    if (!data) return [];
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    const cells: { date: string; count: number }[] = [];
    const start = new Date(today);
    start.setDate(start.getDate() - 364); // ~52 weeks

    for (let d = new Date(start); d <= today; d.setDate(d.getDate() + 1)) {
      const key = d.toISOString().slice(0, 10);
      cells.push({ date: key, count: data.daily_counts[key] ?? 0 });
    }
    return cells;
  });

  function intensity(count: number): string {
    if (count === 0) return "bg-neutral-800";
    if (count < 2) return "bg-accent/30";
    if (count < 5) return "bg-accent/50";
    if (count < 10) return "bg-accent/70";
    return "bg-accent";
  }
</script>

<div class="bg-card/50 backdrop-blur-lg border border-border rounded-3xl shadow-lg p-5">
  <h3 class="text-lg font-bold text-white mb-1">Activity Streak</h3>
  {#if data}
    <div class="flex gap-4 mb-4 text-sm">
      <div>
        <span class="text-gray-400">Current streak:</span>
        <span class="text-white font-bold ml-1">{data.current_streak} days</span>
      </div>
      <div>
        <span class="text-gray-400">Longest streak:</span>
        <span class="text-white font-bold ml-1">{data.longest_streak} days</span>
      </div>
    </div>
  {/if}
  {#if !data || weeks.length === 0}
    <div class="flex items-center justify-center text-gray-500 text-sm py-4">No streak data</div>
  {:else}
    <div class="overflow-x-auto">
      <div class="flex gap-[2px]" style="flex-wrap: wrap;">
        {#each weeks as cell}
          <div
            class={cn("size-3 rounded-[2px]", intensity(cell.count))}
            title="{cell.date}: {cell.count} plays"
          ></div>
        {/each}
      </div>
    </div>
    <div class="flex items-center gap-2 mt-2 text-xs text-gray-500">
      <span>Less</span>
      <div class="size-3 rounded bg-neutral-800"></div>
      <div class="size-3 rounded bg-accent/30"></div>
      <div class="size-3 rounded bg-accent/50"></div>
      <div class="size-3 rounded bg-accent/70"></div>
      <div class="size-3 rounded bg-accent"></div>
      <span>More</span>
    </div>
  {/if}
</div>
