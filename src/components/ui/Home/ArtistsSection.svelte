<script lang="ts">
    import ArtistCard from "$components/ui/Card/ArtistCard.svelte";
    import { getTopArtists } from "$lib/commands.svelte";
    import type { Artist } from "$lib/types";
    import HorizontalScroll from "$components/ui/HorizontalScroll.svelte";
    import { store } from "$lib/stores.svelte";

    let { title }: { title: string } = $props();

    let artists = $state([] as Artist[]);

    $effect(() => {
        store.tracks;
        getTopArtists(6)
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
