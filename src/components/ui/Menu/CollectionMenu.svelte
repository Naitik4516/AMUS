<script lang="ts">
    import DropdownMenu, { type MenuItem } from "./DropdownMenu.svelte";
    import { player } from "$lib/player.svelte";
    import { store } from "$lib/stores.svelte";
    import { toast } from "svelte-sonner";
    import type { MenuPosition, PlaybackSource, Track } from "$lib/types";

    type CollectionType = "playlist" | "artist" | "album";

    interface Props {
        position: MenuPosition;
        type: CollectionType;
        id: number;
        name: string;
        onEdit: () => void;
        onDelete?: () => void;
        detailsHref?: string;
        onClose: () => void;
    }

    let { position, type, id, name, onEdit, onDelete, detailsHref, onClose }: Props =
        $props();

    function getSource(): PlaybackSource {
        if (type === "playlist") return { type: "Playlist", id };
        if (type === "artist") return { type: "Artist", id };
        return { type: "Album", id };
    }

    const EMPTY_MESSAGES: Record<CollectionType, string> = {
        playlist: "No tracks in this playlist",
        artist: "No tracks by this artist",
        album: "No tracks in this album",
    };

    function getTracks(): Track[] {
        if (type === "playlist") return store.tracksByPlaylist(id);
        if (type === "artist") return store.tracksByArtist(id);
        return store.tracksByAlbum(id);
    }

    function buildItems(): MenuItem[] {
        const tracks = getTracks();
        const emptyMessage = EMPTY_MESSAGES[type];
        const items: MenuItem[] = [];

        items.push({
            label: "Play",
            icon: "play",
            onClick: () => {
                if (tracks.length === 0) {
                    toast.error(emptyMessage);
                    return;
                }
                player.play(tracks, getSource(), 0, name);
            },
        });

        items.push({
            label: "Play next",
            icon: "skip-forward",
            onClick: () => {
                if (tracks.length === 0) {
                    toast.error(emptyMessage);
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
                    toast.error(emptyMessage);
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

        if (detailsHref) {
            items.push({
                label: "View details",
                icon: "info",
                href: detailsHref,
            });
        }

        if (onDelete) {
            items.push({
                label: "Delete",
                icon: "trash",
                danger: true,
                onClick: () => onDelete(),
            });
        }

        return items;
    }

    let items = $derived(buildItems());
</script>

<DropdownMenu {position} items={items} {onClose} />
