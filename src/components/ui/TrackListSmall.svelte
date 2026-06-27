<script lang="ts">
    import { Music } from "@lucide/svelte";
    import { getImageUrl } from "$lib/utils";
    import type { Snippet } from "svelte";
    import type { Track } from "$lib/types";
    import { player } from "$lib/player.svelte";

    let {
        track,
        onclick,
        titleColor = "text-white",
        className,
        coverArtSize = "w-10 h-10",
    }: {
        track: Track;
        onclick?: () => void;
        titleColor?: string;
        className?: string;
        coverArtSize?: string;
    } = $props();
</script>

<button
    class="w-full flex items-center gap-4 px-2 py-2 hover:bg-white/5 transition-colors group text-left {className} "
    onclick={() => {
        player.play(track);
        onclick?.();
    }}
>
    <div
        class="{coverArtSize} relative rounded-lg bg-neutral-800 flex items-center justify-center overflow-hidden"
    >
        {#if track.cover_art}
            <img
                src={await getImageUrl(track.cover_art)}
                alt={track.title}
                class="w-full h-full object-cover"
            />
        {:else}
            <Music size={20} class="text-gray-400" />
        {/if}
        {#if player.isPlaying && player.currentTrack?.id === track.id}
            <div
                class="absolute inset-0 bg-black/40 flex items-end justify-center"
            >
                <div class="visualizer-container">
                    <div class="bar"></div>
                    <div class="bar"></div>
                    <div class="bar"></div>
                    <div class="bar"></div>
                </div>
            </div>
        {/if}
    </div>
    <div class="flex-1 min-w-0">
        <p class="text-sm font-semibold truncate {titleColor}">
            {track.title}
        </p>
        <p class="text-xs text-gray-300 truncate">
            {track.artists.map((a) => a.name).join(", ") || "Unknown Artist"}
        </p>
    </div>
</button>

<style>
    .visualizer-container {
        display: flex;
        align-items: flex-end; /* Keeps bars aligned at the bottom */
        gap: 4px;
        height: 24px;
        width: max-content;
        margin-bottom: 3px;
    }

    .bar {
        width: 3px;
        height: 100%;
        background-color: #1db954; 
        border-radius: 30px 30px 0 0; 
        animation: bounce 0.5s ease-in-out infinite alternate;
        transform-origin: bottom;
    }

    /* Offset the animation delay for each bar to create the wave effect */
    .bar:nth-child(1) {
        animation-delay: 0.1s;
    }
    .bar:nth-child(2) {
        animation-delay: 0.25s;
    }
    .bar:nth-child(3) {
        animation-delay: 0.35s;
    }
    .bar:nth-child(4) {
        animation-delay: 0.5s;
    }

    @keyframes bounce {
        0% {
            transform: scaleY(0.1);
        }
        100% {
            transform: scaleY(1);
        }
    }
</style>
