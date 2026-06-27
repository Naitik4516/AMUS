<script lang="ts">
    import { page } from "$app/state";
    import type { Track } from "$lib/types";
    import type { PageProps } from "./$types";
    import TrackList from "$components/ui/TrackList.svelte";

    let { data }: PageProps = $props();
    let tracks = $derived((data.data as Track[]) || []);
    let playlistId = $derived(Number(page.params.id));
    let playlistName = $derived(data.name ?? "");
    let playlistCoverArt = $derived(data.coverArtFilename ?? null);
</script>


<TrackList
    {tracks}
    context="playlist"
    {playlistId}
    {playlistName}
    {playlistCoverArt}
    canEdit={true}
/>
