<script lang="ts">
  import { Button } from "$components/ui/button/index.js";
  import { selectAndUploadImage } from "$lib/edit-helpers";
  import { store } from "$lib/stores.svelte";
  import { ImagePlus, X, LoaderCircle, User } from "@lucide/svelte";
  import Dialog from "$components/Dialog.svelte";

  let {
    open = $bindable(false),
    artistId = 0,
    name = "",
    profileImage = null as string | null,
    bannerImage = null as string | null,
  }: {
    open: boolean;
    artistId: number;
    name: string;
    profileImage?: string | null;
    bannerImage?: string | null;
  } = $props();

  let editName = $state<string>("");
  let editProfileImage = $state<string | null>(null);
  let editBannerImage = $state<string | null>(null);
  let saving = $state(false);

  $effect(() => {
    editName = name;
    editProfileImage = profileImage;
    editBannerImage = bannerImage;
  });

  async function pickProfile() {
    const filename = await selectAndUploadImage("artist");
    if (filename) {
      editProfileImage = filename;
    }
  }

  function removeProfile() {
    editProfileImage = null;
  }

  async function pickBanner() {
    const filename = await selectAndUploadImage("artist");
    if (filename) {
      editBannerImage = filename;
    }
  }

  function removeBanner() {
    editBannerImage = null;
  }

  async function save() {
    saving = true;
    try {
      await store.saveArtist(artistId, editName.trim(), editProfileImage, editBannerImage);
      open = false;
    } catch (e) {
      console.error("Failed to update artist", e);
    } finally {
      saving = false;
    }
  }
</script>

<Dialog bind:open title="Edit Artist">
  <div class="flex flex-col gap-5">
    <div class="flex flex-col gap-2">
      <label for="artist-name" class="text-sm font-medium text-zinc-300">Name</label>
      <input
        id="artist-name"
        type="text"
        bind:value={editName}
        placeholder="Artist name"
        class="w-full px-3 py-2 rounded-xl text-lg font-semibold text-white placeholder-gray-400 focus:outline-2"
      />
    </div>

    <div class="flex flex-col gap-2">
      <label class="text-sm font-medium text-zinc-300">Profile Image</label>
      <div class="flex items-center justify-between gap-3">
        {#if editProfileImage || profileImage}
          <img
            src={store.getImageSrc(editProfileImage ?? profileImage, "artist")}
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
            {editProfileImage || profileImage ? "Replace" : "Choose"}
          </Button>
          {#if editProfileImage || profileImage}
            <Button variant="outline" size="sm" onclick={removeProfile}>
              <X size={14} />
              Remove
            </Button>
          {/if}
        </div>
      </div>
    </div>

    <div class="flex flex-col gap-2">
      <label class="text-sm font-medium text-zinc-300">Banner Image</label>
      <div class="flex items-center gap-3 justify-between">
        {#if editBannerImage || bannerImage}
          <img
            src={store.getImageSrc(editBannerImage ?? bannerImage, "artist")}
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
            {editBannerImage || bannerImage ? "Replace" : "Choose"}
          </Button>
          {#if editBannerImage || bannerImage}
            <Button variant="outline" size="sm" onclick={removeBanner}>
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
