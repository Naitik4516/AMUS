<script lang="ts">
    import DropdownMenu from "./DropdownMenu.svelte";
    import { player } from "$lib/player.svelte";
    import { store } from "$lib/stores.svelte";
    import { toast } from "svelte-sonner";
    import type { Playlist, MenuPosition } from "$lib/types";

    interface Props {
        position: MenuPosition;
        playlist: Playlist;
        onEdit: () => void;
        onDelete: () => void;
        onClose: () => void;
    }

    let { position, playlist, onEdit, onDelete, onClose }: Props = $props();

    function buildItems() {
        const tracks = store.tracksByPlaylist(playlist.id);
        const items: any[] = [];

        items.push({
            label: "Play",
            icon: "play",
            onClick: () => {
                if (tracks.length === 0) {
                    toast.error("No tracks in this playlist");
                    return;
                }
                player.play(tracks, { type: "Playlist", id: playlist.id }, 0, playlist.name);
            },
        });

        items.push({
            label: "Play next",
            icon: "skip-forward",
            onClick: () => {
                if (tracks.length === 0) {
                    toast.error("No tracks in this playlist");
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
                    toast.error("No tracks in this playlist");
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
            label: "Delete",
            icon: "trash",
            danger: true,
            onClick: () => onDelete(),
        });

        return items;
    }
</script>

<DropdownMenu {position} items={buildItems()} {onClose} />
