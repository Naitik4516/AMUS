<script lang="ts">
    import type { Track } from "$lib/types.d.ts";
    import type { PageProps } from "./$types";
    import TrackList from "$components/ui/TrackList.svelte";
    import { formatDuration } from "$lib/utils";
    import { Disc } from "@lucide/svelte";
    import { getSwatchesSync, type HSL, type Color } from "colorthief";
    import type { Attachment } from "svelte/attachments";
    import Artist from "$components/icons/Artist.svelte";
    import { getImageUrl } from "$lib/utils";

    let { data }: PageProps = $props();
    let tracks = $derived((data.data as Track[]) || []);
    let name = $derived(data.name || "Album");
    let coverArtFilename = $derived(data.coverArtFilename ?? null);
    let albumArtist = $derived(data.albumInfo?.album_artist || []);
    let coverArt = $derived(data?.cover_art || null);

    let dominantColor = $state<Color>();
    let color1 = $state<Color>();
    let color2 = $state<Color>();

    let totalDuration = $derived(
        tracks.reduce((sum, track) => sum + track.duration_seconds, 0),
    );

    function getDarkenedHslGradient(
        hsl: HSL,
        targetLightness: number = 15,
    ): string {
        const darkenedL = Math.min(hsl.l, targetLightness);

        const darkColor = `hsl(${hsl.h}, ${hsl.s}%, ${darkenedL}%)`;
        const baseColor = `hsl(${hsl.h}, ${hsl.s}%, ${hsl.l}%)`;

        return `linear-gradient(to bottom, ${baseColor}, ${darkColor} 80%, transparent)`;
    }

    const CoverImage: Attachment = (e) => {
        e.addEventListener("load", () => {
            try {
                const swatches = getSwatchesSync(e);
                console.log("Extracted swatches:", swatches);
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
    <div class="flex gap-15 items-end p-5 pb-30">
        <img
            src={coverArt ? coverArt : "/PhonographRecord.png"}
            alt={name}
            class="w-64 rounded-xl shadow-xl"
            crossorigin="anonymous"
            {@attach CoverImage}
        />

        <div class="flex flex-col gap-4 min-w-0 pb-2">
            <h1
                class="text-3xl md:text-5xl lg:text-[5cqw] max-text-[6rem] font-switzer font-black drop-shadow-lg line-clamp-2"
            >
                {name}
            </h1>
            {#if albumArtist.length > 0}
                {#each albumArtist as artist (artist.id)}
                    <div class="flex gap-1 items-center font-medium">
                        {#if artist.profile_image}
                            <img
                                src={await getImageUrl(
                                    artist.profile_image,
                                    "artist",
                                )}
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
            <span class="text-gray-300">
                {tracks.length} songs, {formatDuration(totalDuration)}
            </span>
        </div>
    </div>

    {#if tracks.length > 0}
        <div class="-translate-y-22">
            <TrackList
                context={{
                    type: "Album",
                    id: Number(data.albumInfo?.id ?? 0),
                    name,
                    coverArt: coverArtFilename,
                }}
                {tracks}
                canSort={false}
            />
        </div>
    {:else}
        <div
            class="flex flex-col items-center justify-center py-20 text-gray-500 w-full"
        >
            <Disc size={64} class="mb-4 opacity-20" />
            <p class="text-xl font-medium">No tracks in this album</p>
            <p class="text-sm">This album doesn't have any tracks yet.</p>
        </div>
    {/if}
</div>
<div
    class="fixed w-100 h-100 blur-[180px] -bottom-40 left-30 rounded-full"
    style:background="{color1?.hex()}4D"
></div>
<div
    class="absolute w-90 h-90 blur-[150px] bottom-10 right-20 rounded-full"
    style:background="{color2?.hex()}99"
></div>

<div
    class="fixed w-[80vw] h-50 top-30 right-5 blur-[150px]"
    style:background={dominantColor?.css()}
></div>
