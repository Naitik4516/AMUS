<script lang="ts">
    import DropdownMenu from "./DropdownMenu.svelte";
    import { player } from "$lib/player.svelte";
    import { toast } from "svelte-sonner";
    import type { Track, MenuPosition } from "$lib/types";
    import PlaylistMenu from "./PlaylistMenu.svelte";
    import ArtistsMenu from "./ArtistsMenu.svelte";
    import type { Context } from "$lib/types";
    import { store } from "$lib/stores.svelte";

    const ALL_OPTIONS = [
        "addToQueue",
        "playNext",
        "addToPlaylist",
        "viewDetails",
        "goToArtist",
        "goToAlbum",
        "removeFromPlaylist",
    ] as const;

    type MenuOption = (typeof ALL_OPTIONS)[number];

    interface Props {
        position: MenuPosition;
        track: Track;
        context: Context;
        exclude?: MenuOption[];
        onClose: () => void;
    }

    let { position, track, context, exclude = [], onClose }: Props = $props();

    function isExcluded(option: MenuOption): boolean {
        return exclude.includes(option);
    }

    function buildItems() {
        const items: any[] = [];

        if (!isExcluded("removeFromPlaylist") && context.type === "Playlist") {
            items.push({
                label: "Remove from this playlist",
                icon: "circle-minus",
                danger: true,
                onClick: async () => {
                    await store.removeTrackFromPlaylist(track.id, context.id);
                    toast.success("Removed from playlist");
                },
            });
        }

        if (!isExcluded("addToQueue")) {
            items.push({
                label: "Add to queue",
                icon: "list-plus",
                onClick: () => {
                    player.enqueueEnd(track);
                    toast.success("Added to queue");
                },
            });
        }

        if (!isExcluded("playNext")) {
            items.push({
                label: "Play next",
                icon: "skip-forward",
                onClick: () => {
                    player.enqueueNext(track);
                    toast.success("Will play next");
                },
            });
        }
        // Add to playlist submenu
        if (!isExcluded("addToPlaylist")) {
            items.push({
                label: "Add to playlist",
                icon: "plus",
                submenu: PlaylistMenu,
                track: track,
                context,
            });
        }

        items.push({ type: "separator" });

        if (!isExcluded("viewDetails")) {
            items.push({
                label: "View song details",
                icon: "info",
                href: `/library/track/${track.id}`,
            });
        }

        // Artist links
        if (!isExcluded("goToArtist") && track.artists?.length) {
            if (track.artists.length === 1) {
                items.push({
                    label: "Go to artist",
                    icon: "user",
                    href: `/library/artists/${track.artists[0].id}`,
                });
            } else {
                items.push({
                    label: "Go to artist",
                    icon: "user",
                    submenu: ArtistsMenu,
                    track: track,
                });
            }
        }

        // Album link
        if (
            !isExcluded("goToAlbum") &&
            context.type !== "Album" &&
            track.album
        ) {
            items.push({
                label: "Go to album",
                icon: "disc",
                href: `/library/albums/${track.album.id}`,
            });
        }

        return items;
    }
</script>

<DropdownMenu {position} items={buildItems()} {onClose} />
