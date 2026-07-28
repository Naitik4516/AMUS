<script lang="ts">
    import { Button } from "$components/ui/button/index.js";
    import { selectAndUploadImage } from "$lib/edit-helpers";
    import { store } from "$lib/stores.svelte";
    import { ImagePlus, X, LoaderCircle, User } from "@lucide/svelte";
    import Dialog from "$components/Dialog.svelte";
    import { onMount } from "svelte";
    import Input from "../input/input.svelte";
    import EditImage from "../EditImage.svelte";

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

    onMount(() => {
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
            await store.saveArtist(
                artistId,
                editName.trim(),
                editProfileImage,
                editBannerImage,
            );
            open = false;
        } catch (e) {
            console.error("Failed to update artist", e);
        } finally {
            saving = false;
        }
    }
</script>

<Dialog bind:open title="Edit Artist">
    <div class="flex flex-col gap-5 mb-5 font-satoshi">
        <div class="flex flex-col gap-2">
            <Input
                id="artist-name"
                type="text"
                bind:value={editName}
                placeholder="Artist name"
            />
        </div>

        <div class="flex justify-around items-center mx-5">
            <div class="flex flex-col gap-2 justify-between h-46">
                <EditImage
                    onclick={pickProfile}
                    removeCover={removeProfile}
                    class="h-30 w-30 shrink-0 rounded-full shadow-lg overflow-hidden mt-5"
                >
                    {#if editProfileImage || profileImage}
                        <img
                            src={store.getImageSrc(
                                editProfileImage ?? profileImage,
                                "artist",
                            )}
                            alt="Profile preview"
                            id="profile-image"
                            class="object-cover"
                        />
                    {:else}
                        <div
                            class="bg-zinc-800 flex items-center justify-center text-zinc-500"
                        >
                            <User size={20} />
                        </div>
                    {/if}
                </EditImage>
                <label
                    class="text-sm text-center font-bold text-zinc-300"
                    for="profile-image">Profile Image</label
                >
            </div>

            <div class="flex flex-col gap-2 h-46 justify-between items-center">
                <EditImage
                    onclick={pickBanner}
                    removeCover={removeBanner}
                    class="h-38 max-w-2/3 ml-auto  shrink-0 rounded-xl shadow-lg overflow-hidden "
                >
                    {#if editBannerImage || bannerImage}
                        <img
                            src={store.getImageSrc(
                                editBannerImage ?? bannerImage,
                                "artist",
                            )}
                            alt="Banner preview"
                            id="banner-image"
                            class=""
                        />
                    {:else}
                        <div
                            class="w-24 bg-zinc-800 flex items-center justify-center text-zinc-500"
                        >
                            <ImagePlus size={20} />
                        </div>
                    {/if}
                </EditImage>
                <label
                    class="text-sm text-center font-bold text-zinc-300"
                    for="banner-image">Banner Image</label
                >
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
