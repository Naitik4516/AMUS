<script lang="ts">
    import { Button } from "$components/ui/button/index.js";
    import { selectAndUploadImage } from "$lib/edit-helpers";
    import { store } from "$lib/stores.svelte";
    import { LoaderCircle, ImagePlus } from "@lucide/svelte";
    import Dialog from "$components/Dialog.svelte";
    import { onMount } from "svelte";
    import EditImage from "../EditImage.svelte";

    let {
        open = $bindable(false),
        genreId = 0,
        name = "",
        thumbnail = null as string | null,
    }: {
        open: boolean;
        genreId: number;
        name: string;
        thumbnail?: string | null;
    } = $props();

    let editName = $state("");
    let editThumbnail = $state<string | null>();
    let thumbnailChanged = $state(false);
    let saving = $state(false);

    onMount(() => {
        if (open) {
            editName = name;
            editThumbnail = thumbnail;
            thumbnailChanged = false;
        }
    });

    async function pickThumbnail() {
        const filename = await selectAndUploadImage("cover");
        if (filename) {
            editThumbnail = filename;
            thumbnailChanged = true;
        }
    }

    function removeThumbnail() {
        editThumbnail = null;
        thumbnailChanged = true;
    }

    async function save() {
        saving = true;
        try {
            await store.saveGenre(genreId, editName.trim(), thumbnailChanged ? editThumbnail : undefined);
            open = false;
        } catch (e) {
            console.error("Failed to update genre", e);
        } finally {
            saving = false;
        }
    }
</script>

<Dialog bind:open title="Edit Genre">
    <div class="flex gap-5 items-center">
        <div class="flex flex-col gap-2">
            <EditImage
                onclick={pickThumbnail}
                removeCover={removeThumbnail}
                class="w-42 h-42 rounded-3xl overflow-clip"
            >
                {#if editThumbnail || thumbnail}
                    <img
                        src={store.getImageSrc(editThumbnail ?? thumbnail, "cover")}
                        alt="Thumbnail preview"
                        class="h-full w-full object-cover"
                    />
                {:else}
                    <div class="h-full w-full shrink-0 bg-zinc-800 flex items-center justify-center text-zinc-500">
                        <ImagePlus size={20} />
                    </div>
                {/if}
            </EditImage>
        </div>

        <div class="flex flex-col gap-2">
            <input
                id="genre-name"
                type="text"
                bind:value={editName}
                placeholder="Genre Name"
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
