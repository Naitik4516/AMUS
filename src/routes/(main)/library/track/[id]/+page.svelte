<script lang="ts">
    import { player } from "$lib/player.svelte";
    import {
        Play,
        Pause,
        Heart,
        Music,
        Calendar,
        Clock,
        ChartNoAxesColumn,
        Folder,
        Info,
    } from "@lucide/svelte";
    import Artist from "$components/icons/Artist.svelte";
    import type { PageProps } from "./$types";
    import { formatDuration } from "$lib/utils";
    import { invoke } from "@tauri-apps/api/core";
    import { store } from "$lib/stores.svelte";
    import { invalidate } from "$app/navigation";

    let { data }: PageProps = $props();
    let track = $derived(data.trackDetails);

    async function toggleFavorite() {
        try {
            await invoke<boolean>("toggle_favorite", {
                id: track.id,
            });
            invalidate("app:track-details");
        } catch (e) {
            console.error("Failed to toggle favorite", e);
        }
    }

    $inspect(track, "track");
</script>

<div class="p-8 pb-32 max-w-5xl mx-auto">
    <div
        class="flex flex-col md:flex-row gap-12 items-center md:items-end mb-16"
    >
        <div
            class="w-82 h-82 rounded-3xl bg-muted shadow-2xl flex items-center justify-center overflow-hidden"
        >
            {#if track.cover_art}
                <img
                    src={track.cover_art
                        ? store.getImageSrc(track.cover_art)
                        : "/PhonographRecord.webp"}
                    alt={track.title}
                    class="w-full h-full object-cover"
                />
            {:else}
                <Music size={80} class="text-muted-foreground" />
            {/if}
        </div>

        <div
            class="flex flex-col gap-4 flex-1 text-center md:text-left md:pb-2"
        >
            <div>
                <p
                    class="text-sm font-bold uppercase tracking-wider text-muted-foreground"
                >
                    Song track
                </p>
                <h1 class="text-5xl font-black text-white font-switzer">
                    {track.title}
                </h1>
            </div>
            <div class="font-medium  text-gray-200 flex gap-2" >
                <span>
                    • {track.album ? track.album.name : "Unknown Album"}
                </span>
                <span class="proportional-nums">
                    • {track.year}
                </span>
            </div>
            <div class="flex flex-wrap gap-3">
                {#each track.artists as artist (artist.id)}
                    <div class="flex gap-1 items-center font-medium">
                        {#if artist.profile_image}
                            <img
                                src={store.getImageSrc(
                                    artist.profile_image,
                                    "artist",
                                )}
                                alt={artist.name}
                                class="w-6 h-6 rounded-full object-cover"
                            />
                        {:else}
                            <Artist size={24} class="text-gray-400" />
                        {/if}
                        <a
                            href={`/library/artists/${artist.id}`}
                            class="hover:text-white text-sm transition-colors"
                            >{artist.name}</a
                        >
                    </div>
                {/each}
            </div>

            <div
                class="flex items-center gap-4 mt-4 justify-center md:justify-start"
            >
                <button
                    onclick={() => player.play([track])}
                    class="size-16 bg-accent text-accent-foreground rounded-full flex items-center justify-center hover:scale-105 transition-transform shadow-xl"
                >
                    {#if player.currentTrack?.id === track.id && player.isPlaying}
                        <Pause size={26} fill="currentColor" />
                    {:else}
                        <Play size={26} fill="currentColor" />
                    {/if}
                </button>
                <button
                    onclick={() => toggleFavorite()}
                    class="size-16 rounded-full border bg-white/5 flex items-center justify-center hover:scale-105 transition-transform shadow-xl"
                >
                    <Heart
                        size={26}
                        class={track.is_favorite
                            ? "text-rose-600 fill-rose-600"
                            : "text-gray-300"}
                    />
                </button>
            </div>
        </div>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
        <div
            class="bg-card/30 background-blur-md rounded-3xl p-6 border border-border shadow-xl"
        >
            <h3
                class="text-lg font-bold text-white mb-6 flex items-center gap-2"
            >
                <Info size={18}  /> General Information
            </h3>
            <div class="flex flex-col gap-4">
                <div class="flex justify-between items-center">
                    <span class="text-gray-400 flex items-center gap-2"
                        ><Clock size={16} /> Duration</span
                    >
                    <span class="text-white font-medium"
                        >{formatDuration(track.duration_seconds)}</span
                    >
                </div>
                <div class="flex justify-between items-center">
                    <span class="text-gray-400 flex items-center gap-2"
                        ><Calendar size={16} /> Added on</span
                    >
                    <span class="text-white font-medium"
                        >{new Date(
                            track.mtime * 1000,
                        ).toLocaleDateString()}</span
                    >
                </div>
            </div>
        </div>

        <div
            class="bg-card/20 background-blur-md rounded-3xl p-6 border border-border shadow-xl "
        >
            <h3
                class="text-lg font-bold text-white mb-6 flex items-center gap-2"
            >
                <ChartNoAxesColumn size={18} /> Playback Stats
            </h3>
            <div class="flex flex-col gap-4">
                <div class="flex justify-between items-center">
                    <span class="text-gray-400">Total Plays</span>
                    <span class="text-white font-medium"
                        >{track.play_count}</span
                    >
                </div>
                <div class="flex justify-between items-center">
                    <span class="text-gray-400">Total Skips</span>
                    <span class="text-white font-medium"
                        >{track.skipped_count}</span
                    >
                </div>
                <div class="flex justify-between items-center">
                    <span class="text-gray-400">Last Played</span>
                    <span class="text-white font-medium">
                        {track.last_played_at
                            ? new Date(track.last_played_at).toLocaleString()
                            : "Never"}
                    </span>
                </div>
            </div>
        </div>

        <div
            class="md:col-span-2 bg-card/20 background-blur-md rounded-3xl p-6 border border-border shadow-xl"
        >
            <h3
                class="text-lg font-bold text-white mb-4 flex items-center gap-2"
            >
                <Folder size={18} /> File Information
            </h3>
            <p
                class="text-gray-400 text-sm break-all bg-black/30 p-4 rounded-lg font-mono"
            >
                {track.path}
            </p>
        </div>
    </div>
</div>
