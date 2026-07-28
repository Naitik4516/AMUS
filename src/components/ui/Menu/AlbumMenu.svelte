<script lang="ts">
    import DropdownMenu from "./DropdownMenu.svelte";
    import { player } from "$lib/player.svelte";
    import { store } from "$lib/stores.svelte";
    import { toast } from "svelte-sonner";
    import type { Album, MenuPosition } from "$lib/types";

    interface Props {
        position: MenuPosition;
        album: Album;
        onEdit: () => void;
        onClose: () => void;
    }

    let { position, album, onEdit, onClose }: Props = $props();

    function buildItems() {
        const tracks = store.tracksByAlbum(album.id);
        const items: any[] = [];

        items.push({
            label: "Play",
            icon: "play",
            onClick: () => {
                if (tracks.length === 0) {
                    toast.error("No tracks in this album");
                    return;
                }
                player.play(tracks, { type: "Album", id: album.id }, 0, album.name);
            },
        });

        items.push({
            label: "Play next",
            icon: "skip-forward",
            onClick: () => {
                if (tracks.length === 0) {
                    toast.error("No tracks in this album");
                    return;
                }
                for (let i = tracks.length - 1; i >= 0; i--) {
                    player.enqueueNext(tracks[i]);
                }
                toast.success(`${tracks.length} tracks queued next`);
            },
        });

        items.push({
            label: "Add to queue",
            icon: "list-plus",
            onClick: () => {
                if (tracks.length === 0) {
                    toast.error("No tracks in this album");
                    return;
                }
                player.enqueueEndMany(tracks);
                toast.success(`${tracks.length} tracks added to queue`);
            },
        });

        items.push({ type: "separator" });

        items.push({
            label: "Edit",
            icon: "edit",
            onClick: () => onEdit(),
        });

        items.push({
            label: "View details",
            icon: "info",
            href: `/library/albums/${album.id}`,
        });

        return items;
    }
</script>

<DropdownMenu {position} items={buildItems()} {onClose} />
