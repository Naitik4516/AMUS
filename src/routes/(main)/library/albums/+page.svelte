<script lang="ts">
    import AlbumCard from "$components/ui/Card/AlbumCard.svelte";
    import DisplayList from "$components/ui/DisplayList.svelte";
    import { store } from "$lib/stores.svelte";

    let albums = $derived(store.albums);
</script>

<DisplayList listItems={albums} title="Albums" Card={AlbumCard} />
