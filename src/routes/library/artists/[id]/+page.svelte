<script lang="ts">
    import type { PageProps } from "./$types";
    import TrackList from "$components/ui/TrackList.svelte";
    import AlbumRow from "$components/ui/AlbumRow.svelte";
    import { getArtistPicUrl } from "$lib/utils";
    import { User } from "@lucide/svelte";

    let { data }: PageProps = $props();
    let artist = $derived(data.artist);
    let tracks = $derived(data.tracks || []);
    let albums = $derived(data.albums || []);

    let subtitle = $derived(
        `${tracks.length} ${tracks.length === 1 ? "track" : "tracks"}, ${albums.length} ${albums.length === 1 ? "album" : "albums"}`
    );
</script>

{#snippet artistCover()}
    {#if artist.profile_picture}
        {#await getArtistPicUrl(artist.profile_picture)}
            <div class="w-full h-full bg-neutral-800 animate-pulse rounded-full"></div>
        {:then url}
            <img
                src={url}
                alt={artist.name}
                class="w-full h-full object-cover rounded-full"
            />
        {/await}
    {:else}
        <div class="w-full h-full flex items-center justify-center rounded-full">
            <User size={64} class="text-neutral-700" />
        </div>
    {/if}
{/snippet}

<TrackList
    {tracks}
    name={artist.name}
    coverArt={null}
    {subtitle}
    coverSnippet={artistCover}
/>

{#if albums.length > 0}
    <div class="px-8 pb-8">
        <h2 class="text-2xl font-bold text-white mb-6">Albums</h2>
        <AlbumRow {albums} />
    </div>
{/if}
