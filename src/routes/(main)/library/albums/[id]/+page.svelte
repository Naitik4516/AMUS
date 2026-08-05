<script lang="ts">
    import { page } from "$app/state";
    import Artist from "$components/icons/Artist.svelte";
    import TrackList from "$components/ui/TrackList.svelte";
    import { store } from "$lib/stores.svelte";
    import { formatDuration, sumDuration } from "$lib/utils";
    import { Disc } from "@lucide/svelte";
    import { getSwatchesSync, type Color } from "colorthief";
    import type { Attachment } from "svelte/attachments";
    import { fade } from "svelte/transition";

    let albumId = $derived(Number(page.params.id));
    let tracks = $derived(store.tracksByAlbum(albumId));
    let album = $derived(store.albums.find((a) => a.id === albumId));
    let name = $derived(album?.name ?? "Album");
    let coverArt = $derived(album ? store.getImageSrc(album.cover_art) : null);
    let coverArtFilename = $derived(album?.cover_art ?? null);
    let albumArtist = $derived(album?.album_artist || []);

    let dominantColor = $state<Color>();
    let color1 = $state<Color>();
    let color2 = $state<Color>();

    let totalDuration = $derived(sumDuration(tracks));

    const CoverImage: Attachment = (e) => {
        e.addEventListener("load", () => {
            try {
                const swatches = getSwatchesSync(
                    e as unknown as HTMLVideoElement,
                );
                dominantColor = swatches.Vibrant
                    ? swatches.Vibrant.color
                    : swatches.Muted?.color;
                color1 = swatches.LightVibrant
                    ? swatches.LightVibrant.color
                    : swatches.LightMuted?.color;
                color2 = swatches.DarkVibrant
                    ? swatches.DarkVibrant.color
                    : swatches.DarkMuted?.color;
            } catch (error) {
                console.error(
                    "Failed to extract color palette from cover art",
                    error,
                );
            }
        });
    };
</script>

<div
    class="relative flex flex-col h-full w-full overflow-y-scroll px-6 pb-10 z-1"
>
    <div class="flex gap-10 items-end p-5 pb-6" in:fade>
        <img
            src={coverArt ? coverArt : "/PhonographRecord.webp"}
            alt={name}
            class="w-64 {coverArt
                ? 'rounded-2xl shadow-xl'
                : 'drop-shadow-xl drop-shadow-black/50'}"
            crossorigin="anonymous"
            {@attach CoverImage}
        />

        <div class="flex flex-col gap-4 min-w-0 pb-1">
            <h1
                class="text-3xl md:text-5xl lg:text-6xl xl:text-7xl max-text-[7rem] text-left font-switzer font-black drop-shadow-lg line-clamp-2"
            >
                {name}
            </h1>
            <div class="ml-2 flex flex-col gap-1">
                {#if albumArtist.length > 0}
                    {#each albumArtist as artist (artist.id)}
                        <div class="flex gap-1 items-center font-medium">
                            {#if artist.profile_image}
                                <img
                                    src={store.getImageSrc(
                                        artist.profile_image,
                                        "artist",
                                    ) ?? ""}
                                    alt={artist.name}
                                    class="w-6 h-6 rounded-full object-cover"
                                />
                            {:else}
                                <Artist size={24} class="text-gray-400" />
                            {/if}
                            <a
                                href={`/library/artists/${artist.id}`}
                                class="hover:text-white text-sm transition-colors"
                                >{artist.name}</a
                            >
                        </div>
                    {/each}
                {/if}
                <span class="text-gray-300 font-mono font-medium pl-1">
                    {tracks.length} songs, {formatDuration(totalDuration)}
                </span>
            </div>
        </div>
    </div>

    {#if tracks.length > 0}
        <div>
            <TrackList
                context={{
                    type: "Album",
                    id: albumId,
                    name,
                    coverArt: coverArtFilename,
                }}
                {tracks}
                canSort={false}
            />
        </div>
    {:else}
        <div
            class="flex flex-col items-center justify-center py-20 text-gray-400 w-full"
        >
            <Disc size={80} class="mb-4 text-gray-600" />
            <p class="text-2xl font-semibold">No tracks in this album</p>
            <p class="text-base font-medium">This album doesn't have any tracks yet.</p>
        </div>
    {/if}
</div>
<div
    class="fixed w-100 h-100 blur-[180px] -bottom-40 left-30 rounded-full"
    style:background="{color1?.hex()}4D"
    in:fade={{ duration: 300, delay: 200 }}
></div>
<div
    class="absolute w-90 h-90 blur-[150px] bottom-10 right-20 rounded-full"
    style:background="{color2?.hex()}99"
    in:fade={{ duration: 300, delay: 200 }}
></div>

<div
    class="fixed w-[80vw] h-50 top-30 right-5 blur-[150px]"
    style:background={dominantColor?.css()}
    in:fade={{ duration: 300, delay: 200 }}
></div>
