<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import type { Album } from "$lib/types";
    import AlbumRow from "$components/ui/AlbumRow.svelte";

    type loadFunction = "get_top_albums";

    let { loadFunction, title }: { loadFunction: loadFunction; title: string } =
        $props();

    let albums = $state([] as Album[]);

    onMount(() => {
        invoke<Album[]>(loadFunction, {
            limit: 8,
        })
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
