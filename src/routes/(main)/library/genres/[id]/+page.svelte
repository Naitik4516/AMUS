<script lang="ts">
    import { page } from "$app/state";
    import TrackList from "$components/ui/TrackList.svelte";
    import EditGenreDialog from "$components/ui/Dialog/EditGenreDialog.svelte";
    import { formatDuration, sumDuration } from "$lib/utils";
    import { Music, Pen } from "@lucide/svelte";
    import { store } from "$lib/stores.svelte";

    let genreId = $derived(Number(page.params.id));
    let genre = $derived(store.genresById.get(genreId) ?? { id: genreId, name: "Unknown Genre" });
    let tracks = $derived(store.tracksByGenre(genreId));
    let editOpen = $state(false);

    let totalDuration = $derived(sumDuration(tracks));

    let thumbnailUrl = $derived(genre.thumbnail ? store.getImageSrc(genre.thumbnail) : null);
</script>

<div class="flex flex-col p-5 z-1 isolate">
    <div class="flex gap-10 items-end p-5 pb-8">
        <div class="relative group w-60 h-60 rounded-[70px] bg-linear-to-br from-indigo-700 to-purple-900 shadow-xl flex items-center justify-center overflow-hidden shrink-0">
            {#if thumbnailUrl}
                <img
                    src={thumbnailUrl}
                    alt={genre.name}
                    class="w-full h-full object-cover"
                />
            {:else}
                <Music class="text-fuchsia-100/60" size="120" />
            {/if}
            <button
                onclick={() => (editOpen = true)}
                class="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center"
            >
                <Pen size={32} class="text-white" />
            </button>
        </div>

        <div class="flex flex-col min-w-0 pb-2">
            <h1 class="text-3xl md:text-5xl lg:text-7xl font-black font-switzer line-clamp-2 drop-shadow-xl">
                {genre.name}
            </h1>
            <span class="text-gray-300 font-satoshi font-extrabold text-lg ml-2">
                {tracks.length} song{tracks.length !== 1 ? "s" : ""}, {formatDuration(totalDuration)}
            </span>
        </div>
    </div>

    {#if tracks.length > 0}
        <div>
            <TrackList
                context={{ type: "Genre", id: genreId, name: genre.name, thumbnail: genre.thumbnail ?? null }}
                {tracks}
            />
        </div>
    {:else}
        <div class="flex flex-col items-center justify-center py-20 text-gray-500 w-full">
            <Music size={64} class="mb-4 opacity-20" />
            <p class="text-xl font-medium">No tracks in this genre</p>
            <p class="text-sm">This genre doesn't have any tracks yet.</p>
        </div>
    {/if}
</div>

<EditGenreDialog
    bind:open={editOpen}
    genreId={genre.id}
    name={genre.name}
    thumbnail={genre.thumbnail ?? null}
/>
