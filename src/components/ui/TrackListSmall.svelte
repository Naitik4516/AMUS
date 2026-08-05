<script lang="ts">
    import TrackMenu from "$components/ui/Menu/TrackMenu.svelte";
    import { openContextMenu } from "$lib/context-menu.svelte";
    import { player } from "$lib/player.svelte";
    import { store } from "$lib/stores.svelte";
    import type { Context, Track } from "$lib/types";
    import { Music, Play } from "@lucide/svelte";
    import { fade } from "svelte/transition";
    import PlayingVisualizer from "./PlayingVisualizer.svelte";

    let {
        track,
        titleColor = "text-white",
        coverArtSize = 48,
        styled = true,
        onclick,
        context = null,
        ...props
    }: {
        track: Track;
        titleColor?: string;
        coverArtSize?: number;
        styled?: boolean;
        onclick?: () => void;
        context?: Context | null;
    } = $props();

    let hovering = $state(false);

    let trackContext = $derived<Context>(
        context ?? {
            type: "Album",
            id: track.album.id,
            name: track.album.name,
            coverArt: track.album.cover_art ?? null,
        },
    );

    function handleContextMenu(e: MouseEvent) {
        e.preventDefault();
        openContextMenu(TrackMenu, {
            position: { type: "coordinates", x: e.clientX, y: e.clientY },
            track: track,
            context: trackContext,
        });
    }
</script>

<div
    class="w-full flex items-center gap-4 px-2 py-2 overflow-hidden select-none {styled
        ? 'hover:bg-white/5 transition-colors rounded-xl'
        : ''}  text-left"
    oncontextmenu={handleContextMenu}
    {...props}
>
    <button
        class="relative rounded-lg flex items-center justify-center overflow-hidden shrink-0"
        style="width: {coverArtSize}px; height: {coverArtSize}px;"
        onclick={() => (onclick ? onclick() : player.play([track]))}
        onmouseenter={() => (hovering = true)}
        onmouseleave={() => (hovering = false)}
    >
        {#if track.cover_art}
            <img
                src={store.getImageSrc(track.cover_art)}
                alt={track.title}
                class="w-full h-full object-cover"
            />
        {:else if !hovering}
            <div
                class="absolute inset-0 flex items-center justify-center border bg-gray-700/60 inset-shadow-xs"
            >
                <Music size={coverArtSize / 2.5} strokeWidth={2.5} class="text-gray-300" />
            </div>
        {/if}
        {#if player.isPlaying && player.currentTrack?.id === track.id}
            <div
                class="absolute inset-0 bg-black/50 flex items-end justify-around p-1.5"
            >
                <PlayingVisualizer />
            </div>
        {/if}
        {#if hovering}
            <div
                class="absolute inset-0 bg-black/50 flex items-center justify-center"
                transition:fade={{ duration: 150 }}
            >
                <Play size={20} class="text-white fill-white" />
            </div>
        {/if}
    </button>
    <div class="flex flex-col min-w-0 flex-1 font-satoshi">
        <span class="truncate block">
            <a
                href="/library/track/{track.id}"
                class="font-semibold text-[16px] tracking-wide {titleColor}"
            >
                {track.title}
            </a>
        </span>
        <div class="text-sm text-gray-300 truncate block">
            {#each track.artists as artist, index (artist.id)}
                <a href="/library/artists/{artist.id}">
                    {artist.name}{#if index < track.artists.length - 1}, {""}
                    {/if}
                </a>
            {/each}
        </div>
    </div>
</div>
