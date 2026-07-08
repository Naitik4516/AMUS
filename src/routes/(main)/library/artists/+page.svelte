<script lang="ts">
    import ArtistCard from "$components/ui/Card/ArtistCard.svelte";
    import DisplayList from "$components/ui/DisplayList.svelte";
    import { store } from "$lib/stores.svelte";

    let artists = $derived(store.artists);
</script>

<DisplayList listItems={artists} title="Artists" Card={ArtistCard} />
