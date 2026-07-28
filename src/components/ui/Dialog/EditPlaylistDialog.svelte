<script lang="ts">
    import { Button } from "$components/ui/button/index.js";
    import { selectAndUploadImage } from "$lib/edit-helpers";
    import { store } from "$lib/stores.svelte";
    import { X, LoaderCircle, Pen } from "@lucide/svelte";
    import PlaylistCoverArt from "$components/ui/PlaylistCoverArt.svelte";
    import Dialog from "$components/Dialog.svelte";
    import EditImage from "../EditImage.svelte";

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
    let editCoverArt = $state<string | null>();
    let saving = $state(false);

    $effect(() => {
        if (open) {
            editName = name;
            editCoverArt = coverArt;
        }
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
</script>

<Dialog bind:open title="Edit Playlist">
    <div class="flex gap-5 items-center">
        <div class="flex flex-col gap-2">
            <EditImage
                onclick={pickCover}
                {removeCover}
                class=" w-42 h-42 rounded-3xl overflow-clip"
            >
                <PlaylistCoverArt
                    playlist={{
                        id: playlistId,
                        name: editName,
                        cover_art: editCoverArt,
                    }}
                />
            </EditImage>
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
