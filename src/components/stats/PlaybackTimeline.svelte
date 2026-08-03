<script lang="ts">
    import type { PlaybackEvent } from "$lib/commands.svelte";
    import { formatDateShort } from "$lib/utils";
    import TrackListSmall from "$components/ui/TrackListSmall.svelte";

    let {
        events,
        canLoadMore,
        onLoadMore,
    }: {
        events: PlaybackEvent[];
        canLoadMore: boolean;
        onLoadMore: () => void;
    } = $props();


</script>

<div
    class="bg-card/50 backdrop-blur-lg border border-border rounded-3xl shadow-lg p-5"
>
    <h3 class="text-xl font-extrabold text-white mb-4 font-switzer">
        Playback History
    </h3>
    {#if events.length === 0}
        <div
            class="flex items-center justify-center text-gray-500 text-sm py-4"
        >
            No history
        </div>
    {:else}
        <div
            class="space-y-1 max-h-150 no-smooth-scroll overflow-y-auto font-satoshi font-medium"
        >
            {#each events as e}
                <div
                    class="flex items-center gap-3 pl-1 pr-2 hover:bg-neutral-800/30 rounded-xl transition-colors"
                >
                    <TrackListSmall
                        track={e.track}
                        styled={false}
                        coverArtSize={44}
                    />
                    <div class="text-right shrink-0">
                        <p class="text-sm text-gray-400">
                            {formatDateShort(e.played_at)}
                        </p>
                        <div class="flex items-center gap-1 justify-end mt-1">
                            <!-- <span
                                class="text-[10px] font-mono px-2 py-1 rounded-lg bg-white/5 text-gray-400 capitalize"
                                >{sourceLabel[e.source_type] ??
                                    e.source_type}</span
                            > -->
                            <span class="text-sm text-gray-500">
                                {e.completion_percent.toFixed(0)}%
                            </span>
                            <div
                                class="w-16 h-1 bg-neutral-800 rounded-full overflow-hidden"
                            >
                                <div
                                    class="h-full bg-accent rounded-full"
                                    style="width: {e.completion_percent}%"
                                ></div>
                            </div>
                        </div>
                    </div>
                </div>
            {/each}
        </div>
        {#if canLoadMore}
            <button
                onclick={onLoadMore}
                class="mt-3 w-full py-2 text-sm font-medium text-gray-400 hover:text-white hover:bg-neutral-800/50 rounded-xl transition-colors"
            >
                Load more
            </button>
        {/if}
    {/if}
</div>
