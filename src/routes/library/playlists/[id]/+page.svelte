<script lang="ts">
    import { Trash2 } from "@lucide/svelte";
    import type { Track } from "$lib/types.d.ts";
    import type { PageProps } from "./$types";
    import MenuItem from "$components/ui/MenuItem.svelte";
    import TrackList from "$components/ui/TrackList.svelte";

    let { data }: PageProps = $props();
    let tracks = $derived((data.data as Track[]) || []);
    let name = $derived(data.name || "Playlist");
    let coverArt = $derived.by(() => {
        let arts = [];
        for (let track of tracks) {
            if (track.cover_art) arts.push(track.cover_art);
            if (arts.length >= 4) break;
        }
        if (arts.length === 0) return null;
        else if (arts.length === 4) return arts;
        else return arts[0];
    });

</script>

{#snippet playlistTrackOptions()}
    <MenuItem Icon={Trash2} label="Remove from Playlist" onclick={() => {}} />
{/snippet}

<TrackList
    {tracks}
    {name}
    {coverArt}
    otherMenuItems={playlistTrackOptions}
/>
