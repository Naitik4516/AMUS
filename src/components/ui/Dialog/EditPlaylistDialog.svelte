<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { invalidateAll } from "$app/navigation";
  import * as Dialog from "$components/ui/dialog/index.js";
  import { Button } from "$components/ui/button/index.js";
  import { Input } from "$components/ui/input/index.js";
  import { Label } from "$components/ui/label/index.js";
  import { selectAndUploadImage } from "$lib/edit-helpers";
  import { store } from "$lib/stores.svelte";
  import { ImagePlus, X, LoaderCircle } from "@lucide/svelte";

  let {
    open = $bindable(false),
    playlistId = 0,
    name = "",
    coverArt = null as string | null,
    onClose = () => {},
  }: {
    open: boolean;
    playlistId: number;
    name: string;
    coverArt?: string | null;
    onClose?: () => void;
  } = $props();

  let editName = $state("");
  let editCoverArt = $state<string | null>(null);
  let coverPreviewUrl = $state<string | null>(null);
  let saving = $state(false);
  let nameDirty = $state(false);
  let coverDirty = $state(false);

  $effect(() => {
    editName = name;
    editCoverArt = coverArt;
    nameDirty = false;
    coverDirty = false;
  });

  $effect(() => {
    coverPreviewUrl = editCoverArt ? store.getImageSrc(editCoverArt, "cover") : null;
  });

  function onNameInput() {
    nameDirty = true;
  }

  async function pickCover() {
    const filename = await selectAndUploadImage("cover");
    if (filename) {
      editCoverArt = filename;
      coverDirty = true;
    }
  }

  function removeCover() {
    editCoverArt = null;
    coverPreviewUrl = null;
    coverDirty = true;
  }

  async function save() {
    saving = true;
    try {
      const args: Record<string, unknown> = { id: playlistId };
      if (nameDirty && editName.trim()) args.name = editName.trim();
      if (coverDirty) args.coverArt = editCoverArt ?? "";
      await invoke("update_playlist", args);
      await invalidateAll();
      onClose();
    } catch (e) {
      console.error("Failed to update playlist", e);
    } finally {
      saving = false;
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>Edit Playlist</Dialog.Title>
    </Dialog.Header>

    <div class="flex flex-col gap-5">
      <div class="flex flex-col gap-2">
        <Label for="playlist-name">Name</Label>
        <Input
          id="playlist-name"
          type="text"
          bind:value={editName}
          oninput={onNameInput}
          placeholder="Playlist name"
        />
      </div>

      <div class="flex flex-col gap-2">
        <Label>Cover Image</Label>
        <div class="flex items-center gap-3">
          {#if coverPreviewUrl}
            <img
              src={coverPreviewUrl}
              alt="Cover preview"
              class="h-16 w-16 shrink-0 rounded-lg object-cover"
            />
          {:else}
            <div class="h-16 w-16 shrink-0 rounded-lg bg-zinc-800 flex items-center justify-center text-zinc-500">
              <ImagePlus size={20} />
            </div>
          {/if}
          <div class="flex gap-2">
            <Button variant="secondary" size="sm" onclick={pickCover}>
              {coverPreviewUrl ? "Replace" : "Choose"}
            </Button>
            {#if coverPreviewUrl}
              <Button variant="ghost" size="sm" onclick={removeCover}>
                <X size={14} />
                Remove
              </Button>
            {/if}
          </div>
        </div>
      </div>
    </div>

    <Dialog.Footer class="flex gap-3">
      <Button variant="secondary" onclick={onClose}>Cancel</Button>
      <Button onclick={save} disabled={saving || !editName.trim()}>
        {#if saving}
          <LoaderCircle size={14} class="animate-spin" />
        {/if}
        Save
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
