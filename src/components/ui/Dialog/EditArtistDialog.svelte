<script lang="ts">
    import { Button } from "$components/ui/button/index.js";
    import { selectAndUploadImage } from "$lib/edit-helpers";
    import { store } from "$lib/stores.svelte";
    import { ImagePlus, LoaderCircle, User } from "@lucide/svelte";
    import Dialog from "$components/Dialog.svelte";
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

    $effect(() => {
        if (open) {
            editName = name;
            editProfileImage = profileImage;
            editBannerImage = bannerImage;
        }
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

<Dialog bind:open title="Edit Artist" maxWidth="xl">
    <div class="flex flex-col gap-5 mb-5 font-satoshi">
        <div class="flex flex-col gap-2">
            <Input
                id="artist-name"
                type="text"
                bind:value={editName}
                placeholder="Artist name"
            />
        </div>

        <div class="flex justify-around mx-5 h-70">
            <div class="flex flex-col gap-2 h-full ">
                <EditImage
                    onclick={pickProfile}
                    removeCover={removeProfile}
                    class="h-40 w-40 shrink-0 rounded-full shadow-lg overflow-hidden border my-auto"
                    closeButtonClass="top-2 right-2"
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
                            class="bg-zinc-800 flex items-center justify-center text-zinc-300 h-full"
                        >
                            <User size={32} strokeWidth={3} />
                        </div>
                    {/if}
                </EditImage>
                <label
                    class="text-sm text-center font-bold text-zinc-300 "
                    for="profile-image">Profile Image</label
                >
            </div>

            <div
                class="flex flex-col gap-2 h-full max-w-1/2 justify-between items-center"
            >
                <EditImage
                    onclick={pickBanner}
                    removeCover={removeBanner}
                    class="h-70 rounded-2xl shadow-lg overflow-hidden bg-neutral-800/50"
                >
                    {#if editBannerImage || bannerImage}
                        <img
                            src={store.getImageSrc(
                                editBannerImage ?? bannerImage,
                                "artist",
                            )}
                            alt="Banner preview"
                            id="banner-image"
                            class="object-cover h-full "
                        />
                    {:else}
                        <div
                            class="w-50 h-full bg-zinc-800 flex items-center justify-center text-zinc-300"
                        >
                            <ImagePlus size={40} strokeWidth={2.5} />
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
