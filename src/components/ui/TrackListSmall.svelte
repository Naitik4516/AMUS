<script lang="ts">
    import { Music } from "@lucide/svelte";
    import type { Track } from "$lib/types";
    import { player } from "$lib/player.svelte";
    import { store } from "$lib/stores.svelte";
    import PlayingVisualizer from "./PlayingVisualizer.svelte";

    let {
        track,
        titleColor = "text-white",
        coverArtSize = "w-11 h-11",
        styled = false,
        ...props
    }: {
        track: Track;
        titleColor?: string;
        coverArtSize?: string;
        styled?: boolean;
    } = $props();
</script>

<div
    class="w-full flex items-center gap-4 px-2 py-2 overflow-hidden  {styled
        ? 'hover:bg-white/5 transition-colors'
        : ''}  text-left"
    {...props}
>
    <button
        class="{coverArtSize} relative rounded-lg bg-neutral-800 flex items-center justify-center overflow-hidden shrink-0"
        onclick={() => player.play([track])}
    >
        {#if track.cover_art}
            <img
                src={store.getImageSrc(track.cover_art)}
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
                <PlayingVisualizer />
            </div>
        {/if}
    </button>
    <div class="flex flex-col min-w-0 flex-1">
        <a href="/track/{track.id}" class="font-semibold truncate {titleColor}">
            {track.title}
        </a>
        <div
            class="text-sm text-gray-300 truncate block"
        >
            {#each track.artists as artist, index (artist.id)}
                <a href="/artist/{artist.id}">
                    {artist.name}{#if index < track.artists.length - 1}, {""}
                    {/if}
                </a>
            {/each}
        </div>
    </div>
</div>
