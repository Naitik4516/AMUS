<script lang="ts">
    import AlbumCard from "$components/ui/Card/AlbumCard.svelte";
    import DisplayList from "$components/ui/DisplayList.svelte";
    import { store } from "$lib/stores.svelte";

    let albums = $derived(store.albums);
</script>

{#snippet Fallback()}
    <p class="text-gray-500 text-sm mb-4">No albums found.</p>
{/snippet}

<DisplayList
    listItems={albums}
    title="Albums"
    Card={AlbumCard}
    fallBack={Fallback}
/>
