<script lang="ts">
    import Banner from "$components/ui/Home/Banner.svelte";
    import TracksSection from "$components/ui/Home/TracksSection.svelte";
    import ArtistsSection from "$components/ui/Home/ArtistsSection.svelte";
    import AlbumsSection from "$components/ui/Home/AlbumsSection.svelte";

    import type { PageProps } from "./$types";

    let { data }: PageProps = $props();

    const rootStyles = getComputedStyle(document.documentElement);
    const primaryColor = rootStyles.getPropertyValue('--primary-color');
    console.log(primaryColor); // "#336699"

</script>

<div class="container">
    <Banner hasMusic={data.hasMusic} />

    <div class="flex flex-col gap-8 py-10">
        <TracksSection
            title="Continue Listening"
            loadFunction="get_recently_played"
        />

        <TracksSection
            title="Recently Added"
            loadFunction="get_recently_added"
        />

        <TracksSection
            title="On repeat"
            loadFunction="get_most_played_tracks"
            args={{ timeframe: "this_month" }}
        />

        <TracksSection title="Favorites" loadFunction="get_favorite_tracks" />

        <ArtistsSection
            title="Your Top Artists"
            loadFunction="get_top_artists"
        />

        <AlbumsSection title="Albums You Love" loadFunction="get_top_albums" />

        <TracksSection
            title="Remember These?"
            loadFunction="get_forgotten_tracks"
        />
    </div>
</div>
