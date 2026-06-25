<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import PlaylistCard from "$components/ui/Card/PlaylistCard.svelte";
    import { Plus } from "@lucide/svelte";
    import type { PageProps } from "./$types";
    import { fly } from "svelte/transition";

    let { data }: PageProps = $props();
    let playlists = $derived(data.playlists);

    let showCreateModal = $state(false);
    let newPlaylistName = $state("");

    async function createPlaylist() {
        if (!newPlaylistName.trim()) return;
        try {
            await invoke("create_playlist", { name: newPlaylistName });
            newPlaylistName = "";
            showCreateModal = false;
        } catch (e) {
            console.error("Failed to create playlist", e);
        }
    }
</script>

<div class="p-8">
    <div class="flex items-center justify-between mb-8">
        <h1 class="text-3xl font-black text-white">Playlists</h1>
        <button
            onclick={() => (showCreateModal = true)}
            class="flex items-center gap-2 px-4 py-2 bg-neutral-800 text-white font-bold rounded-full hover:bg-neutral-700 transition-colors"
            transition:fly={{ y: -20, duration: 300 }}
        >
            <Plus size={20} /> New Playlist
        </button>
    </div>

    <div
        class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 gap-6"
    >
        {#each playlists as playlist}
            <PlaylistCard {...playlist} />
        {/each}
    </div>

    {#if playlists.length === 0}
        <p class="text-gray-500 text-sm mt-6 text-center">
            No custom playlists yet. Click "New Playlist" to create one.
        </p>
    {/if}

    {#if showCreateModal}
        <div
            class="fixed inset-0 z-10 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm"
        >
            <div
                class="bg-neutral-900 border border-neutral-800 rounded-2xl p-8 w-full max-w-md shadow-2xl"
            >
                <h2 class="text-2xl font-bold mb-6">Create New Playlist</h2>
                <input
                    type="text"
                    placeholder="Playlist name"
                    bind:value={newPlaylistName}
                    class="w-full bg-neutral-800 border border-neutral-700 rounded-lg px-4 py-3 outline-none focus:border-secondary mb-8 text-lg"
                    onkeydown={(e) => e.key === "Enter" && createPlaylist()}
                />
                <div class="flex gap-4 justify-end">
                    <button
                        onclick={() => (showCreateModal = false)}
                        class="px-6 py-2 text-gray-400 hover:text-white transition-colors"
                    >
                        Cancel
                    </button>
                    <button
                        onclick={createPlaylist}
                        class="px-8 py-2 bg-accent text-black font-bold rounded-full hover:scale-105 transition-transform"
                    >
                        Create
                    </button>
                </div>
            </div>
        </div>
    {/if}
</div>
