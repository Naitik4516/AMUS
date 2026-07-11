<script lang="ts">
    import LibraryCard from "$components/ui/Card/LibraryCard.svelte";
    import {
        ListMusic,
        Heart,
        ChartNoAxesColumn,
        Music2,
        Disc,
    } from "@lucide/svelte";
    import ArtistIcon from "$components/icons/Artist.svelte";
</script>

<div
    class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6 w-full p-8 h-fit"
>
    <LibraryCard
        label="All Tracks"
        Icon={Music2}
        href="/library/playlists/-1"
    />
    <LibraryCard label="Artists" Icon={ArtistIcon} href="/library/artists" />
    <LibraryCard label="Albums" Icon={Disc} href="/library/albums" />
    <LibraryCard label="Playlists" Icon={ListMusic} href="/library/playlists" />
    <LibraryCard label="Favorites" Icon={Heart} href="/library/favourites" />
    <LibraryCard label="Stats" Icon={ChartNoAxesColumn} href="/library/stats" />
</div>
