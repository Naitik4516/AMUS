<script lang="ts">
    import type { FormatStat } from "$lib/commands.svelte";
    import { formatBytes } from "$lib/utils";

    let {
        data,
        title = "Format Distribution",
    }: { data: FormatStat[]; title?: string } = $props();

    const colors = [
        "#f5f5f5",
        "#4ade80",
        "#facc15",
        "#f87171",
        "#60a5fa",
        "#c084fc",
        "#f472b6",
        "#fb923c",
    ];

    const formatLabel: Record<string, string> = {
        mpeg: "MP3",
        mp4: "M4A",
        aac: "AAC",
        flac: "FLAC",
        opus: "Opus",
        vorbis: "Vorbis",
        wav: "WAV",
        aiff: "AIFF",
        wavpack: "WavPack",
        ape: "APE",
        mpc: "MPC",
    };

    const total = $derived(data.reduce((a, b) => a + b.count, 0));

    const R = 60;
    const CIRC = 2 * Math.PI * R;
    const GAP = 2;

    const segments = $derived.by(() => {
        let offset = 0;
        return data.map((f, i) => {
            const dash = Math.max((f.percentage / 100) * CIRC - GAP, 0.5);
            const seg = {
                color: colors[i % colors.length],
                dash,
                offset,
                percentage: f.percentage,
                format: f.format,
            };
            offset += (f.percentage / 100) * CIRC;
            return seg;
        });
    });

    function labelOf(f: FormatStat): string {
        return formatLabel[f.format.toLowerCase()] ?? f.format.toUpperCase();
    }

    function qualityLine(f: FormatStat): string {
        const parts: string[] = [];
        if (f.avg_bitrate_kbps != null)
            parts.push(`${f.avg_bitrate_kbps.toFixed(0)} kbps`);
        if (f.avg_sample_rate != null)
            parts.push(`${(f.avg_sample_rate / 1000).toFixed(1)} kHz`);
        if (f.avg_bit_depth != null)
            parts.push(`${f.avg_bit_depth.toFixed(0)} bit`);
        return parts.join(" · ");
    }
</script>

<div class="bg-card/50 border border-border rounded-3xl shadow-lg p-5">
    <h3 class="text-xl font-bold font-switzer text-white mb-5">{title}</h3>
    {#if data.length === 0}
        <div
            class="flex items-center justify-center text-gray-500 text-sm py-4"
        >
            No data
        </div>
    {:else}
        <div class="flex items-center gap-6">
            <div class="relative size-36 shrink-0">
                <svg viewBox="0 0 140 140" class="size-full -rotate-90">
                    <circle
                        cx="70"
                        cy="70"
                        r={R}
                        fill="none"
                        stroke="rgba(255, 255, 255, 0.06)"
                        stroke-width="16"
                    />
                    {#each segments as seg}
                        <circle
                            cx="70"
                            cy="70"
                            r={R}
                            fill="none"
                            stroke={seg.color}
                            stroke-width="16"
                            stroke-linecap="round"
                            stroke-dasharray="{seg.dash} {CIRC}"
                            stroke-dashoffset={-seg.offset}
                        />
                    {/each}
                </svg>
                <div
                    class="absolute inset-0 flex flex-col items-center justify-center pointer-events-none"
                >
                    <span class="text-2xl font-black text-white tabular-nums"
                        >{total}</span
                    >
                    <span class="text-xs text-gray-400 font-medium">tracks</span>
                </div>
            </div>
            <div class="flex-1 min-w-0 space-y-2.5 font-satoshi font-semibold">
                {#each data as f, i}
                    <div class="group flex items-center gap-2.5 text-sm">
                        <span
                            class="size-2.5 rounded-sm shrink-0"
                            style="background: {colors[i % colors.length]}"
                        ></span>
                        <span
                            class="text-white font-bold uppercase w-10 shrink-0"
                            >{labelOf(f)}</span
                        >
                        <span class="text-gray-300 tabular-nums">{f.count}</span
                        >
                        <span class="text-gray-400 text-xs"
                            >({f.percentage.toFixed(1)}%)</span
                        >
                        <span class="text-gray-400 text-xs ml-auto tabular-nums"
                            >{formatBytes(f.total_bytes)}</span
                        >
                    </div>
                    <div class="pl-5 -mt-1.5 text-xs text-gray-400">
                        {#if qualityLine(f)}
                            {qualityLine(f)}
                        {:else}
                            <span class="text-gray-500">no quality data</span>
                        {/if}
                    </div>
                {/each}
            </div>
        </div>
    {/if}
</div>
