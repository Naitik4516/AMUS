<script lang="ts">
    import { page } from "$app/state";
    import TrackList from "$components/ui/TrackList.svelte";
    import AlbumRow from "$components/ui/AlbumRow.svelte";
    import { formatDuration } from "$lib/utils";
    import { store } from "$lib/stores.svelte";
    import { onMount } from "svelte";
    import type { Artist } from "$lib/types";
    import { gsap } from "gsap";
    import { ScrollTrigger } from "gsap/ScrollTrigger";
    import { getSwatches, type Color } from "colorthief";
    import type { Attachment } from "svelte/attachments";
    import { blur, fade, fly, slide } from "svelte/transition";
    import { cubicIn, cubicOut } from "svelte/easing";

    let artistId = $derived(Number(page.params.id));
    let artist = $derived(
        store.artists.find((a) => a.id === artistId) ??
            ({
                id: artistId,
                name: "Unknown Artist",
                profile_image: undefined,
                banner_image: undefined,
            } as Artist),
    );
    let tracks = $derived(store.tracksByArtist(artistId));
    let albums = $derived(store.albumsByArtist(artistId));

    let totalDuration = $derived(
        tracks.reduce((acc, track) => acc + track.duration_seconds, 0),
    );

    let gColor = $state<Color>();
    let aColor = $state<Color>();

    let bgUrl = $derived(store.getImageSrc(artist.banner_image, "artist"));

    onMount(() => {
        gsap.registerPlugin(ScrollTrigger);

        const tl = gsap.timeline({
            scrollTrigger: {
                trigger: ".artist-image",
                scroller: ".main-scroller",
                start: "top top",
                end: "+=400",
                pin: true,
                pinSpacing: false,
                scrub: true,
            },
        });

        tl.to(".banner", {
            opacity: 0,
            scale: 1.15,
            ease: "none",
        });

        return () => tl.scrollTrigger?.kill();
    });

    const ArtistImage: Attachment = (e) => {
        e.addEventListener("load", async () => {
            try {
                const swatches = await getSwatches(
                    e as unknown as HTMLImageElement,
                );
                aColor = swatches.Vibrant?.color;
                gColor = swatches.DarkMuted?.color;
            } catch (error) {
                console.error(
                    "Failed to extract color palette from cover art",
                    error,
                );
            }
        });
    };

    $inspect(gColor?.hex());
</script>

<div
    class="relative flex flex-col h-full w-full pb-10 overflow-hidden"
    in:fade={{
        duration: 500,
        easing: cubicIn,
    }}
    out:fade={{
        duration: 300,
    }}
>
    <div
        class="sticky artist-image z-0 top-0 flex justify-between px-2 mask-b-from-80% mask-r-from-90% mask-t-from-80%"
    >
        <div class="flex-1 min-w-0"></div>

        <div
            class="banner h-170 w-full flex relative"
            style="background: linear-gradient(to left, {gColor}, transparent);"
        >
            <img
                src={bgUrl}
                {@attach ArtistImage}
                alt={artist.name}
                class="h-full object-cover ml-auto aspect-auto z-1 mask-l-from-70%"
                crossorigin="anonymous"
            />
        </div>
    </div>

    <div class="flex flex-col gap-2 pb-2 min-w-0 px-8 -mt-90 z-10 pr-5">
        <h1
            class="text-3xl md:text-5xl lg:text-[6cqw] xl:[7cqw] max-text-[7rem] font-black font-clash bg-linear-to-b from-white from-50% to-gray-400 bg-clip-text text-transparent truncate drop-shadow-lg py-4.5 -mb-4"
        >
            {artist.name}
        </h1>
        <span class="text-gray-300 ml-5 font-satoshi font-semibold">
            {tracks.length} songs, {formatDuration(totalDuration)}
        </span>
    </div>

    {#if tracks.length > 0}
        <div class="mt-10 px-4 pr-8" transition:fade>
            <TrackList
                context={{
                    type: "Artist",
                    id: artist.id,
                    name: artist.name,
                    profileImage: artist.profile_image,
                    bannerImage: artist.banner_image,
                }}
                accentColor={aColor ? aColor.hex() : "#fff"}
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
