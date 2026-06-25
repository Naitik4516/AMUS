<!-- <script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import type { Track } from "$lib/types";
    import TrackList from "$components/ui/TrackList.svelte";
    import AlbumHeader from "$components/ui/Headers/AlbumHeader.svelte";
    import { Heart } from "@lucide/svelte";

    let tracks = $state<Track[]>([]);
    let sort = $state("recently_added");

    async function loadFavourites() {
        try {
            tracks = await invoke("get_favorite_tracks", { sortBy: sort });
        } catch (e) {
            console.error("Failed to load favourites", e);
        }
    }

    $effect(() => {
        if (sort) loadFavourites();
    });

    onMount(loadFavourites);
</script>

{#snippet Header()}
    <AlbumHeader name="Favourites" coverArt={null} {tracks} />
{/snippet}

<TrackList {tracks} name="Favourites" {Header} />
 -->
