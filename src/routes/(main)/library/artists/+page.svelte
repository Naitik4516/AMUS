<script lang="ts">
    import ArtistCard from "$components/ui/Card/ArtistCard.svelte";
    import DisplayList from "$components/ui/DisplayList.svelte";
    import { store } from "$lib/stores.svelte";
    import { User } from "@lucide/svelte";

    let artists = $derived(store.artists);
</script>

{#snippet Fallback()}
    <User size={64} class="mb-4 opacity-20" />
    <p class="text-xl font-medium">No artists found</p>
    <p class="text-sm">Scan your music library to see artists here.</p>
{/snippet}

<DisplayList
    listItems={artists}
    title="Artists"
    Card={ArtistCard}
    fallBack={Fallback}
/>
