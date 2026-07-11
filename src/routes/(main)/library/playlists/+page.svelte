<script lang="ts">
    import PlaylistCard from "$components/ui/Card/PlaylistCard.svelte";
    import { Plus } from "@lucide/svelte";
    import { fly, blur } from "svelte/transition";
    import { Button } from "$components/ui/button/index.js";
    import { Input } from "$components/ui/input/index.js";
    import { store } from "$lib/stores.svelte";


    let showCreateModal = $state(false);
    let newPlaylistName = $state("");

    async function createPlaylist() {
        if (!newPlaylistName.trim()) return;
        try {
            await store.createPlaylist(newPlaylistName);
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
        <Button
            onclick={() => (showCreateModal = true)}
            title="Create New Playlist"
            size="lg"
        >
            <Plus class="w-4 h-4" />
            Create New Playlist
        </Button>
    </div>

    <div
        class="flex flex-wrap w-full"
    >
        {#each store.playlists as playlist}
            <div class="mx-5 my-4">
            <PlaylistCard data={playlist} />
            </div>
        {/each}
    </div>

    {#if store.playlists.length === 0}
        <p class="text-gray-500 text-sm mt-6 text-center">
            No custom store.playlists yet. Click "New Playlist" to create one.
        </p>
    {/if}

    {#if showCreateModal}
        <div
            class="fixed inset-0 z-10 flex items-center justify-center p-4 bg-black/20"
            transition:blur={{ duration: 300 }}
        >
            <div
                class="bg-card border rounded-2xl p-5 w-full max-w-md shadow-2xl"
                transition:fly={{ y: 600, duration: 300 }}
            >
                <h2 class="text-2xl font-bold mb-6">Create New Playlist</h2>
                <Input
                    type="text"
                    placeholder="Playlist name"
                    bind:value={newPlaylistName}
                    class="w-full mb-8"
                    onkeydown={(e) => e.key === "Enter" && createPlaylist()}
                />
                <div class="flex gap-4 justify-end">
                    <Button
                        variant="secondary"
                        onclick={() => (showCreateModal = false)}
                    >
                        Cancel
                    </Button>
                    <Button onclick={createPlaylist}>Create</Button>
                </div>
            </div>
        </div>
    {/if}
</div>
