<script lang="ts">
    import TrackCard from "$components/ui/Card/TrackCard.svelte";
    import { invoke, type InvokeArgs } from "@tauri-apps/api/core";
    import type { Track } from "$lib/types";
    import HorizontalScroll from "$components/ui/HorizontalScroll.svelte";

    type loadFunction =
        | "get_recently_played"
        | "get_most_played_tracks"
        | "get_favorite_tracks"
        | "get_forgotten_tracks"
        | "get_unplayed_tracks"
        | "get_recently_added";

    let {
        title,
        loadFunction,
        args,
        tracks: tracksProp,
    }: {
        title: string;
        loadFunction?: loadFunction;
        args?: InvokeArgs;
        tracks?: Track[];
    } = $props();

    let tracks = $state([] as Track[]);

    $effect(() => {
        if (tracksProp) {
            tracks = tracksProp;
        } else if (loadFunction) {
            invoke<Track[]>(loadFunction, { limit: 10, ...args } as InvokeArgs)
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
    <HorizontalScroll {title} data={tracks} Card={TrackCard} />
{/if}
