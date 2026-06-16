<script lang="ts">
    import type { Track } from "$lib/types.d.ts";
    import type { PageProps } from "./$types";
    import TrackList from "$components/ui/TrackList.svelte";
    import { Disc } from "@lucide/svelte";

    let { data }: PageProps = $props();
    let tracks = $derived((data.data as Track[]) || []);
    let name = $derived(data.name || "Album");
    let coverArt = $derived(data?.cover_art || null);
</script>

{#if tracks.length > 0}
    <TrackList {tracks} {name} {coverArt} />
{:else}
    <div class="flex flex-col items-center justify-center py-20 text-gray-500 w-full">
        <Disc size={64} class="mb-4 opacity-20" />
        <p class="text-xl font-medium">No tracks in this album</p>
        <p class="text-sm">This album doesn't have any tracks yet.</p>
    </div>
{/if}
