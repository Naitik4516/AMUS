<script lang="ts">
    import TrackCard from "$components/ui/Card/TrackCard.svelte";
    import type { Track } from "$lib/types";
    import type { InvokeArgs } from "@tauri-apps/api/core";
    import HorizontalScroll from "$components/ui/HorizontalScroll.svelte";
    import {
        getRecentlyPlayed,
        getMostPlayedTracks,
        getFavoriteTracks,
        getForgottenTracks,
        getUnplayedTracks,
        getRecentlyAddedTracks,
    } from "$lib/commands.svelte";
    import { store } from "$lib/stores.svelte";

    type loadFunction =
        | "get_recently_played"
        | "get_most_played_tracks"
        | "get_favorite_tracks"
        | "get_forgotten_tracks"
        | "get_unplayed_tracks"
        | "get_recently_added";

    const LOADERS: Record<
        loadFunction,
        (limit: number, args?: InvokeArgs) => Promise<Track[]>
    > = {
        get_recently_played: getRecentlyPlayed,
        get_most_played_tracks: (limit, args) =>
            getMostPlayedTracks(
                limit,
                (args as { timeframe?: string } | undefined)?.timeframe,
            ),
        get_favorite_tracks: getFavoriteTracks,
        get_forgotten_tracks: getForgottenTracks,
        get_unplayed_tracks: getUnplayedTracks,
        get_recently_added: getRecentlyAddedTracks,
    };

    let {
        title,
        loadFunction,
        args,
        tracks: tracksProp,
        class: className,
    }: {
        title: string;
        loadFunction?: loadFunction;
        args?: InvokeArgs;
        tracks?: Track[];
        class?: string;
    } = $props();

    let tracks = $state([] as Track[]);

    $effect(() => {
        if (tracksProp) {
            tracks = tracksProp;
        } else if (loadFunction) {
            LOADERS[loadFunction](10, args)
                .then((data) => {
                    tracks = data;
                })
                .catch((error) => {
                    console.error("Error loading tracks:", error);
                });
        }
    });
</script>

{#if tracks.length > 0}
    <HorizontalScroll
        {title}
        data={tracks}
        Card={TrackCard}
        class={className}
    />
{/if}
