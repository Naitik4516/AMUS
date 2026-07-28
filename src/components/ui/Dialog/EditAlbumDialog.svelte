<script lang="ts">
    import { Button } from "$components/ui/button/index.js";
    import { selectAndUploadImage } from "$lib/edit-helpers";
    import { store } from "$lib/stores.svelte";
    import { Pen, X, LoaderCircle, ImagePlus } from "@lucide/svelte";
    import Dialog from "$components/Dialog.svelte";
    import { onMount } from "svelte";

    let {
        open = $bindable(false),
        albumId = 0,
        name = "",
        coverArt = null as string | null,
    }: {
        open: boolean;
        albumId: number;
        name: string;
        coverArt?: string | null;
    } = $props();

    let editName = $state("");
    let editCoverArt = $state<string | null>();
    let coverChanged = $state(false);
    let saving = $state(false);

    onMount(() => {
        if (open) {
            editName = name;
            editCoverArt = coverArt;
            coverChanged = false;
        }
    });

    async function pickCover() {
        const filename = await selectAndUploadImage("cover");
        if (filename) {
            editCoverArt = filename;
            coverChanged = true;
        }
    }

    function removeCover() {
        editCoverArt = null;
        coverChanged = true;
    }

    async function save() {
        saving = true;
        try {
            if (coverChanged) {
                await store.saveAlbum(
                    albumId,
                    editName.trim(),
                    editCoverArt ?? "",
                );
            } else {
                await store.saveAlbum(albumId, editName.trim());
            }
            open = false;
        } catch (e) {
            console.error("Failed to update album", e);
        } finally {
            saving = false;
        }
    }
</script>

<Dialog bind:open title="Edit Playlist">
    <div class="flex gap-5 items-center">
        <div class="flex flex-col gap-2">
            <div class="relative w-42 h-42 rounded-3xl overflow-clip group">
                {#if editCoverArt || coverArt}
                    <img
                        src={store.getImageSrc(
                            editCoverArt ?? coverArt,
                            "cover",
                        )}
                        alt="Cover preview"
                        class="h-full w-full"
                    />
                {:else}
                    <div
                        class="h-full w-full shrink-0 bg-zinc-800 flex items-center justify-center text-zinc-500"
                    >
                        <ImagePlus size={20} />
                    </div>
                {/if}

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
                id="album-name"
                type="text"
                bind:value={editName}
                placeholder="Album Name"
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
