<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { invalidateAll } from "$app/navigation";
  import * as Dialog from "$components/ui/dialog/index.js";
  import { Button } from "$components/ui/button/index.js";
  import { Input } from "$components/ui/input/index.js";
  import { Label } from "$components/ui/label/index.js";
  import { selectAndUploadImage } from "$lib/edit-helpers";
  import { store } from "$lib/stores.svelte";
  import { ImagePlus, X, LoaderCircle, User } from "@lucide/svelte";

  let {
    open = $bindable(false),
    artistId = 0,
    name = "",
    profileImage = null as string | null,
    bannerImage = null as string | null,
    onClose = () => {},
  }: {
    open: boolean;
    artistId: number;
    name: string;
    profileImage?: string | null;
    bannerImage?: string | null;
    onClose?: () => void;
  } = $props();

  let editName = $state<string>("");
  let editProfileImage = $state<string | null>(null);
  let editBannerImage = $state<string | null>(null);
  let profilePreviewUrl = $state<string | null>(null);
  let bannerPreviewUrl = $state<string | null>(null);
  let saving = $state(false);
  let nameDirty = $state(false);
  let profileDirty = $state(false);
  let bannerDirty = $state(false);

  $effect(() => {
    editName = name;
    editProfileImage = profileImage;
    editBannerImage = bannerImage;
    nameDirty = false;
    profileDirty = false;
    bannerDirty = false;
  });

  $effect(() => {
    profilePreviewUrl = editProfileImage ? store.getImageSrc(editProfileImage, "artist") : null;
  });

  $effect(() => {
    bannerPreviewUrl = editBannerImage ? store.getImageSrc(editBannerImage, "banner") : null;
  });

  function onNameInput() {
    nameDirty = true;
  }

  async function pickProfile() {
    const filename = await selectAndUploadImage("artist");
    if (filename) {
      editProfileImage = filename;
      profileDirty = true;
    }
  }

  function removeProfile() {
    editProfileImage = null;
    profilePreviewUrl = null;
    profileDirty = true;
  }

  async function pickBanner() {
    const filename = await selectAndUploadImage("banner");
    if (filename) {
      editBannerImage = filename;
      bannerDirty = true;
    }
  }

  function removeBanner() {
    editBannerImage = null;
    bannerPreviewUrl = null;
    bannerDirty = true;
  }

  async function save() {
    saving = true;
    try {
      const args: Record<string, unknown> = { id: artistId };
      if (nameDirty && editName.trim()) args.name = editName.trim();
      if (profileDirty) args.profileImage = editProfileImage ?? "";
      if (bannerDirty) args.bannerImage = editBannerImage ?? "";
      await invoke("update_artist", args);
      await invalidateAll();
      onClose();
    } catch (e) {
      console.error("Failed to update artist", e);
    } finally {
      saving = false;
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>Edit Artist</Dialog.Title>
    </Dialog.Header>

    <div class="flex flex-col gap-5">
      <div class="flex flex-col gap-2">
        <Label for="artist-name">Name</Label>
        <Input
          id="artist-name"
          type="text"
          bind:value={editName}
          oninput={onNameInput}
          placeholder="Artist name"
        />
      </div>

      <div class="flex flex-col gap-2">
        <Label class="mb-2" >Profile Image</Label>
        <div class="flex items-center justify-between gap-3">
          {#if profilePreviewUrl}
            <img
              src={profilePreviewUrl}
              alt="Profile preview"
              class="h-16 w-16 shrink-0 rounded-full shadow-lg object-cover"
            />
          {:else}
            <div class="h-16 w-16 shrink-0 rounded-full bg-zinc-800 flex items-center justify-center text-zinc-500">
              <User size={20} />
            </div>
          {/if}
          <div class="flex gap-2">
            <Button variant="secondary" size="sm" onclick={pickProfile}>
              {profilePreviewUrl ? "Replace" : "Choose"}
            </Button>
            {#if profilePreviewUrl}
              <Button variant="outline" size="sm" onclick={removeProfile}>
                <X size={14} />
                Remove
              </Button>
            {/if}
          </div>
        </div>
      </div>

      <div class="flex flex-col gap-2">
        <Label>Banner Image</Label>
        <div class="flex items-center gap-3 justify-between">
          {#if bannerPreviewUrl}
            <img
              src={bannerPreviewUrl}
              alt="Banner preview"
              class="h-16 aspect-video shrink-0 rounded-xl shadow-lg object-cover"
            />
          {:else}
            <div class="h-16 w-24 shrink-0 rounded-lg bg-zinc-800 flex items-center justify-center text-zinc-500">
              <ImagePlus size={20} />
            </div>
          {/if}
          <div class="flex gap-2">
            <Button variant="secondary" size="sm" onclick={pickBanner}>
              {bannerPreviewUrl ? "Replace" : "Choose"}
            </Button>
            {#if bannerPreviewUrl}
              <Button variant="outline" size="sm" onclick={removeBanner}>
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
