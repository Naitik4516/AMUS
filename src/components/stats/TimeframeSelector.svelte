<script lang="ts">
  import type { Timeframe } from "$lib/commands.svelte";

  const allOptions: { label: string; value: Timeframe }[] = [
    { label: "Today", value: "today" },
    { label: "Week", value: "this_week" },
    { label: "Month", value: "this_month" },
    { label: "3M", value: "last_3_months" },
    { label: "6M", value: "last_6_months" },
    { label: "1Y", value: "last_year" },
    { label: "5Y", value: "last_5_years" },
    { label: "All", value: "all_time" },
  ];

  let {
    value,
    onchange,
    available = allOptions.map((o) => o.value),
  }: {
    value: Timeframe;
    onchange: (tf: Timeframe) => void;
    available?: Timeframe[];
  } = $props();

  const options = $derived(allOptions.filter((o) => available.includes(o.value)));
</script>

<div class="flex items-center gap-1 bg-card/50 backdrop-blur-lg border border-border rounded-3xl shadow-lg p-1 overflow-x-auto">
  {#each options as opt}
    <button
      onclick={() => onchange(opt.value)}
      class="px-3 py-1.5 text-xs font-medium rounded-lg whitespace-nowrap transition-colors
        {value === opt.value ? 'bg-accent text-black' : 'text-gray-400 hover:text-white hover:bg-neutral-700/50'}"
    >
      {opt.label}
    </button>
  {/each}
</div>
