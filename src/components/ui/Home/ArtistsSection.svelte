<script lang="ts">
    import ArtistCard from "$components/ui/Card/ArtistCard.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import type { Artist } from "$lib/types";
    import HorizontalScroll from "$components/ui/HorizontalScroll.svelte";

    type loadFunction = "get_top_artists";

    let { loadFunction, title }: { loadFunction: loadFunction; title: string } =
        $props();

    let artists = $state([] as Artist[]);

    onMount(() => {
        invoke<Artist[]>(loadFunction, {
            limit: 6,
        })
            .then((data) => {
                artists = data;
            })
            .catch((error) => {
                console.error("Error loading artists:", error);
            });
    });
</script>

{#if artists.length > 0}
    <HorizontalScroll {title} data={artists} Card={ArtistCard} />
{/if}
