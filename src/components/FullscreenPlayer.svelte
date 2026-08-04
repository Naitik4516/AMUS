<script lang="ts">
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { Heart, X, Music2, Maximize2, MicVocal } from "@lucide/svelte";
    import { player } from "$lib/player.svelte";
    import { store } from "$lib/stores.svelte";
    import { formatDurationColon } from "$lib/utils";
    import { fullscreen } from "$lib/fullscreen.svelte";
    import Slider from "$components/ui/Slider.svelte";
    import LyricsView from "./LyricsView.svelte";
    import TransportControls from "./TransportControls.svelte";
    import VolumeControl from "./VolumeControl.svelte";
    import Button from "./ui/button/button.svelte";
    import { fade } from "svelte/transition";
    import { gsap } from "gsap";
    import { Flip } from "gsap/Flip";

    gsap.registerPlugin(Flip);

    let { onExit = () => {} } = $props();

    let coverUrl = $derived(
        player.currentTrack?.cover_art
            ? store.getImageSrc(player.currentTrack.cover_art)
            : null,
    );

    let showLyrics = $state(false);

    let flipTarget: HTMLDivElement | null = $state(null);
    let flipSnapshot: Flip.FlipState | null = null;
    let prevLyrics = false;

    $effect.pre(() => {
        if (showLyrics !== prevLyrics) {
            if (flipTarget) {
                flipSnapshot = Flip.getState(flipTarget);
            }
            prevLyrics = showLyrics;
        }
    });

    $effect(() => {
        if (flipSnapshot && flipTarget) {
            Flip.from(flipSnapshot, {
                targets: flipTarget,
                duration: 0.55,
                ease: "power3.inOut",
                scale: true,
                clearProps: "transform",
            });
            flipSnapshot = null;
        }
    });

    let displayPosition = $derived(Math.round(player.position * 4) / 4);

    function exit() {
        getCurrentWindow().setFullscreen(false);
        fullscreen.active = false;
        onExit();
    }

    $effect(() => {
        getCurrentWindow().setFullscreen(true);
        return () => {
            getCurrentWindow().setFullscreen(false);
        };
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

<div class="fixed inset-0 z-50 bg-neutral-900" out:fade>
    <div class="absolute inset-0 -z-10" in:fade={{ delay: 300 }}>
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

    <div class="z-10 h-full" in:fade={{ duration: 300 }}>
        {#if player.currentTrack}
            {@const track = player.currentTrack}
            <div
                bind:this={flipTarget}
                class="{!showLyrics
                    ? 'flex justify-center'
                    : 'grid grid-cols-[2fr_3fr]'} h-full"
            >
                <div
                    class="flex flex-col items-center justify-center gap-6 px-8 py-16"
                >
                    <div
                        class="size-80 rounded-3xl shadow-2xl overflow-hidden shrink-0 inset-shadow-sm"
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
                            class="text-4xl font-black text-white font-switzer truncate"
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
                        class="{!showLyrics
                            ? 'w-lg'
                            : 'w-md'}  flex flex-col gap-5 items-center px-4"
                    >
                        <div class="flex w-full items-center gap-3">
                            <span
                                class="text-xs font-medium text-gray-300 w-10 text-right tabular-nums"
                            >
                                {formatDurationColon(displayPosition)}
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
                            <TransportControls size="lg" />
                        </div>

                        <div
                            class="flex items-center w-full justify-evenly mt-3"
                        >
                            <Button
                                variant="outline"
                                size="icon-xl"
                                onclick={toggleFavorite}
                                aria-label="Toggle favorite"
                            >
                                <Heart
                                    size={20}
                                    class={track.is_favorite
                                        ? "fill-rose-600 text-rose-600"
                                        : ""}
                                />
                            </Button>
                            <Button
                                variant="outline"
                                size="icon-xl"
                                onclick={() => (showLyrics = !showLyrics)}
                                class="text-gray-300 hover:text-white transition-colors"
                                aria-label="Show lyrics"
                            >
                                <MicVocal size={20} />
                            </Button>
                            <div class="bg-white/5 px-6 py-4 rounded-full">
                                <VolumeControl width="w-20" />
                            </div>
                        </div>
                    </div>
                </div>
                {#if showLyrics}
                    <div
                        class="relative overflow-hidden"
                        in:fade={{ duration: 400, delay: 100 }}
                        out:fade={{ duration: 200 }}
                    >
                        <LyricsView
                            trackId={track.id ?? 0}
                            position={displayPosition}
                            isPlaying={player.isPlaying}
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
        will-change: transform, opacity;
        animation: animate-bg 24s ease-in-out infinite alternate;
        animation-play-state: paused;
    }

    .bg-image.playing {
        animation-play-state: running;
    }

    @keyframes animate-bg {
        0% {
            transform: scale(1) rotate(45deg);
            opacity: 0.9;
        }
        50% {
            transform: scale(1.3) rotate(0deg);
            opacity: 1;
        }
        100% {
            transform: scale(0.8) rotate(-90deg);
            opacity: 0.8;
        }
    }
</style>
