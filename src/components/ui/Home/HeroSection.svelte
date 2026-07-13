<script lang="ts">
    import { onMount } from "svelte";
    import { Play, Shuffle, FolderInput } from "@lucide/svelte";
    import Button from "$components/ui/button/button.svelte";
    import { importAudioLibrary, getStatsOverview } from "$lib/commands.svelte";
    import { player } from "$lib/player.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { loadSession, clearSession, hasSession } from "$lib/session.svelte";

    let { hasMusic } = $props();

    let totalTracks = $state(0);
    let totalArtists = $state(0);
    let totalAlbums = $state(0);
    let hasSessionState = $state(false);

    onMount(async () => {
        if (hasMusic) {
            try {
                const [overview, sessionExists] = await Promise.all([
                    getStatsOverview("all_time"),
                    hasSession(),
                ]);
                totalTracks = overview.total_tracks;
                totalArtists = overview.total_artists;
                totalAlbums = overview.total_albums;
                hasSessionState = sessionExists && !player.currentTrack;
            } catch (e) {
                console.error("Failed to load stats", e);
            }
        }
    });

    async function playAllTracks() {
        try {
            const tracks = await invoke("get_all_tracks", { sortBy: null });
            if (tracks.length > 0) {
                await player.play(tracks, { type: "Other" }, 0);
            }
        } catch (e) {
            console.error("Failed to play all tracks", e);
        }
    }

    import type { Track, TrackDetails } from "$lib/types";

    async function resumeSession() {
        const session = await loadSession();
        if (!session) {
            await resumeFallback();
            return;
        }

        let contextTracks: Track[] = [];
        try {
            switch (session.context_type) {
                case "ALBUM":
                    contextTracks = await invoke<Track[]>("get_tracks_by_album", { albumId: session.context_id });
                    break;
                case "PLAYLIST":
                    contextTracks = await invoke<Track[]>("get_tracks_by_playlist", { playlistId: session.context_id });
                    break;
                case "ARTIST":
                    contextTracks = await invoke<Track[]>("get_tracks_by_artist", { artistId: session.context_id });
                    break;
                case "FAVORITES":
                    contextTracks = await invoke<Track[]>("get_favorite_tracks");
                    break;
                default:
                    if (session.current_track_id) {
                        const details = await invoke<TrackDetails>("get_track_details", { id: session.current_track_id });
                        contextTracks = [details];
                    }
            }
        } catch (e) {
            console.error("Failed to fetch context tracks", e);
        }

        if (contextTracks.length === 0) {
            await resumeFallback();
            return;
        }

        let userQueueTracks: Track[] = [];
        if (session.user_queue_ids.length > 0) {
            try {
                const all = await invoke<Track[]>("get_all_tracks", { sortBy: null });
                const idSet = new Set(session.user_queue_ids);
                userQueueTracks = all.filter((t) => idSet.has(t.id));
                const idOrder = session.user_queue_ids;
                userQueueTracks.sort((a, b) => idOrder.indexOf(a.id) - idOrder.indexOf(b.id));
            } catch (e) {
                console.error("Failed to fetch user queue tracks", e);
            }
        }

        const startIndex = Math.min(session.context_position, contextTracks.length - 1);
        const sourceType = session.context_type;
        const sourceId = session.context_id;

        try {
            await invoke("restore_session", {
                contextTracks,
                sourceType,
                sourceId,
                startIndex: Math.max(0, startIndex),
                contextLabel: session.context_label,
                userQueueTracks,
                positionSec: session.position_sec,
                volume: session.volume,
                repeat: session.repeat,
                shuffle: session.shuffle,
            });
            await clearSession();
            hasSessionState = false;
        } catch (e) {
            console.error("Failed to restore session", e);
            await resumeFallback();
        }
    }

    async function resumeFallback() {
        try {
            const recent = await invoke<Track[]>("get_recently_played", { limit: 1 });
            if (recent.length > 0) {
                await player.play(recent, { type: "Other" }, 0);
            } else {
                await playAllTracks();
            }
        } catch (e) {
            console.error("Failed to resume fallback", e);
            await playAllTracks();
        }
    }

    async function randomMix() {
        try {
            const tracks = await invoke("get_all_tracks", { sortBy: null });
            if (tracks.length > 0) {
                const randomIndex = Math.floor(Math.random() * tracks.length);
                await player.play([tracks[randomIndex]]);
            }
        } catch (e) {
            console.error("Failed to play random mix", e);
        }
    }
</script>

<div
    class="relative w-full h-90 max-h-3/5 md:min-h-110 md:h-full rounded-[4rem] bg-linear-to-br from-secondary/60 via-secondary/20 to-slate-900 shadow-xl border mt-4 overflow-hidden group"
>
    <div
        class="absolute -right-8 md:right-0 top-1/2 -translate-y-1/2 w-[70%] sm:w-[50%] lg:w-[50%] h-[120%] pointer-events-none transition-transform duration-700 group-hover:scale-105 group-hover:-translate-x-4 group-hover:-rotate-3 z-0 opacity-90"
    >
        <div class="relative w-full h-full p-5">
            <img
                src="/headphones.png"
                alt="3D Headphones"
                class="w-full h-full object-contain object-right"
                style="filter: hue-rotate(var(--model-hue-rotate, 175deg))"
            />
        </div>
    </div>

    <div
        class="absolute inset-0 p-8 md:p-12 flex flex-col justify-end w-full md:w-2/3"
    >
        <span
            class="inline-block px-3 py-1 bg-white/10 backdrop-blur-md rounded-full text-xs font-bold tracking-wider text-light mb-4 border border-white/5 uppercase w-fit"
        >
            LOCAL LIBRARY
        </span>

        <h1
            class="text-3xl md:text-4xl lg:text-6xl xl:text-7xl font-black text-light mb-4 tracking-tight drop-shadow-2xl font-satoshi"
        >
            <div class="mb-3">Your Library</div>
            <div class="whitespace-nowrap">
                Your
                <span
                    class="bg-clip-text bg-linear-to-br from-blue-500 to-purple-700 text-transparent"
                    >Soundtrack</span
                >
            </div>
        </h1>
        <p class="text md:text-xl text-foreground/80 font-medium mb-8 px-2">
            • {totalTracks.toLocaleString()} Songs • {totalArtists.toLocaleString()}
            Artists • {totalAlbums.toLocaleString()} Albums
        </p>

        <div class="flex items-center gap-4">
            {#if hasMusic}
                {#if hasSessionState}
                    <Button
                        size="xl"
                        class="font-semibold roounded-full bg-accent text-black hover:bg-accent/80"
                        onclick={resumeSession}
                    >
                        <Play class="w-5 h-5 fill-current" />
                        Resume Session
                    </Button>
                {:else}
                    <Button
                        size="xl"
                        class="font-semibold roounded-full"
                        onclick={resumeFallback}
                    >
                        <Play class="w-5 h-5 fill-current" />
                        Resume Listening
                    </Button>
                {/if}
                <Button
                    size="xl"
                    class="font-semibold roounded-full"
                    variant="outline"
                    onclick={randomMix}
                >
                    <Shuffle size={20} />
                    Random Mix
                </Button>
            {:else}
                <Button
                    size="xl"
                    class="font-semibold roounded-full"
                    onclick={importAudioLibrary}
                >
                    <FolderInput size={20} />
                    Add Your Music
                </Button>
            {/if}
        </div>
    </div>
</div>
