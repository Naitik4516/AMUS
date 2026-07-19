<script lang="ts">
    import { ListMusic } from "@lucide/svelte";
    import { store } from "$lib/stores.svelte";
    import type { Track, Playlist } from "$lib/types";

    let { playlist, playlistTracks, ...props } = $props();

    let tracks = $derived(
        playlistTracks ?? store.tracksByPlaylist(playlist.id),
    );

    const coverArts = $derived.by(() => {
        if (playlist.cover_art) return playlist.cover_art;

        let coverArts: string[] = [];

        for (const track of tracks) {
            if (coverArts.length >= 4) break;
            if (track.cover_art && !coverArts.includes(track.cover_art)) {
                coverArts.push(track.cover_art);
            }
        }

        if (coverArts.length === 1) return coverArts[0];
        if (coverArts.length === 4) return coverArts;
        return null;
    });
</script>

{#snippet fallBackPlaylistCoverArt()}
    <div class="w-full h-full bg-gray-300/5 flex items-center justify-center">
        <ListMusic class="text-slate-400 size-1/2" />
    </div>
{/snippet}

{#if !tracks || tracks.length === 0}
    {@render fallBackPlaylistCoverArt()}
{:else}
    {#if typeof coverArts === "string"}
        <img
            src={store.getImageSrc(coverArts)}
            alt={playlist.name}
            class="w-full h-full object-cover"
            {...props}
        />
    {:else if Array.isArray(coverArts)}
        <div class="grid grid-cols-2 grid-rows-2 w-full h-full">
            {#each coverArts as cover}
                <img
                    src={store.getImageSrc(cover)}
                    alt={playlist.name}
                    class="w-full h-full object-cover"
                    {...props}
                />
            {/each}
        </div>
    {:else}
        {@render fallBackPlaylistCoverArt()}
    {/if}
{/if}
