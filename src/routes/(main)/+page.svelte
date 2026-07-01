<script lang="ts">
    import Banner from "$components/ui/Home/Banner.svelte";
    import TracksSection from "$components/ui/Home/TracksSection.svelte";
    import ArtistsSection from "$components/ui/Home/ArtistsSection.svelte";
    import AlbumsSection from "$components/ui/Home/AlbumsSection.svelte";
    import type { InvokeArgs } from "@tauri-apps/api/core";

    import type { PageProps } from "./$types";

    let { data }: PageProps = $props();

    type LoadFunction =
        | "get_recently_played"
        | "get_most_played_tracks"
        | "get_favorite_tracks"
        | "get_forgotten_tracks"
        | "get_unplayed_tracks"
        | "get_recently_added";

    const trackSections: {
        title: string;
        loadFunction: LoadFunction;
        args?: InvokeArgs;
    }[] = [
        { title: "Continue Listening", loadFunction: "get_recently_played" },
        { title: "Recently Added", loadFunction: "get_recently_added" },
        {
            title: "On repeat",
            loadFunction: "get_most_played_tracks",
            args: { timeframe: "this_month" },
        },
        { title: "Favorites", loadFunction: "get_favorite_tracks" },
    ];
</script>

<div class="container">
    <div
        class="reveal-section"
        data-scroll
        data-scroll-class="section-reveal"
        data-scroll-offset="0, 5%"
    >
        <Banner hasMusic={data.hasMusic} />
    </div>

    <div class="flex flex-col gap-8 py-10">
        {#each trackSections as section}
            <TracksSection
                title={section.title}
                loadFunction={section.loadFunction}
                args={section.args}
            />
        {/each}

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
