<script lang="ts">
    import type { Component } from "svelte";

    let {
        label,
        value,
        subtitle,
        Icon,
        trend,
    }: {
        label: string;
        value: string | number | null | undefined;
        subtitle?: string;
        Icon?: Component;
        trend?: { value: string; positive: boolean } | null;
    } = $props();
</script>

<div
    class="bg-card/50 border border-border rounded-3xl shadow-lg p-4 flex flex-col items-center justify-center gap-2"
>
    {#if Icon}
        <Icon class="text-gray-300 size-8" strokeWidth={2.5} />
    {/if}
    <span class="text-2xl font-black text-white tabular-nums">
        {value ?? "—"}
    </span>
    <span class="text-sm text-gray-400 font-medium">{label}</span>
    {#if subtitle}
        <span class="text-xs text-gray-500">{subtitle}</span>
    {/if}
    {#if trend}
        <span
            class="text-xs font-medium {trend.positive
                ? 'text-green-400'
                : 'text-red-400'}"
        >
            {trend.value}
        </span>
    {/if}
</div>
