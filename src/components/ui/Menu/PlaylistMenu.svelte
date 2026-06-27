<script lang="ts">
    import { getPlaylists } from "$lib/data.svelte";
    import { onMount } from "svelte";
    import type { Playlist } from "$lib/types";
    import { Input } from "../input";
    import Button from "../button/button.svelte";
    import { getImageUrl } from "$lib/utils";
    import { Disc, Plus, CircleCheck, X } from "@lucide/svelte";
    import { toast } from "svelte-sonner";
    import type { Track } from "$lib/types";
    import { invoke } from "@tauri-apps/api/core";

    let { track }: { track: Track } = $props();

    let playlists = $state<Playlist[]>([]);
    let trackPlaylistIds = $state<Set<number>>(new Set());
    let searchQuery = $state("");
    let creating = $state(false);
    let newName = $state("");

    let filtered = $derived(
        playlists.filter((p) =>
            p.name.toLowerCase().includes(searchQuery.toLowerCase()),
        ),
    );

    onMount(() => loadPlaylists());

    async function loadPlaylists() {
        const result = await getPlaylists();
        playlists = result.playlists;
        const ids = await invoke<number[]>("get_track_playlist_ids", {
            trackId: track.id,
        });
        trackPlaylistIds = new Set(ids);
    }

    async function togglePlaylist(plId: number) {
        const wasIn = trackPlaylistIds.has(plId);
        try {
            if (wasIn) {
                await invoke("remove_track_from_playlist", {
                    trackId: track.id,
                    playlistId: plId,
                });
                trackPlaylistIds.delete(plId);
                toast.success("Removed from playlist");
            } else {
                await invoke("add_track_to_playlist", {
                    trackId: track.id,
                    playlistId: plId,
                });
                trackPlaylistIds.add(plId);
                toast.success("Added to playlist");
            }
            trackPlaylistIds = new Set(trackPlaylistIds);
        } catch (e) {
            console.error("Failed to update playlist", e);
            toast.error(
                wasIn
                    ? "Failed to remove from playlist"
                    : "Failed to add to playlist",
            );
        }
    }

    async function createPlaylist() {
        const name = newName.trim();
        if (!name) return;
        try {
            const id = await invoke<number>("create_playlist", { name });
            playlists = [...playlists, { id, name, coverArts: [] }];
            newName = "";
            creating = false;
            toast.success("Playlist created");
        } catch (e) {
            console.error("Failed to create playlist", e);
            toast.error("Failed to create playlist");
        }
    }

    function cancelCreate() {
        creating = false;
        newName = "";
    }
</script>

<div class="flex flex-col gap-2">
    <Input placeholder="Search playlists..." bind:value={searchQuery} />

    {#if creating}
        <form
            class="flex gap-2"
            onsubmit={(e) => {
                e.preventDefault();
                createPlaylist();
            }}
        >
            <Input
                placeholder="Playlist name"
                bind:value={newName}
                class="flex-1"
            />
            <Button type="submit" size="sm">Create</Button>
            <Button size="sm" variant="outline" onclick={cancelCreate}>
                <X size={14} />
            </Button>
        </form>
    {:else}
        <Button
            class="w-full"
            size="sm"
            variant="outline"
            onclick={() => (creating = true)}
        >
            <Plus />
            Create Playlist
        </Button>
    {/if}

    {#if filtered.length > 0}
        <ul class="max-h-60 overflow-y-auto">
            {#each filtered as playlist}
                <li>
                    <button
                        class="flex items-center w-full gap-3 rounded-xl px-3 py-1 text-zinc-200 transition-colors hover:bg-gray-400/10 hover:text-white cursor-pointer"
                        onclick={() => togglePlaylist(playlist.id)}
                    >
                        <div
                            class="aspect-square w-8 rounded-lg overflow-hidden bg-neutral-800 shadow-lg relative"
                        >
                            {#if !playlist.coverArts || playlist.coverArts.length === 0}
                                <div
                                    class="absolute inset-0 flex items-center justify-center"
                                >
                                    <Disc size={28} class="text-neutral-700" />
                                </div>
                            {:else if playlist.coverArts.length < 4}
                                {#await getImageUrl(playlist.coverArts[0])}
                                    <div
                                        class="absolute inset-0 bg-neutral-800 animate-pulse"
                                    ></div>
                                {:then url}
                                    <img
                                        src={url}
                                        alt={playlist.name}
                                        class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
                                    />
                                {/await}
                            {:else}
                                <div
                                    class="grid grid-cols-2 grid-rows-2 w-full h-full group-hover:scale-105 transition-transform duration-500"
                                >
                                    {#each playlist.coverArts.slice(0, 4) as art}
                                        {#await getImageUrl(art)}
                                            <div
                                                class="w-full h-full bg-neutral-800 animate-pulse"
                                            ></div>
                                        {:then url}
                                            <img
                                                src={url}
                                                alt=""
                                                class="w-full h-full object-cover"
                                            />
                                        {/await}
                                    {/each}
                                </div>
                            {/if}
                        </div>
                        <div class="flex-1 truncate text-left">
                            {playlist.name}
                        </div>
                        {#if trackPlaylistIds.has(playlist.id)}
                            <CircleCheck
                                size={24}
                                fill="var(--color-accent)"
                                stroke="black"
                                class="text-black shrink-0"
                            />
                        {/if}
                    </button>
                </li>
            {/each}
        </ul>
    {:else if playlists.length > 0}
        <p class="text-sm text-zinc-500 px-3">No playlists match your search</p>
    {:else}
        <p class="text-sm text-zinc-500 px-3">No playlists yet</p>
    {/if}
</div>
