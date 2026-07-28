<script lang="ts">
    import DropdownMenu from "./DropdownMenu.svelte";
    import { player } from "$lib/player.svelte";
    import { store } from "$lib/stores.svelte";
    import { toast } from "svelte-sonner";
    import type { Artist, MenuPosition } from "$lib/types";

    interface Props {
        position: MenuPosition;
        artist: Artist;
        onEdit: () => void;
        onClose: () => void;
    }

    let { position, artist, onEdit, onClose }: Props = $props();

    function buildItems() {
        const tracks = store.tracksByArtist(artist.id);
        const items: any[] = [];

        items.push({
            label: "Play",
            icon: "play",
            onClick: () => {
                if (tracks.length === 0) {
                    toast.error("No tracks by this artist");
                    return;
                }
                player.play(tracks, { type: "Artist", id: artist.id }, 0, artist.name);
            },
        });

        items.push({
            label: "Play next",
            icon: "skip-forward",
            onClick: () => {
                if (tracks.length === 0) {
                    toast.error("No tracks by this artist");
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
                    toast.error("No tracks by this artist");
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
            href: `/library/artists/${artist.id}`,
        });

        return items;
    }
</script>

<DropdownMenu {position} items={buildItems()} {onClose} />
