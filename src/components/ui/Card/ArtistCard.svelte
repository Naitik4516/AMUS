<script lang="ts">
    import { User } from "@lucide/svelte";
    import { store } from "$lib/stores.svelte";
    import type { Artist } from "$lib/types";
    import Icon from "../Icon.svelte";
    import ArtistMenu from "$components/ui/Menu/ArtistMenu.svelte";
    import EditArtistDialog from "$components/ui/Dialog/EditArtistDialog.svelte";
    import { openContextMenu } from "$lib/context-menu.svelte";

    let { data }: { data: Artist } = $props();

    let editDialogOpen = $state(false);

    function handleContextMenu(e: MouseEvent) {
        e.preventDefault();
        openContextMenu(ArtistMenu, {
            position: { type: "coordinates", x: e.clientX, y: e.clientY },
            artist: data,
            onEdit: () => {
                editDialogOpen = true;
            },
        });
    }
</script>

<a
    href="/library/artists/{data.id}"
    oncontextmenu={handleContextMenu}
    class="group flex flex-col items-center text-center gap-4 px-5 py-3"
>
    <div
        class="w-60 h-60 rounded-full overflow-hidden bg-gray-800 border border-black/30 shadow-xl relative"
    >
        {#if data.profile_image}
            <img
                src={store.getImageSrc(data.profile_image, "artist")}
                alt={data.name}
                class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
            />
        {:else}
            <div
                class="w-full h-full flex items-center justify-center alighn-center"
            >
                <Icon name="artist" size={80} fill="white" />
            </div>
        {/if}

        <div
            class="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center"
        >
            <div
                class="bg-gray-200/10 backdrop-blur-md border text-gray-800 p-3 rounded-full shadow-lg transform translate-y-4 group-hover:translate-y-0 transition-transform"
            >
                <Icon name="artist" size={30} fill="black" />
            </div>
        </div>
    </div>

    <h3 class="font-extrabold font-satoshi text-lg truncate text-white">{data.name}</h3>
</a>

<EditArtistDialog
    bind:open={editDialogOpen}
    artistId={data.id}
    name={data.name}
    profileImage={data.profile_image}
    bannerImage={data.banner_image}
/>
