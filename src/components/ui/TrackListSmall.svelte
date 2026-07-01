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
                class="absolute inset-0 bg-black/40 flex items-end justify-between px-2 py-1"
            >
                <span class="playing__bar playing__bar1"></span>
                <span class="playing__bar playing__bar2"></span>
                <span class="playing__bar playing__bar3"></span>
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
    .playing__bar {
        display: inline-block;
        background: lightgray;
        width: 15%;
        height: 80%;
        animation: up-and-down 1.3s ease infinite alternate;
        border-radius: 12px;
    }

    .playing__bar1 {
        height: 60%;
    }

    .playing__bar2 {
        height: 30%;
        animation-delay: -2.2s;
    }

    .playing__bar3 {
        height: 75%;
        animation-delay: -3.7s;
    }

    @keyframes up-and-down {
        10% {
            height: 30%;
        }

        30% {
            height: 100%;
        }

        60% {
            height: 50%;
        }

        80% {
            height: 75%;
        }

        100% {
            height: 60%;
        }
    }
</style>
