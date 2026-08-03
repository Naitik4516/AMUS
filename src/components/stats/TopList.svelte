<script lang="ts" generics="T extends TopRankItem">
    import type { Snippet } from "svelte";
    import type { TopRankItem, TopSort } from "$lib/commands.svelte";
    import { formatDurationShort } from "$lib/utils";
    import { flip } from "svelte/animate";

    let {
        title,
        items,
        sortBy,
        onSortChange,
        leading,
        subtitle,
    }: {
        title: string;
        items: T[];
        sortBy?: TopSort;
        onSortChange?: (sort: TopSort) => void;
        leading: Snippet<[T]>;
        subtitle?: Snippet<[T]>;
    } = $props();

    const sortOptions: { label: string; value: TopSort }[] = [
        { label: "Plays", value: "plays" },
        { label: "Time", value: "time" },
    ];

    function rankName(item: T): string {
        if ("track" in item) return item.track.title;
        if ("artist" in item) return item.artist.name;
        if ("album" in item) return item.album.name;
        return item.genre.name;
    }

    function getId(item: T): number {
        if ("track" in item) return item.track.id;
        if ("artist" in item) return item.artist.id;
        if ("album" in item) return item.album.id;
        return item.genre.id;
    }
</script>

<div
    class="bg-card/50 border border-border rounded-3xl shadow-lg overflow-hidden"
>
    <div
        class="p-4 border-b border-neutral-800 flex items-center justify-between gap-2 font-satoshi"
    >
        <h3 class="text-xl font-extrabold text-white font-switzer">{title}</h3>
        {#if onSortChange}
            <div
                class="flex items-center gap-1 bg-neutral-800/60 rounded-full p-0.5 shrink-0"
            >
                {#each sortOptions as opt}
                    <button
                        onclick={() => onSortChange(opt.value)}
                        class="px-2.5 py-1 text-xs font-semibold rounded-full transition-colors
              {sortBy === opt.value
                            ? 'bg-accent text-black'
                            : 'text-gray-400 hover:text-white'}"
                        aria-pressed={sortBy === opt.value}
                    >
                        {opt.label}
                    </button>
                {/each}
            </div>
        {/if}
    </div>
    {#if items.length === 0}
        <div class="p-8 text-center text-gray-500">
            No listening history yet
        </div>
    {:else}
        <div
            class="divide-y divide-neutral-800/30 max-h-125 no-smooth-scroll overflow-y-auto font-satoshi font-medium"
        >
            {#each items as item, i (getId(item))}
                <div
                    class="flex items-center gap-3 pr-5 pl-2 py-2 hover:bg-neutral-800/30 transition-colors"
                    animate:flip={{ duration: 200 }}
                >
                    <span
                        class="text-sm font-mono text-gray-500 w-6 shrink-0 text-right"
                        >{i + 1}</span
                    >
                    {@render leading(item)}
                    <div class="min-w-0 flex-1">
                        <p class="font-semibold text-base text-white truncate">
                            {rankName(item)}
                        </p>
                        {#if subtitle}
                            {@render subtitle(item)}
                        {/if}
                    </div>
                    <div class="text-right shrink-0">
                        <p class="text-sm font-medium text-white tabular-nums">
                            {item.play_count}x
                        </p>
                        <p class="text-xs text-gray-400">
                            {formatDurationShort(item.total_listening_time_sec)}
                        </p>
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</div>
