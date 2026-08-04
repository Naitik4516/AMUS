<script lang="ts">
    import { User } from "@lucide/svelte";
    import { store } from "$lib/stores.svelte";
    import type { Artist } from "$lib/types";
    import Icon from "../Icon.svelte";
    import CollectionMenu from "$components/ui/Menu/CollectionMenu.svelte";
    import EditArtistDialog from "$components/ui/Dialog/EditArtistDialog.svelte";
    import { openContextMenu } from "$lib/context-menu.svelte";
    import ArtistAvatar from "../ArtistAvatar.svelte";

    let { data }: { data: Artist } = $props();

    let editDialogOpen = $state(false);

    function handleContextMenu(e: MouseEvent) {
        e.preventDefault();
        openContextMenu(CollectionMenu, {
            position: { type: "coordinates", x: e.clientX, y: e.clientY },
            type: "artist",
            id: data.id,
            name: data.name,
            detailsHref: `/library/artists/${data.id}`,
            onEdit: () => {
                editDialogOpen = true;
            },
        });
    }
</script>

<div
    oncontextmenu={handleContextMenu}
    class="group flex flex-col items-center text-center gap-4 px-5 py-3"
    role="feed"
>
    <ArtistAvatar
        size={240}
        profileImage={data.profile_image}
        name={data.name}
    />

    <a
        href="/library/artists/{data.id}"
        class="font-extrabold font-satoshi text-lg truncate text-white"
    >
        {data.name}
    </a>
</div>

<EditArtistDialog
    bind:open={editDialogOpen}
    artistId={data.id}
    name={data.name}
    profileImage={data.profile_image}
    bannerImage={data.banner_image}
/>
