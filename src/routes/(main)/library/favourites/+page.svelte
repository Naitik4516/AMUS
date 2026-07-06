<script lang="ts">
    import type { PageProps } from "./$types";
    import TrackList from "$components/ui/TrackList.svelte";
    import { Heart } from "@lucide/svelte";
    import { formatDuration } from "$lib/utils";

    let { data }: PageProps = $props();
    let tracks = $derived(data.tracks);

    let totalDuration = $derived(
        tracks.reduce((sum, track) => sum + track.duration_seconds, 0),
    );
</script>

<div
    class="relative flex flex-col rounded-2xl h-full w-full overflow-y-scroll px-4 pb-10"
>
    <div
        class="flex gap-15 items-end p-5 pb-30 rounded-t-2xl bg-linear-to-b from-rose-600 via-pink-900 to-fuchsia-950/20"
    >
        <div
            class="flex items-center justify-center rounded-2xl w-60 h-60 bg-rose-600"
        >
            <Heart fill="white" size="200" />
        </div>

        <div class="flex flex-col gap-4 min-w-0 pb-2">
            <h1
                class="text-3xl md:text-5xl lg:text-[6cqw] font-black font-switzer line-clamp-2"
            >
                Favourites
            </h1>
            <span class="text-gray-300">
                {tracks.length} songs, {formatDuration(totalDuration)}
            </span>
        </div>
    </div>

    {#if tracks.length > 0}
        <div class="-translate-y-22">
            <TrackList
                context={{ type: "Favorites", name: "Favorites" }}
                {tracks}
                canEdit={false}
            />
        </div>
    {:else}
        <div
            class="flex flex-col items-center justify-center py-20 text-gray-500 w-full"
        >
            <p class="text-xl font-medium">No favourite tracks</p>
            <p class="text-sm">
                You haven't added any tracks to your favourites yet.
            </p>
        </div>
    {/if}
</div>
