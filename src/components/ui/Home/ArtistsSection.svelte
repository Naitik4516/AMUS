<script lang="ts">
    import ArtistCard from "$components/ui/Card/ArtistCard.svelte";
    import HorizontalScroll from "$components/ui/HorizontalScroll.svelte";
    import { getTopArtists } from "$lib/commands.svelte";
    import type { Artist } from "$lib/types";

    let { title }: { title: string } = $props();

    let artists = $state([] as Artist[]);

    $effect(() => {
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
