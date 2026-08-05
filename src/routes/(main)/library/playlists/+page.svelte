<script lang="ts">
    import Dialog from "$components/Dialog.svelte";
    import { Button } from "$components/ui/button/index.js";
    import PlaylistCard from "$components/ui/Card/PlaylistCard.svelte";
    import { Input } from "$components/ui/input/index.js";
    import SortControl from "$components/ui/SortControl.svelte";
    import { store } from "$lib/stores.svelte";
    import type { CollectionSortDir, CollectionSortField } from "$lib/utils";
    import { sortCollectionItems } from "$lib/utils";
    import { Plus } from "@lucide/svelte";

    let showCreateModal = $state(false);
    let newPlaylistName = $state("");
    let sortField = $state<CollectionSortField>("name");
    let sortDir = $state<CollectionSortDir>("asc");

    let sortedPlaylists = $derived(
        sortCollectionItems(store.playlists, sortField, sortDir),
    );

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
        <h1 class="text-7xl font-black text-white">Playlists</h1>
        <div class="flex items-center gap-3">
            <SortControl
                sortKey="playlists"
                bind:field={sortField}
                bind:dir={sortDir}
                options={[
                    { value: "name", label: "Name" },
                    { value: "added_at", label: "Date Added" },
                    { value: "last_played_at", label: "Recently Played" },
                    { value: "total_plays", label: "Most Played" },
                    { value: "track_count", label: "Track Count" },
                ]}
            />
            <Button
                onclick={() => (showCreateModal = true)}
                title="Create New Playlist"
                size="lg"
            >
                <Plus class="w-4 h-4" />
                Create New Playlist
            </Button>
        </div>
    </div>

    <div class="flex flex-wrap w-full">
        {#each sortedPlaylists as playlist}
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

    <Dialog title="Create New Playlist" open={showCreateModal}>
        <Input
            type="text"
            placeholder="Playlist name"
            bind:value={newPlaylistName}
            class="w-full mb-8"
            onkeydown={(e) => e.key === "Enter" && createPlaylist()}
        />

        {#snippet Footer()}
            <div class="flex gap-4 justify-end">
                <Button
                    variant="secondary"
                    onclick={() => (showCreateModal = false)}
                >
                    Cancel
                </Button>
                <Button onclick={createPlaylist}>Create</Button>
            </div>
        {/snippet}
    </Dialog>
</div>
