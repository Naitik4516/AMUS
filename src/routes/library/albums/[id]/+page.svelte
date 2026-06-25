<script lang="ts">
    import type { Track } from "$lib/types.d.ts";
    import type { PageProps } from "./$types";
    import TrackList from "$components/ui/TrackList.svelte";
    import { formatDuration, getImageUrl } from "$lib/utils";
    import { Disc } from "@lucide/svelte";
    import { getColorSync, type HSL, createColor } from "colorthief";

    let { data }: PageProps = $props();
    let tracks = $derived((data.data as Track[]) || []);
    let name = $derived(data.name || "Album");
    let coverArt = $derived(data?.cover_art || null);
    let accentColorHsl = $state(createColor(0, 0, 0, 0, 0).hsl());
    let coverImageEl: HTMLImageElement | null = $state(null);

    let totalDuration = $derived(
        tracks.reduce((sum, track) => sum + track.duration_seconds, 0),
    );

    function getDarkenedHslGradient(
        hsl: HSL,
        targetLightness: number = 15,
        baseLightness: number = hsl.l,
    ): string {
        const darkenedL = Math.min(hsl.l, targetLightness);

        const darkColor = `hsl(${hsl.h}, ${hsl.s}%, ${darkenedL}%)`;
        const baseColor = `hsl(${hsl.h}, ${hsl.s * 5}%, ${baseLightness}%)`;

        return `linear-gradient(to bottom, ${baseColor}, ${darkColor} 80%, transparent)`;
    }

    $effect(() => {
        if (!coverImageEl) return;

        const handleColorExtraction = () => {
            try {
                // getColorSync returns an array like [R, G, B]
                const dominantColor = getColorSync(coverImageEl);
                if (dominantColor) {
                    accentColorHsl = dominantColor.hsl();
                }
            } catch (e) {
                console.error("Failed to extract color from cover art", e);
            }
        };

        // If the image is cached and already done loading, run it immediately
        if (coverImageEl.complete) {
            handleColorExtraction();
        } else {
            // Otherwise, wait for the network request to finish
            coverImageEl.addEventListener("load", handleColorExtraction);
        }

        // Cleanup listener if coverImageEl changes or component unmounts
        return () => {
            coverImageEl?.removeEventListener("load", handleColorExtraction);
        };
    });
</script>

<div
    class="relative flex flex-col rounded-2xl h-full w-full overflow-y-scroll px-4 pb-10"
>
    <div
        class="flex gap-15 items-end p-5 pb-30 rounded-t-2xl"
        style="background: {getDarkenedHslGradient(accentColorHsl, 10)}"
    >
        <div class="aspect-square w-60 shrink-0">
            {#if !coverArt}
                <div class="absolute inset-0 flex items-center justify-center">
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        height="48px"
                        width="48px"
                        viewBox="0 -960 960 960"
                        fill="#e3e3e3"
                        ><path
                            d="M480-300q75 0 127.5-52.5T660-480q0-75-52.5-127.5T480-660q-75 0-127.5 52.5T300-480q0 75 52.5 127.5T480-300Zm-28.5-151.5Q440-463 440-480t11.5-28.5Q463-520 480-520t28.5 11.5Q520-497 520-480t-11.5 28.5Q497-440 480-440t-28.5-11.5ZM480-80q-83 0-156-31.5T197-197q-54-54-85.5-127T80-480q0-83 31.5-156T197-763q54-54 127-85.5T480-880q83 0 156 31.5T763-763q54 54 85.5 127T880-480q0 83-31.5 156T763-197q-54 54-127 85.5T480-80Z"
                        /></svg
                    >
                </div>
            {:else}
                <img
                    src={coverArt}
                    alt={name}
                    class="w-full h-full object-cover rounded-lg shadow-2xl"
                    crossorigin="anonymous"
                    bind:this={coverImageEl}
                />
            {/if}
        </div>

        <div class="flex flex-col gap-4 min-w-0">
            <h1
                class="text-3xl md:text-5xl lg:text-[4cqw] max-text-[6rem] font-black line-clamp-2"
            >
                {name}
            </h1>
            <span class="text-gray-300">
                {tracks.length} songs, {formatDuration(totalDuration, true)}
            </span>
        </div>
    </div>

    {#if tracks.length > 0}
        {@const lightness = Math.min(accentColorHsl.l, 40)}
        <div class="-translate-y-22">
            <TrackList {tracks} />
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

    <div
        class="fixed w-100 h-100 blur-[180px] bg-green-400/30 -bottom-40 left-30 rounded-full -z-15"
    ></div>
    <div
        class="absolute w-90 h-90 blur-[150px] bg-pink-400/30 bottom-10 right-20 rounded-full -z-15"
    ></div>
</div>
