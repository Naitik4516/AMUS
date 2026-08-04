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

<a
    oncontextmenu={handleContextMenu}
    class="group flex flex-col items-center text-center gap-4"
    href="/library/artists/{data.id}"
>
    <ArtistAvatar
        size={240}
        profileImage={data.profile_image}
        name={data.name}
    />

    <h4
        class="font-extrabold font-satoshi text-lg truncate text-white"
    >
        {data.name}
    </h4>
</a>

<EditArtistDialog
    bind:open={editDialogOpen}
    artistId={data.id}
    name={data.name}
    profileImage={data.profile_image}
    bannerImage={data.banner_image}
/>
