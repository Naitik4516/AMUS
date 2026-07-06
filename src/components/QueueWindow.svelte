<script lang="ts">
    import { X } from "@lucide/svelte";
    import TrackListSmall from "./ui/TrackListSmall.svelte";
    import { player } from "$lib/player.svelte";

    let { showQueue = $bindable(false) }: { showQueue?: boolean } = $props();
</script>

{#if showQueue}
    <div
        class="absolute bottom-full right-1 mb-4 w-90 bg-card/60 backdrop-blur-lg border-2 border-border/70 rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[75vh]"
    >
        <div
            class="p-4 border-b border-neutral-800 flex justify-between items-center bg-neutral-900/50"
        >
            <h3 class="font-bold text-white text-lg">Queue</h3>
            <button
                onclick={() => (showQueue = false)}
                class="text-gray-300 hover:text-white"
            >
                <X size={18} />
            </button>
        </div>
        <div class="overflow-y-auto px-4 pb-4">
            <!-- Now Playing -->
            {#if player.currentTrack}
                <h4
                    class="py-2 text-[13px] font-switzer font-bold uppercase tracking-wider text-stone-400"
                >
                    Now Playing
                </h4>

                <TrackListSmall
                    track={player.currentTrack}
                    titleColor="text-accent"
                    className="rounded-xl"
                    onclick={() => {}}
                />
            {/if}

            <!-- Next in Queue -->
            {#if player.userQueue.length > 0}
                <h4
                    class="py-2 text-[13px] font-bold uppercase tracking-wider text-stone-400"
                >
                    Next in Queue
                </h4>
                {#each player.userQueue as track, i}
                    <TrackListSmall
                        {track}
                        className="rounded-xl"
                        onclick={() => {
                            player.contextPosition = i;
                        }}
                    />
                {/each}
            {/if}

            <!-- Next from Playlist/Album/Artist -->
            <!-- {#if player.playNext.length > 0}
                <p
                    class="py-2 text-[13px] font-bold uppercase tracking-wider text-stone-400"
                >
                    Next from Suggestions
                </p>
                {#each player.playNext.slice(0, 5) as track, i}
                    <TrackListSmall {track} className="rounded-xl" />
                {/each}
            {/if}

            {#if !player.currentTrack && player.userQueue.length === 0 && player.playNext.length === 0}
                <p class="px-4 py-8 text-center text-sm text-zinc-500">
                    No tracks in queue
                </p>
            {/if} -->
        </div>
    </div>
{/if}
