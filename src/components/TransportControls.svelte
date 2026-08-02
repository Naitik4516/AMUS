<script lang="ts">
    import {
        Play,
        Pause,
        SkipBack,
        SkipForward,
        Shuffle,
        Repeat,
        Repeat1,
    } from "@lucide/svelte";
    import { player } from "$lib/player.svelte";

    let { size = "sm", showShuffle = true, showRepeat = true } = $props();
    let sm = $derived(size === "sm");
</script>

<div class="flex items-center {sm ? 'gap-6' : 'gap-8'}">
    {#if showShuffle}
        <button
            class="hover:text-white transition-colors"
            class:text-accent={player.shuffleEnabled}
            class:text-gray-200={!player.shuffleEnabled}
            onclick={() => player.toggleShuffle()}
            aria-label="Toggle shuffle"
        >
            <Shuffle size={20} />
        </button>
    {/if}

    <button
        class="text-gray-300 hover:text-white transition-colors"
        onclick={() => player.previous()}
        aria-label="Previous track"
    >
        <SkipBack size={sm ? 22 : 28} fill="currentColor" />
    </button>

    <button
        class="bg-white text-black rounded-full {sm ? 'p-3' : 'p-4'} hover:scale-105 transition-transform shadow-lg"
        onclick={() => player.playPause()}
        aria-label={player.isPlaying ? "Pause" : "Play"}
    >
        {#if player.isPlaying}
            <Pause size={sm ? 24 : 32} fill="currentColor" />
        {:else}
            <Play
                size={sm ? 24 : 32}
                fill="currentColor"
                class={sm ? "" : "ml-0.5"}
            />
        {/if}
    </button>

    <button
        class="text-gray-200 hover:text-white transition-colors"
        onclick={() => player.next()}
        aria-label="Next track"
    >
        <SkipForward size={sm ? 22 : 28} fill="currentColor" />
    </button>

    {#if showRepeat}
        <button
            class="hover:text-white transition-colors"
            class:text-accent={player.repeatMode !== "OFF"}
            class:text-gray-200={player.repeatMode === "OFF"}
            onclick={() => player.cycleRepeat()}
            aria-label="Cycle repeat mode"
        >
            {#if player.repeatMode === "ONE"}
                <Repeat1 size={20} />
            {:else}
                <Repeat size={20} />
            {/if}
        </button>
    {/if}
</div>
