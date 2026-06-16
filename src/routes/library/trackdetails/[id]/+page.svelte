<script lang="ts">
    import { page } from "$app/state";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import { player } from "$lib/player.svelte";
    import type { TrackDetails } from "$lib/types";
    import { Play, Pause, Heart, Music, Calendar, Clock, BarChart2, Folder, Info, AlertCircle } from "@lucide/svelte";

    let trackId = $derived(parseInt(page.params.id ?? "0"));
    let details = $state<TrackDetails | null>(null);
    let loading = $state(true);
    let error = $state<string | null>(null);

    async function loadDetails() {
        loading = true;
        error = null;
        try {
            details = await invoke<TrackDetails>("get_track_details", { id: trackId });
        } catch (e) {
            console.error("Failed to load track details", e);
            error = "Failed to load track details.";
        } finally {
            loading = false;
        }
    }

    function play() {
        if (!details) return;
        player.play(details);
    }

    function toggleFavorite() {
        if (!details) return;
        player.toggleFavorite(details);
        details.is_favorite = !details.is_favorite;
    }

    function formatDuration(seconds: number) {
        const mins = Math.floor(seconds / 60);
        const secs = seconds % 60;
        return `${mins}:${secs.toString().padStart(2, '0')}`;
    }

    onMount(loadDetails);
</script>

<div class="p-8 pb-32 max-w-4xl mx-auto">
    {#if loading}
        <div class="flex items-center justify-center py-40">
            <div class="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-secondary"></div>
        </div>
    {:else if error}
        <div class="flex flex-col items-center justify-center py-20 text-gray-500">
            <AlertCircle size={64} class="mb-4 opacity-20" />
            <p class="text-xl font-medium">{error}</p>
            <button
                onclick={loadDetails}
                class="mt-4 px-6 py-2 bg-neutral-800 text-white rounded-full hover:bg-neutral-700 transition-colors"
            >
                Retry
            </button>
        </div>
    {:else if details}
        <div class="flex flex-col md:flex-row gap-12 items-center md:items-end mb-16">
            <div class="w-64 h-64 rounded-3xl bg-neutral-800 shadow-2xl flex items-center justify-center overflow-hidden">
                <Music size={80} class="text-neutral-700" />
            </div>

            <div class="flex flex-col gap-4 flex-1 text-center md:text-left">
                <p class="text-sm font-bold uppercase tracking-wider text-secondary">Song Details</p>
                <h1 class="text-5xl font-black text-white">{details.title}</h1>
                <div class="flex flex-wrap items-center gap-x-6 gap-y-2 text-gray-400 justify-center md:justify-start">
                    <span class="hover:text-white transition-colors cursor-pointer">{details.artist.map(a => a.name).join(", ") || "Unknown Artist"}</span>
                    <span class="w-1 h-1 rounded-full bg-gray-600"></span>
                    <span class="hover:text-white transition-colors cursor-pointer">{details.album.map(a => a.name).join(", ") || "Unknown Album"}</span>
                </div>

                <div class="flex items-center gap-4 mt-4 justify-center md:justify-start">
                    <button
                        onclick={play}
                        class="w-16 h-16 bg-secondary text-black rounded-full flex items-center justify-center hover:scale-105 transition-transform shadow-xl"
                    >
                        {#if player.currentTrack?.id === details.id && player.isPlaying}
                            <Pause size={32} fill="currentColor" />
                        {:else}
                            <Play size={32} fill="currentColor" />
                        {/if}
                    </button>
                    <button
                        onclick={toggleFavorite}
                        class="w-16 h-16 bg-neutral-800 text-gray-400 rounded-full flex items-center justify-center hover:text-secondary hover:bg-neutral-700 transition-all border border-neutral-700"
                        class:text-secondary={details.is_favorite}
                    >
                        <Heart size={24} fill={details.is_favorite ? "currentColor" : "none"} />
                    </button>
                </div>
            </div>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div class="bg-neutral-900/50 rounded-2xl p-6 border border-neutral-800">
                <h3 class="text-lg font-bold text-white mb-6 flex items-center gap-2">
                    <Info size={18} class="text-secondary" /> General Information
                </h3>
                <div class="flex flex-col gap-4">
                    <div class="flex justify-between items-center">
                        <span class="text-gray-400 flex items-center gap-2"><Clock size={16} /> Duration</span>
                        <span class="text-white font-medium">{formatDuration(details.duration_seconds)}</span>
                    </div>
                    <div class="flex justify-between items-center">
                        <span class="text-gray-400 flex items-center gap-2"><Music size={16} /> Genre</span>
                        <span class="text-white font-medium">{details.genre.map(g => g.name).join(", ") || "Unknown"}</span>
                    </div>
                    <div class="flex justify-between items-center">
                        <span class="text-gray-400 flex items-center gap-2"><Calendar size={16} /> Added on</span>
                        <span class="text-white font-medium">{new Date(details.mtime * 1000).toLocaleDateString()}</span>
                    </div>
                </div>
            </div>

            <div class="bg-neutral-900/50 rounded-2xl p-6 border border-neutral-800">
                <h3 class="text-lg font-bold text-white mb-6 flex items-center gap-2">
                    <BarChart2 size={18} class="text-secondary" /> Playback Stats
                </h3>
                <div class="flex flex-col gap-4">
                    <div class="flex justify-between items-center">
                        <span class="text-gray-400">Total Plays</span>
                        <span class="text-white font-medium">{details.play_count}</span>
                    </div>
                    <div class="flex justify-between items-center">
                        <span class="text-gray-400">Total Skips</span>
                        <span class="text-white font-medium">{details.skipped_count}</span>
                    </div>
                    <div class="flex justify-between items-center">
                        <span class="text-gray-400">Last Played</span>
                        <span class="text-white font-medium">
                            {details.last_played_at ? new Date(details.last_played_at).toLocaleString() : "Never"}
                        </span>
                    </div>
                </div>
            </div>

            <div class="md:col-span-2 bg-neutral-900/50 rounded-2xl p-6 border border-neutral-800">
                <h3 class="text-lg font-bold text-white mb-4 flex items-center gap-2">
                    <Folder size={18} class="text-secondary" /> File Information
                </h3>
                <p class="text-gray-400 text-sm break-all bg-black/30 p-4 rounded-lg font-mono">
                    {details.path}
                </p>
            </div>
        </div>
    {/if}
</div>
