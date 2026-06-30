<script lang="ts">
    import type { PageProps } from "./$types";
    import AlbumCard from "$components/ui/Card/AlbumCard.svelte";
    import DisplayList from "$components/ui/DisplayList.svelte";

    let { data }: PageProps = $props();
    let albums = $derived(data.albums || []);
</script>

<DisplayList listItems={albums} title="Albums" Card={AlbumCard} />
