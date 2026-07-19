<script lang="ts">
    import { Button } from "$components/ui/button/index.js";
    import { selectAndUploadImage } from "$lib/edit-helpers";
    import { store } from "$lib/stores.svelte";
    import { X, LoaderCircle, Pen } from "@lucide/svelte";
    import PlaylistCoverArt from "$components/ui/PlaylistCoverArt.svelte";
    import Dialog from "$components/Dialog.svelte";
    import { onMount } from "svelte";

    let {
        open = $bindable(false),
        playlistId = 0,
        name = "",
        coverArt = null,
    }: {
        open: boolean;
        playlistId: number;
        name: string;
        coverArt?: string | null;
    } = $props();

    let editName = $state("");
    let editCoverArt = $state<string | null>(null);
    let saving = $state(false);

    onMount(() => {
        editName = name;
        editCoverArt = coverArt;
    });

    async function pickCover() {
        const filename = await selectAndUploadImage("cover");
        console.log("Selected cover art:", filename);
        if (filename) {
            editCoverArt = filename;
        }
    }

    function removeCover() {
        editCoverArt = null;
    }

    async function save() {
        saving = true;
        try {
            await store.savePlaylist(playlistId, editName.trim(), editCoverArt);
            open = false;
        } catch (e) {
            console.error("Failed to update playlist", e);
        } finally {
            saving = false;
        }
    }

    $inspect("Cover Art", editCoverArt);
</script>

<Dialog bind:open title="Edit Playlist">
    <div class="flex gap-5 items-center">
        <div class="flex flex-col gap-2">
            <div class="relative w-42 h-42 rounded-3xl overflow-clip group">
                <PlaylistCoverArt
                    playlist={{
                        id: playlistId,
                        name: editName,
                        cover_art: editCoverArt,
                    }}
                />

                <div
                    class="absolute bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center inset-0"
                >
                    <button onclick={pickCover}>
                        <Pen size={28} fill="white" />
                    </button>
                    <Button
                        variant="ghost"
                        size="icon-sm"
                        onclick={removeCover}
                        class="absolute top-1 right-1"
                    >
                        <X />
                    </Button>
                </div>
            </div>
        </div>

        <div class="flex flex-col gap-2">
            <input
                id="playlist-name"
                type="text"
                bind:value={editName}
                placeholder="Playlist name"
                class="w-64 px-3 py-2 rounded-xl text-lg font-semibold text-white placeholder-gray-400 focus:outline-2"
            />
        </div>
    </div>

    {#snippet Footer()}
        <Button onclick={save} disabled={saving || !editName.trim()}>
            {#if saving}
                <LoaderCircle size={14} class="animate-spin" />
            {/if}
            Save
        </Button>
    {/snippet}
</Dialog>
