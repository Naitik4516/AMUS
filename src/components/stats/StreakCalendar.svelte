<script lang="ts">
  import type { StreakData, Timeframe } from "$lib/commands.svelte";
  import { cn, toLocalDateKey } from "$lib/utils";

  let {
    data,
    timeframe = "all_time",
  }: { data: StreakData | null; timeframe?: Timeframe } = $props();

  const windowDays = $derived.by(() => {
    switch (timeframe) {
      case "today":
        return 7;
      case "this_week":
        return 14;
      case "this_month":
        return 45;
      case "last_3_months":
        return 100;
      case "last_6_months":
        return 200;
      case "last_year":
        return 365;
      case "last_5_years":
      case "all_time":
      default:
        return 365;
    }
  });

  const weeks = $derived.by<{ date: string; count: number }[][]>(() => {
    if (!data) return [];
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    const start = new Date(today);
    start.setDate(start.getDate() - (windowDays - 1));
    start.setDate(start.getDate() - start.getDay()); 

    const weeks: { date: string; count: number }[][] = [];
    let week: { date: string; count: number }[] = [];
    for (let d = new Date(start); d <= today; d.setDate(d.getDate() + 1)) {
      const key = toLocalDateKey(d);
      week.push({ date: key, count: data.daily_counts[key] ?? 0 });
      if (d.getDay() === 6) {
        weeks.push(week);
        week = [];
      }
    }
    if (week.length > 0) weeks.push(week);
    return weeks;
  });

  function intensity(count: number): string {
    if (count === 0) return "bg-neutral-800";
    if (count < 2) return "bg-accent/30";
    if (count < 4) return "bg-accent/50";
    if (count < 8) return "bg-accent/70";
    return "bg-accent";
  }
</script>

<div class="bg-card/50  border border-border rounded-3xl shadow-lg p-5">
  <div class="flex items-center justify-between mb-1">
    <h3 class="text-lg font-bold text-white">Activity Streak</h3>
    {#if data}
      <div class="flex gap-4 text-sm">
        <div>
          <span class="text-gray-400">Current:</span>
          <span class="text-white font-bold ml-1">{data.current_streak} days</span>
        </div>
        <div>
          <span class="text-gray-400">Longest:</span>
          <span class="text-white font-bold ml-1">{data.longest_streak} days</span>
        </div>
      </div>
    {/if}
  </div>
  {#if !data || weeks.length === 0}
    <div class="flex items-center justify-center text-gray-500 text-sm py-4">No streak data</div>
  {:else}
    <div class="overflow-x-auto">
      <div class="flex gap-1 w-max">
        <div class="flex flex-col gap-0.75 pt-0.75 pr-1 text-[9px] text-gray-500 leading-none">
          <span class="size-2.5"></span>
          <span class="h-2.5 flex items-center">Mon</span>
          <span class="size-2.5"></span>
          <span class="h-2.5 flex items-center">Wed</span>
          <span class="size-2.5"></span>
          <span class="h-2.5 flex items-center">Fri</span>
          <span class="size-2.5"></span>
        </div>
        {#each weeks as week}
          <div class="flex flex-col gap-0.75">
            {#each week as cell}
              <div
                class={cn("size-2.5 rounded-xs", intensity(cell.count))}
                title="{cell.date}: {cell.count} plays"
              ></div>
            {/each}
          </div>
        {/each}
      </div>
    </div>
    <div class="flex items-center gap-2 mt-3 text-xs text-gray-500">
      <span>Less</span>
      <div class="size-2.5 rounded bg-neutral-800"></div>
      <div class="size-2.5 rounded bg-accent/30"></div>
      <div class="size-2.5 rounded bg-accent/50"></div>
      <div class="size-2.5 rounded bg-accent/70"></div>
      <div class="size-2.5 rounded bg-accent"></div>
      <span>More</span>
    </div>
  {/if}
</div>
