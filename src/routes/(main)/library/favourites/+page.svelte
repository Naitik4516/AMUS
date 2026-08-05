<script lang="ts">
    import TrackList from "$components/ui/TrackList.svelte";
    import { Heart } from "@lucide/svelte";
    import { formatDuration, sumDuration } from "$lib/utils";
    import { store } from "$lib/stores.svelte";

    let tracks = $derived(store.favoriteTracks);

    let totalDuration = $derived(sumDuration(tracks));
</script>

<div
    class="fixed top-1/5 inset-x-25 h-90 blur-[120px] bg-linear-to-b from-rose-600 to-pink-900 z-0"
></div>

<div class="flex flex-col z-1 isolate">
    <div class="flex gap-10 items-end p-5 pb-8">
        <div
            class="flex items-center justify-center rounded-[70px] w-60 h-60 bg-linear-to-b from-pink-700  to-pink-900 shadow-xl"
        >
            <Heart class="fill-fuchsia-100 text-fuchsia-100" size="160" />
        </div>

        <div class="flex flex-col min-w-0 pb-2">
            <h1
                class="text-3xl md:text-5xl lg:text-7xl xl:text-[180px] font-black font-switzer line-clamp-2 text-transparent bg-clip-text bg-linear-to-br from-rose-500 to-pink-600 drop-shadow-xl"
            >
                Favourites
            </h1>
            <span
                class="text-gray-300 font-satoshi font-extrabold text-lg ml-2"
            >
                {tracks.length} songs, {formatDuration(totalDuration)}
            </span>
        </div>
    </div>

    {#if tracks.length > 0}
        <div>
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
