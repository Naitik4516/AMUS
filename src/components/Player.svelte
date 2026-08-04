<script lang="ts">
    import {
        ListMusic,
        Heart,
        Music2,
        Maximize,
        MicVocal,
        X,
    } from "@lucide/svelte";
    import Slider from "./ui/Slider.svelte";
    import { player } from "$lib/player.svelte";
    import { formatDurationColon } from "$lib/utils";

    import { ui } from "$lib/shortcut-handler.svelte";
    import Marquee from "./ui/Marquee.svelte";
    import { slide } from "svelte/transition";
    import QueueView from "./QueueView.svelte";
    import LyricsView from "./LyricsView.svelte";
    import { store } from "$lib/stores.svelte";
    import { fullscreen } from "$lib/fullscreen.svelte";
    import Button from "./ui/button/button.svelte";
    import TrackMenu from "$components/ui/Menu/TrackMenu.svelte";
    import { openContextMenu } from "$lib/context-menu.svelte";
    import type { Context } from "$lib/types";
    import TransportControls from "./TransportControls.svelte";
    import VolumeControl from "./VolumeControl.svelte";

    let showQueue = $state(false);
    let showLyrics = $state(false);

    let displayPosition = $derived(Math.round(player.position * 4) / 4);

    let trackContext = $derived.by<Context>(() => {
        const t = player.currentTrack;
        if (!t) return null;
        const src = player.source;
        if (src?.type === "Playlist") {
            const p = store.playlistsById.get(src.id);
            return p
                ? {
                      type: "Playlist",
                      id: p.id,
                      name: p.name,
                      coverArt: p.cover_art,
                  }
                : null;
        }
        if (src?.type === "Artist") {
            const a = store.artistsById.get(src.id);
            return a
                ? {
                      type: "Artist",
                      id: a.id,
                      name: a.name,
                      profileImage: a.profile_image,
                      bannerImage: a.banner_image,
                  }
                : null;
        }
        if (src?.type === "Favorites") {
            return { type: "Favorites", name: "Favorites" };
        }
        return {
            type: "Album",
            id: t.album.id,
            name: t.album.name,
            coverArt: t.album.cover_art ?? null,
        };
    });

    $effect(() => {
        showQueue = ui.queueVisible;
    });

    async function toggleFavorite() {
        if (player.currentTrack) {
            await player.toggleFavorite(player.currentTrack);
        }
    }

    function handleContextMenu(e: MouseEvent) {
        e.preventDefault();
        openContextMenu(TrackMenu, {
            position: { type: "coordinates", x: e.clientX, y: e.clientY },
            track: player.currentTrack,
            context: trackContext,
        });
    }
</script>

{#if player.currentTrack}
    <div class="fixed bottom-0 left-0 w-full px-4 pb-3 z-15">
        <div
            class="grid grid-cols-3 items-center justify-between px-6 py-3 rounded-3xl relative bg-linear-to-br from-white/10 to-white/5 backdrop-blur-2xl backdrop-brightness-75 backdrop-saturate-200 ring-1 ring-white/10 shadow-[0_8px_32px_0_rgba(0,0,0,0.25)]"
            oncontextmenu={handleContextMenu}
            role="contentinfo"
        >
            <!-- Track Info -->
            <div
                class="flex items-center gap-4 pr-10 z-1"
                ondblclick={() => player.close()}
                role="group"
            >
                <div
                    class="w-15 h-15 rounded-lg bg-neutral-800 shadow-md flex items-center justify-center overflow-hidden shrink-0"
                >
                    {#if player.currentTrack?.cover_art}
                        <img
                            src={store.getImageSrc(
                                player.currentTrack.cover_art,
                            )}
                            alt=""
                            class="w-full h-full object-cover"
                        />
                    {:else}
                        <Music2 size={32} />
                    {/if}
                </div>
                <div class="flex flex-col overflow-hidden">
                    <Marquee>
                        <a
                            href="/library/track/{player.currentTrack?.id}"
                            class="font-bold truncate text-white hover:underline cursor-pointer inline-block text-lg"
                        >
                            {player.currentTrack?.title}
                        </a>
                    </Marquee>
                    <Marquee>
                        <p class=" text-gray-300 truncate -mt-2">
                            {#each player.currentTrack?.artists as artist, ai (artist.id)}
                                {#if ai > 0}
                                    <span>, </span>
                                {/if}
                                <a
                                    href="/library/artists/{artist.id}"
                                    class=" hover:text-white font-medium text-sm"
                                    >{artist.name}</a
                                >
                            {/each}
                        </p>
                    </Marquee>
                </div>
                <button
                    onclick={toggleFavorite}
                    class="ml-2 {player.currentTrack?.is_favorite
                        ? 'text-rose-600 fill-rose-600'
                        : 'text-gray-300'}  hover:text-secondary transition-colors"
                    aria-label={player.currentTrack?.is_favorite
                        ? "Remove from favorites"
                        : "Add to favorites"}
                >
                    <Heart
                        size={22}
                        class={player.currentTrack?.is_favorite
                            ? "text-rose-600 fill-rose-600"
                            : "text-gray-300"}
                    ></Heart>
                </button>
            </div>

            <!-- Controls -->
            <div class="flex flex-col items-center gap-2 z-1">
                <TransportControls />
                <div class="w-full flex items-center justify-center gap-3">
                    <span
                        class="text-[10px] font-medium text-gray-200 w-10 text-right"
                    >
                        {formatDurationColon(displayPosition)}
                    </span>
                    <Slider
                        value={player.progress}
                        onValueChange={(val) => {
                            if (player.currentTrack) {
                                let seekVal = val * player.duration;

                                player.seek(seekVal);
                            }
                        }}
                    />
                    <span class="text-[10px] font-medium text-gray-200 w-10">
                        {formatDurationColon(
                            player.currentTrack.duration_seconds,
                        )}
                    </span>
                </div>
            </div>

            <!-- Right Interactables -->
            <div class="flex items-center gap-4 justify-end z-1">
                <button
                    onclick={() => (showLyrics = !showLyrics)}
                    class="text-gray-300 hover:text-white transition-colors"
                    class:text-accent={showLyrics}
                    aria-label="Show lyrics"
                >
                    <MicVocal size={20} />
                </button>

                <button
                    onclick={() => (showQueue = !showQueue)}
                    class="text-gray-300 hover:text-white transition-colors"
                    class:text-accent={showQueue}
                    aria-label="Toggle queue"
                >
                    <ListMusic size={20} />
                </button>

                <VolumeControl />

                <button
                    onclick={() => (fullscreen.active = true)}
                    class="text-gray-300 hover:text-white transition-colors"
                    aria-label="Fullscreen"
                >
                    <Maximize size={20} />
                </button>
            </div>

            <div
                class="absolute bottom-full left-24 right-1 flex mb-3 justify-end items-end"
            >
                {#if showLyrics}
                    <div
                        class="relative h-[60vh] max-w-2/3 w-full bg-card/60 backdrop-saturate-200 backdrop-blur-3xl border-2 border-border/70 rounded-2xl shadow-2xl flex flex-col overflow-hidden mx-auto"
                        transition:slide
                    >
                        <Button
                            onclick={() => (showLyrics = false)}
                            class="text-gray-300 hover:text-white absolute top-1 right-1 "
                            variant="ghost"
                            size="icon"
                        >
                            <X size={20} />
                        </Button>
                        <div class="flex-1 overflow-hidden px-1 pb-1 pt-10">
                            <LyricsView
                                trackId={player.currentTrack?.id ?? 0}
                                position={displayPosition}
                                isPlaying={player.isPlaying}
                                onSeek={(sec: number) => player.seek(sec)}
                            />
                        </div>
                    </div>
                {/if}

                {#if showQueue}
                    <div
                        class="w-[30%] ml-3 h-[60vh] relative bg-card/60 backdrop-blur-2xl backdrop-saturate-200 border-2 border-border/70 rounded-2xl shadow-2xl overflow-hidden"
                        transition:slide
                    >
                        <QueueView bind:showQueue />
                    </div>
                {/if}
            </div>
        </div>
    </div>
{/if}
