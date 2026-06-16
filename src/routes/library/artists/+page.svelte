<script lang="ts">
    import ArtistCard from "$components/ui/ArtistCard.svelte";
    import DisplayList from "$components/ui/DisplayList.svelte";
    import type { PageProps } from "./$types";

    let { data }: PageProps = $props();
    let artists = $derived(data.artists || []);
</script>

<DisplayList listItems={artists} title="Artists" Card={ArtistCard} />
