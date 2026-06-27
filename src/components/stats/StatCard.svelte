<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    label,
    value,
    subtitle,
    icon,
    trend,
  }: {
    label: string;
    value: string | number | null | undefined;
    subtitle?: string;
    icon?: Snippet;
    trend?: { value: string; positive: boolean } | null;
  } = $props();
</script>

<div class="bg-card/50 backdrop-blur-lg border border-border rounded-3xl shadow-lg p-4 flex flex-col items-center gap-2">
  {#if icon}
    <div class="text-foreground/70 ">
      {@render icon()}
    </div>
  {/if}
  <span class="text-2xl font-black text-white tabular-nums ">
    {value ?? "—"}
  </span>
  <span class="text-sm text-gray-400 font-medium">{label}</span>
  {#if subtitle}
    <span class="text-xs text-gray-500">{subtitle}</span>
  {/if}
  {#if trend}
    <span class="text-xs font-medium {trend.positive ? 'text-green-400' : 'text-red-400'}">
      {trend.value}
    </span>
  {/if}
</div>
