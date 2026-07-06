<script>
    import { onMount } from "svelte";
    import { Play, Shuffle, FolderInput } from "@lucide/svelte";
    import Button from "$components/ui/button/button.svelte";
    import { importAudioLibrary, getStatsOverview } from "$lib/commands.svelte";
    import { player } from "$lib/player.svelte";
    import { invoke } from "@tauri-apps/api/core";

    let { hasMusic } = $props();

    let totalTracks = $state(0);
    let totalArtists = $state(0);
    let totalAlbums = $state(0);
    let hasRecentTracks = $state(false);

    onMount(async () => {
        if (hasMusic) {
            try {
                const [overview, recent] = await Promise.all([
                    getStatsOverview("all_time"),
                    invoke("get_recently_played", { limit: 1 }),
                ]);
                totalTracks = overview.total_tracks;
                totalArtists = overview.total_artists;
                totalAlbums = overview.total_albums;
                hasRecentTracks = recent.length > 0;
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

    async function resumeListening() {
        if (player.currentTrack) {
            if (player.isPlaying) return;
            await player.playPause();
        } else {
            try {
                const tracks = await invoke("get_recently_played", {
                    limit: 1,
                });
                if (tracks.length > 0) {
                    await player.play(tracks, { type: "Other" }, 0);
                } else {
                    await playAllTracks();
                }
            } catch (e) {
                console.error("Failed to resume", e);
                await playAllTracks();
            }
        }
    }

    async function randomMix() {
        try {
            const tracks = await invoke("get_all_tracks", { sortBy: null });
            if (tracks.length > 0) {
                if (!player.shuffleEnabled) await player.toggleShuffle();
                const randomIndex = Math.floor(Math.random() * tracks.length);
                await player.play(tracks, { type: "Other" }, randomIndex);
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
            class="text-3xl md:text-5xl lg:text-7xl font-black text-light mb-4 tracking-tight drop-shadow-2xl font-satoshi"
        >
            <div class="mb-3">Your Library</div>
            Your
            <span
                class="bg-clip-text bg-linear-to-br from-blue-500 to-purple-700 text-transparent"
                >Soundtrack</span
            >
        </h1>
        <p class="text md:text-xl text-foreground/80 font-medium mb-8 px-2">
            • {totalTracks.toLocaleString()} Songs • {totalArtists.toLocaleString()}
            Artists • {totalAlbums.toLocaleString()} Albums
        </p>

        <div class="flex items-center gap-4">
            {#if hasMusic}
                {#if hasRecentTracks}
                    <Button
                        size="xl"
                        class="font-semibold roounded-full"
                        onclick={resumeListening}
                    >
                        <Play class="w-5 h-5 fill-current" />
                        Resume Listening
                    </Button>
                {:else}
                    <Button
                        size="xl"
                        class="font-semibold roounded-full"
                        onclick={playAllTracks}
                    >
                        <Play class="w-5 h-5 fill-current" />
                        Play All
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
