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
    sortKey="albums"
    sortOptions={[
        { value: "name", label: "Name" },
        { value: "added_at", label: "Date Added" },
        { value: "last_played_at", label: "Recently Played" },
        { value: "total_plays", label: "Most Played" },
        { value: "track_count", label: "Track Count" },
        { value: "year", label: "Year" },
    ]}
/>
