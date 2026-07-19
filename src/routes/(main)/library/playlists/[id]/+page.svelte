<script lang="ts">
    import { page } from "$app/state";
    import TrackList from "$components/ui/TrackList.svelte";
    import TrackListSmall from "$components/ui/TrackListSmall.svelte";
    import Button from "$components/ui/button/button.svelte";
    import { store } from "$lib/stores.svelte";
    import type { Track } from "$lib/types";
    import { Music, Music2, Search, X } from "@lucide/svelte";
    import Fuse from "fuse.js";
    import { fade, slide } from "svelte/transition";
    import { flip } from "svelte/animate";
    import { formatDuration } from "$lib/utils";
    import { onMount } from "svelte";
    import type { Playlist } from "$lib/types";
    import PlaylistCoverArt from "$components/ui/PlaylistCoverArt.svelte";
    import EditPlaylistDialog from "$components/ui/Dialog/EditPlaylistDialog.svelte";

    let playlistId = $derived(Number(page.params.id));
    let playlist: Playlist = $derived(
        playlistId === -1
            ? {
                  id: -1,
                  name: "All Tracks",
              }
            : (store.playlists.find((p) => p.id === playlistId) ?? {
                  id: playlistId,
                  name: "Unknown Playlist",
              }),
    );

    let searchResults = $state<Track[]>([]);
    let searchQuery = $state("");
    let showAddMore = $state(false);
    let fuse: Fuse<Track>;
    let editOpen = $state(false);

    let tracks = $derived(
        playlistId === -1 ? store.tracks : store.tracksByPlaylist(playlistId),
    );

    function loadFuse() {
        fuse = new Fuse(store.tracks, {
            keys: [
                { name: "title", weight: 1 },
                { name: "artists.name", weight: 0.5 },
                { name: "album.name", weight: 0.5 },
            ],
            threshold: 0.3,
        });
    }

    function addTrackToPlaylist(track: Track) {
        store.addTrackToPlaylist(track.id, playlistId);
        searchResults = searchResults.filter((t) => t.id !== track.id);
    }

    onMount(() => {
        if (tracks.length === 0) {
            showAddMore = true;
        }
    });

    $effect(() => {
        if (searchQuery.length >= 1) {
            if (!fuse) {
                loadFuse();
            }
            if (searchQuery.trim() === "") {
                searchResults = [];
                return;
            }
            const trackIds = new Set(tracks.map((t) => t.id));
            searchResults =
                fuse
                    ?.search(searchQuery)
                    .filter((result) => !trackIds.has(result.item.id))
                    .slice(0, 10)
                    .map((result) => result.item) ?? [];
        }
    });

</script>

<div
    class="fixed inset-0 h-screen w-screen blur-[180px] brightness-90 px-30 py-10"
>
    <PlaylistCoverArt {playlist} playlistTracks={tracks} />
</div>

<div class="flex flex-col p-5 z-1 isolate">
    <div class="flex gap-4 mb-4">
        <button
            class="w-42 lg:w-58 h-42 lg:h-58 rounded-2xl shadow-xl shadow-black/40 overflow-clip"
            onclick={() => editOpen = true}
        >
            <PlaylistCoverArt {playlist} playlistTracks={tracks} />
        </button>
        <div class="flex flex-col justify-end ml-4 py-1">
            <h1
                class="text-3xl md:text-5xl lg:text-7xl xl:text-8xl drop-shadow-lg font-black font-switzer line-clamp-2"
                onclick={() => editOpen = true}
            >
                {playlist.name}
            </h1>
            <div class="flex font-mono text-gray-300 gap-2 items-center">
                <span class="">
                    {tracks.length} songs
                </span>
                {#if tracks.length > 0}
                    <span class="text-gray-300">
                        {formatDuration(
                            tracks.reduce(
                                (acc, track) => acc + track.duration_seconds,
                                0,
                            ),
                        )}
                    </span>
                {/if}
            </div>
        </div>
    </div>

    {#if tracks.length > 0}
        <TrackList
            {tracks}
            context={{
                type: "Playlist",
                id: playlistId,
                name: playlist.name,
                coverArt: playlist.cover_art,
            }}
        />
    {:else}
        <div class="mt-12 lg:mt-24">
            <span
                transition:slide={{ duration: 600 }}
                class="flex items-center justify-center gap-4"
            >
                <Music2 strokeWidth={3} size={30} />
                <h3 class="text-2xl lg:text-4xl font-bold text-gray-200">
                    Silence isn't a playlist.
                </h3>
            </span>
            <h4
                transition:slide={{ duration: 600 }}
                class="text-sm lg:text-lg font-medium text-gray-300 text-center"
            >
                Search your library to add your first track.
            </h4>
        </div>
    {/if}

    {#if showAddMore}
        <div class="flex justify-between items-center w-full pr-10">
            <div
                class="flex items-center gap-2 bg-white/5 backdrop-blur-xl shadow-lg px-4 py-1 border-2 rounded-full w-1/4"
            >
                <Search size={16} class="text-gray-400" />
                <input
                    type="text"
                    placeholder="Search here..."
                    class="w-full py-3 outline-none bg-transparent text-white placeholder-gray-400 text-sm transition-all duration-300"
                    bind:value={searchQuery}
                />
                <button
                    onclick={() => {
                        searchResults = [];
                        searchQuery = "";
                    }}
                >
                    <X size={14} class="text-gray-400 hover:text-white" />
                </button>
            </div>
            <button
                onclick={() => {
                    showAddMore = false;
                    searchResults = [];
                }}><X size={32} class="text-gray-300" /></button
            >
        </div>
    {:else}
        <div class="flex justify-end">
            <Button
                variant="secondary"
                onclick={() => {
                    showAddMore = true;
                }}>Add more</Button
            >
        </div>
    {/if}

    {#if searchResults.length > 0}
        <div class="flex-col gap-2 mt-4 p-1 w-full px-5">
            {#each searchResults as track (track.id)}
                <div
                    class="flex items-center gap-2 p-1 rounded-xl hover:bg-white/5 transition-colors"
                    animate:flip={{ duration: 200 }}
                >
                    <TrackListSmall {track} styled={false} />
                    <Button onclick={() => addTrackToPlaylist(track)}
                        >Add</Button
                    >
                </div>
            {/each}
        </div>
    {/if}
</div>
<EditPlaylistDialog
    bind:open={editOpen}
    playlistId={playlistId}
    name={playlist.name}
    coverArt={playlist.cover_art}
/>