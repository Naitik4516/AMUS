<script lang="ts">
    import { onMount } from "svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import {
        Play,
        Pause,
        SkipBack,
        SkipForward,
        Shuffle,
        Repeat,
        Repeat1,
        Heart,
        X,
        Volume,
        Volume1,
        Volume2,
        VolumeX,
        Music2,
        Maximize2,
        MicVocal,
    } from "@lucide/svelte";
    import { player } from "$lib/player.svelte";
    import { store } from "$lib/stores.svelte";
    import { formatDurationColon } from "$lib/utils";
    import { fullscreen } from "$lib/fullscreen.svelte";
    import Slider from "$components/ui/Slider.svelte";
    import LyricsView from "./LyricsView.svelte";

    let { onExit = () => {} } = $props();

    let coverUrl = $derived(
        player.currentTrack?.cover_art
            ? store.getImageSrc(player.currentTrack.cover_art)
            : null,
    );

    let showLyrics = $state(false);

    function exit() {
        getCurrentWindow().setFullscreen(false);
        fullscreen.active = false;
        onExit();
    }

    onMount(() => {
        getCurrentWindow().setFullscreen(true);
    });

    async function toggleFavorite() {
        if (player.currentTrack) {
            await player.toggleFavorite(player.currentTrack);
        }
    }
</script>

<svelte:window
    onkeydown={(e) => {
        if (e.key === "Escape") exit();
    }}
/>

<div class="fixed inset-0 z-50 bg-neutral-900">
    <div class="absolute inset-0 -z-10">
        {#if coverUrl}
            <img
                src={coverUrl}
                alt=""
                id="bg-image"
                class="w-full h-full object-cover blur-[130px] bg-image {player.isPlaying
                    ? 'playing'
                    : ''} "
            />
        {/if}
        <div
            class="absolute inset-0 {coverUrl
                ? 'bg-black/45'
                : 'bg-linear-to-br from-background to-neutral-950'}"
        ></div>
    </div>

    <button
        onclick={exit}
        class="absolute top-5 left-5 z-20 size-10 rounded-full bg-white/5 hover:bg-white/15 backdrop-blur-md flex items-center justify-center text-gray-200 hover:text-white transition-all"
        aria-label="Exit fullscreen"
    >
        <X size={18} />
    </button>

    <div class="z-10 h-full">
        {#if player.currentTrack}
            {@const track = player.currentTrack}
            <div class="grid grid-cols-[2fr_3fr] h-full">
                <div
                    class="flex flex-col items-center justify-center gap-6 px-8 py-16"
                >
                    <div
                        class="size-72 rounded-3xl shadow-2xl overflow-hidden shrink-0 ring-1 ring-white/10"
                    >
                        {#if coverUrl}
                            <img
                                src={coverUrl}
                                alt={track.title}
                                class="w-full h-full object-cover"
                            />
                        {:else}
                            <div
                                class="w-full h-full bg-neutral-800 flex items-center justify-center"
                            >
                                <Music2 size={64} class="text-neutral-600" />
                            </div>
                        {/if}
                    </div>

                    <div class="text-center max-w-md">
                        <h1
                            class="text-3xl font-black text-white font-switzer truncate"
                        >
                            {track.title}
                        </h1>
                        <p class="text-lg text-gray-300 font-medium mt-1">
                            {#each track.artists as artist, ai (artist.id)}
                                {#if ai > 0}<span>, </span>{/if}
                                {artist.name}
                            {/each}
                        </p>
                        {#if track.album?.name}
                            <p class="text-sm text-gray-300 mt-0.5">
                                {track.album.name}{track.album.year
                                    ? ` \u00B7 ${track.album.year}`
                                    : ""}
                            </p>
                        {/if}
                    </div>

                    <div
                        class="w-full max-w-md flex flex-col gap-5 items-center px-4"
                    >
                        <div class="flex w-full items-center gap-3">
                            <span
                                class="text-xs font-medium text-gray-400 w-10 text-right tabular-nums"
                            >
                                {formatDurationColon(player.position)}
                            </span>
                            <div class="flex-1">
                                <Slider
                                    value={player.progress}
                                    onValueChange={(val) => {
                                        if (player.currentTrack) {
                                            player.seek(
                                                val *
                                                    player.currentTrack
                                                        .duration_seconds,
                                            );
                                        }
                                    }}
                                />
                            </div>
                            <span
                                class="text-xs font-medium text-gray-400 w-10 tabular-nums"
                            >
                                {formatDurationColon(track.duration_seconds)}
                            </span>
                        </div>
                        <div class="flex items-center gap-8">
                            <button
                                class="text-gray-300 hover:text-white transition-colors"
                                onclick={() => player.previous()}
                                aria-label="Previous track"
                            >
                                <SkipBack size={28} fill="currentColor" />
                            </button>
                            <button
                                class="bg-white text-black rounded-full p-4 hover:scale-105 transition-transform shadow-lg"
                                onclick={() => player.playPause()}
                                aria-label={player.isPlaying ? "Pause" : "Play"}
                            >
                                {#if player.isPlaying}
                                    <Pause size={32} fill="currentColor" />
                                {:else}
                                    <Play
                                        size={32}
                                        fill="currentColor"
                                        class="ml-0.5"
                                    />
                                {/if}
                            </button>
                            <button
                                class="text-gray-300 hover:text-white transition-colors"
                                onclick={() => player.next()}
                                aria-label="Next track"
                            >
                                <SkipForward size={28} fill="currentColor" />
                            </button>
                        </div>

                        <div
                            class="flex items-center w-full justify-evenly mt-3"
                        >
                            <button
                                onclick={toggleFavorite}
                                class="transition-colors"
                                class:text-rose-600={track.is_favorite}
                                class:text-gray-400={!track.is_favorite}
                                aria-label="Toggle favorite"
                            >
                                <Heart
                                    size={20}
                                    class={track.is_favorite
                                        ? "fill-rose-600"
                                        : ""}
                                />
                            </button>
                            <button
                                class="text-gray-400 hover:text-white transition-colors"
                                class:text-white={player.shuffleEnabled}
                                onclick={() => player.toggleShuffle()}
                                aria-label="Toggle shuffle"
                            >
                                <Shuffle size={20} />
                            </button>
                            <button
                                class="hover:text-white transition-colors"
                                class:text-accent={player.repeatMode !== "OFF"}
                                class:text-gray-400={player.repeatMode ===
                                    "OFF"}
                                onclick={() => player.cycleRepeat()}
                                aria-label="Cycle repeat mode"
                            >
                                {#if player.repeatMode === "ONE"}
                                    <Repeat1 size={20} />
                                {:else}
                                    <Repeat size={20} />
                                {/if}
                            </button>
                            <button
                                onclick={() => (showLyrics = !showLyrics)}
                                class="text-gray-300 hover:text-white transition-colors"
                                class:text-accent={showLyrics}
                                aria-label="Show lyrics"
                            >
                                <MicVocal size={18} />
                            </button>
                            <div
                                class="flex items-center gap-2"
                                onwheel={(e) => {
                                    e.preventDefault();
                                    const delta = e.deltaY > 0 ? -0.05 : 0.05;
                                    player.setVolume(
                                        Math.max(
                                            0,
                                            Math.min(1, player.volume + delta),
                                        ),
                                    );
                                }}
                            >
                                <button
                                    class="text-gray-400 hover:text-white transition-colors"
                                    onclick={() => player.toggleMute()}
                                    aria-label="Toggle mute"
                                >
                                    {#if player.volume === 0}
                                        <VolumeX size={20} />
                                    {:else if player.volume < 0.33}
                                        <Volume size={20} />
                                    {:else if player.volume < 0.66}
                                        <Volume1 size={20} />
                                    {:else}
                                        <Volume2 size={20} />
                                    {/if}
                                </button>
                                <div class="w-18">
                                    <Slider
                                        value={player.volume}
                                        onValueChange={(val) =>
                                            player.setVolume(val)}
                                    />
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                {#if true}
                    <div class="relative overflow-hidden">
                        <LyricsView
                            trackId={track.id ?? 0}
                            position={player.position}
                            isPlaying={player.isPlaying}
                            durationSec={player.duration}
                            onSeek={(sec: number) => player.seek(sec)}
                        />
                    </div>
                {/if}
            </div>
        {:else}
            <div class="flex flex-col items-center justify-center h-full gap-4">
                <div
                    class="size-20 rounded-full bg-white/5 flex items-center justify-center"
                >
                    <Music2 size={40} class="text-gray-200" />
                </div>
                <div class="text-center">
                    <p class="text-lg font-medium text-gray-300">
                        Nothing Playing
                    </p>
                    <p class="text-sm text-gray-600 mt-1">
                        Play music from your library
                    </p>
                </div>
                <button
                    onclick={exit}
                    class="mt-2 px-4 py-2 rounded-full bg-white/5 hover:bg-white/10 backdrop-blur-sm text-sm text-gray-200 hover:text-white transition-all border border-white/10"
                >
                    Browse Library
                </button>
            </div>
        {/if}
    </div>
</div>

<style>
    .bg-image {
        will-change: transform opacity;
        animation: slow-zoom 30s ease-in-out infinite alternate;
        animation-play-state: paused;
    }

    .bg-image.playing {
        animation-play-state: running;
    }

    @keyframes slow-zoom {
        0% {
            transform: rotate(0deg)  scale(1);
            opacity: 0.8;
        }
        50% {
            transform: rotate(90deg) scale(1.3);
            opacity: 0.9;
        }
        100% {
            transform: rotate(180deg) scale(0.8);
            opacity: 1;
        }
    }
</style>
