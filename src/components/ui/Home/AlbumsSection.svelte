<script lang="ts">
    import AlbumRow from "$components/ui/AlbumRow.svelte";
    import { getTopAlbums } from "$lib/commands.svelte";
    import type { Album } from "$lib/types";

    let { title }: { title: string } = $props();

    let albums = $state([] as Album[]);

    $effect(() => {
        getTopAlbums(8)
            .then((data) => {
                albums = data;
            })
            .catch((error) => {
                console.error("Error loading albums:", error);
            });
    });
</script>

{#if albums.length > 0}
    <AlbumRow {title} {albums} />
{/if}
