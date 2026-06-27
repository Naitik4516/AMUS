<script lang="ts">
    import DropdownMenu from "./DropdownMenu.svelte";
    import { player } from "$lib/player.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { toast } from "svelte-sonner";
    import type { Track } from "$lib/types";
    import PlaylistMenu from "./PlaylistMenu.svelte";
    import ArtistsMenu from "./ArtistsMenu.svelte";

    type Context = "playlist" | "album" | "artist" | "liked";

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
        track: Track;
        context: Context;
        exclude?: MenuOption[];
        onClose: () => void;
        playlistId?: number | null;
    }

    let {
        track,
        context,
        exclude = [],
        onClose,
        playlistId = null,
    }: Props = $props();

    function isExcluded(option: MenuOption): boolean {
        return exclude.includes(option);
    }

    function buildItems() {
        const items: any[] = [];

        if (!isExcluded("addToQueue")) {
            items.push({
                label: "Add to queue",
                icon: "list-plus",
                onClick: () => {
                    player.addToQueue(track);
                    toast.success("Added to queue");
                },
            });
        }

        if (!isExcluded("playNext")) {
            items.push({
                label: "Play next",
                icon: "skip-forward",
                onClick: () => {
                    player.playNextInQueue(track);
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
            });
        }

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
        if (!isExcluded("goToAlbum") && context !== "album" && track.album) {
            items.push({
                label: "Go to album",
                icon: "disc",
                href: `/library/albums/${track.album.id}`,
            });
        }

        // Remove from playlist (only in playlist context)
        if (
            !isExcluded("removeFromPlaylist") &&
            context === "playlist" &&
            playlistId
        ) {
            items.push({ type: "separator" });
            items.push({
                label: "Remove from this playlist",
                icon: "x",
                danger: true,
                onClick: async () => {
                    try {
                        await invoke("remove_track_from_playlist", {
                            trackId: track.id,
                            playlistId,
                        });
                        toast.success("Removed from playlist");
                    } catch (e) {
                        toast.error("Failed to remove from playlist");
                    }
                },
            });
        }

        return items;
    }
</script>

<DropdownMenu items={buildItems()} placement="bottom-left" {onClose} />
