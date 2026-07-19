<script lang="ts">
    import { Button } from "$components/ui/button/index.js";
    import { selectAndUploadImage } from "$lib/edit-helpers";
    import { store } from "$lib/stores.svelte";
    import { ImagePlus, X, LoaderCircle } from "@lucide/svelte";
    import Dialog from "$components/Dialog.svelte";

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
    let editCoverArt = $state<string | null>(null);
    let coverChanged = $state(false);
    let saving = $state(false);

    $effect(() => {
        editName = name;
        editCoverArt = coverArt;
        coverChanged = false;
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
                await store.saveAlbum(albumId, editName.trim(), editCoverArt ?? "");
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

<Dialog bind:open title="Edit Album">
    <div class="flex flex-col gap-5">
        <div class="flex flex-col gap-2">
            <label for="album-name" class="text-sm font-medium text-zinc-300">Name</label>
            <input
                id="album-name"
                type="text"
                bind:value={editName}
                placeholder="Album name"
                class="w-full px-3 py-2 rounded-xl text-lg font-semibold text-white placeholder-gray-400 focus:outline-2"
            />
        </div>

        <div class="flex flex-col gap-2">
            <label class="text-sm font-medium text-zinc-300">Cover Art</label>
            <div class="flex items-center gap-3">
                {#if editCoverArt || coverArt}
                    <img
                        src={store.getImageSrc(editCoverArt ?? coverArt, "cover")}
                        alt="Cover preview"
                        class="h-16 w-16 shrink-0 rounded-lg object-cover"
                    />
                {:else}
                    <div
                        class="h-16 w-16 shrink-0 rounded-lg bg-zinc-800 flex items-center justify-center text-zinc-500"
                    >
                        <ImagePlus size={20} />
                    </div>
                {/if}
                <div class="flex gap-2">
                    <Button variant="secondary" size="sm" onclick={pickCover}>
                        {editCoverArt || coverArt ? "Replace" : "Choose"}
                    </Button>
                    {#if editCoverArt || coverArt}
                        <Button variant="ghost" size="sm" onclick={removeCover}>
                            <X size={14} />
                            Remove
                        </Button>
                    {/if}
                </div>
            </div>
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
