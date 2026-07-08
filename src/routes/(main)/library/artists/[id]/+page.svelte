<script lang="ts">
    import { page } from "$app/state";
    import TrackList from "$components/ui/TrackList.svelte";
    import AlbumRow from "$components/ui/AlbumRow.svelte";
    import { User } from "@lucide/svelte";
    import { formatDuration } from "$lib/utils";
    import { fetchArtistImages } from "$lib/data.svelte";
    import { store } from "$lib/stores.svelte";
    import { onMount } from "svelte";
    import type { Artist } from "$lib/types";

    let artistId = $derived(Number(page.params.id));
    let artist = $derived(store.artistsById.get(artistId) ?? {
        id: artistId,
        name: "Unknown Artist",
        profile_image: undefined,
        banner_image: undefined,
    } as Artist);
    let tracks = $derived(store.tracksByArtist(artistId));
    let albums = $derived(store.albumsByArtist(artistId));

    let totalDuration = $derived(
        tracks.reduce((acc, track) => acc + track.duration_seconds, 0),
    );

    let profileUrl = $derived(store.getArtistProfileUrl(artist));
    let bgUrl = $derived(store.getArtistBannerUrl(artist) ?? store.getArtistProfileUrl(artist));
    let isBanner = $derived(!!artist.banner_image);

    onMount(async () => {
        if (!artist.banner_image) {
            await fetchArtistImages(artistId);
            await store.reloadArtists();
        }
    });
</script>

<div class="relative flex flex-col h-full w-full overflow-y-scroll pb-10 pr-5">
    <div
        class="banner-wrapper relative max-h-80 w-full aspect-video overflow-hidden rounded-t-2xl"
    >
        {#if bgUrl}
            <img
                src={bgUrl}
                alt=""
                class="w-full h-full object-cover banner"
                class:blur-2xl={!isBanner}
                class:scale-110={!isBanner}
            />
        {:else}
            <div class="w-full h-full bg-surface/50"></div>
        {/if}
    </div>

    <div class="flex items-end gap-6 px-6 -mt-32 relative z-10">
        <div
            class="w-48 h-48 rounded-full overflow-hidden ring-3 ring-black shadow-xl shrink-0"
        >
            {#if profileUrl}
                <img
                    src={profileUrl}
                    alt={artist.name}
                    class="w-full h-full object-cover"
                />
            {:else}
                <div
                    class="w-full h-full flex items-center justify-center bg-surface"
                >
                    <User size={48} />
                </div>
            {/if}
        </div>
        <div class="flex flex-col gap-2 pb-2 min-w-0">
            <h1
                class="text-3xl md:text-5xl lg:text-[6cqw] max-text-[6rem] font-black font-clash bg-linear-to-b from-white from-50% to-neutral-600 bg-clip-text text-transparent truncate drop-shadow-lg drop-shadow-black py-4.5 -mb-4"
            >
                {artist.name}
            </h1>
            <span class="text-gray-300">
                {tracks.length} songs, {formatDuration(totalDuration)}
            </span>
        </div>
    </div>

    {#if tracks.length > 0}
        <div class="mt-4 px-4">
            <TrackList
                context={{
                    type: "Artist",
                    id: artist.id,
                    name: artist.name,
                    profileImage: profileUrl,
                    bannerImage: bgUrl,
                }}
                {tracks}
            />
        </div>
    {:else}
        <div
            class="flex flex-col items-center justify-center py-20 text-gray-500 w-full"
        >
            <p class="text-xl font-medium">No tracks found</p>
            <p class="text-sm">This artist doesn't have any tracks yet.</p>
        </div>
    {/if}

    {#if albums.length > 0}
        <div class="px-4 mt-6">
            <AlbumRow title="Albums" {albums} />
        </div>
    {/if}
</div>

<style>
    .banner-wrapper {
        position: relative;
        mask-image: linear-gradient(to bottom, black 60%, transparent 100%);
        -webkit-mask-image: linear-gradient(
            to bottom,
            black 60%,
            transparent 100%
        );
    }
</style>
