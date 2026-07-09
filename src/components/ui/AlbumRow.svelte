<script lang="ts">
    import type { Album } from "$lib/types";
    import HorizontalScroll from "$components/ui/HorizontalScroll.svelte";
    import AlbumCard from "./Card/AlbumCard.svelte";

    let { title, albums }: { title: string; albums: Album[] } = $props();
</script>

<HorizontalScroll {title} data={albums} Card={AlbumCard} />
