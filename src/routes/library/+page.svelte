<script lang="ts">
    import LibraryCard from "$components/ui/LibraryCard.svelte";
    import {
        ListMusic,
        Heart,
        ChartNoAxesColumn,
        Music2,
        Disc,
    } from "@lucide/svelte";
</script>

{#snippet ArtistIcon()}
    <svg
        xmlns="http://www.w3.org/2000/svg"
        height="24px"
        viewBox="0 -960 960 960"
        width="24px"
        fill="#e3e3e3"
        ><path
            d="M629-189q-29-29-29-71t29-71q29-29 71-29 8 0 18 1.5t22 6.5v-168q0-17 11.5-28.5T780-560h60q17 0 28.5 11.5T880-520q0 17-11.5 28.5T840-480h-40v220q0 42-29 71t-71 29q-42 0-71-29ZM327-527q-47-47-47-113t47-113q47-47 113-47t113 47q47 47 47 113t-47 113q-47 47-113 47t-113-47ZM160-160q-17 0-28.5-11.5T120-200v-72q0-35 17.5-63t46.5-43q62-31 126-46.5T440-440q28 0 55.5 3t55.5 9q17 4 21.5 21t-9.5 31q-21 25-31.5 54.5T521-260q0 13 1.5 25.5T528-209q5 18-4.5 33.5T497-160H160Z"
        /></svg
    >
{/snippet}

<div
    class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6 w-full p-8 h-fit"
>
    <LibraryCard
        label="All Tracks"
        Icon={Music2}
        href="/library/playlists/0?name=All%20Tracks"
    />
    <LibraryCard
        label="Artists"
        Icon={ArtistIcon}
        href="/library/artists"
        isSnippet={true}
    />
    <LibraryCard label="Albums" Icon={Disc} href="/library/albums" />
    <LibraryCard label="Playlists" Icon={ListMusic} href="/library/playlists" />
    <LibraryCard label="Favorites" Icon={Heart} href="/library/favourites" />
    <LibraryCard label="Stats" Icon={ChartNoAxesColumn} href="/library/stats" />
</div>
