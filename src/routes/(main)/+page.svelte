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

<div class="pb-12 mr-6">
    <div>
        <Banner hasMusic={data.hasMusic} />
    </div>

    <div class="flex flex-col gap-10 py-10">
        {#each trackSections as section, i}
            <div>
                <TracksSection
                    title={section.title}
                    loadFunction={section.loadFunction}
                    args={section.args}
                />
            </div>
        {/each}

        <div data-scroll>
            <ArtistsSection
                title="Your Top Artists"
                loadFunction="get_top_artists"
            />
        </div>

        <div>
            <AlbumsSection
                title="Albums You Love"
                loadFunction="get_top_albums"
            />
        </div>

        <div>
            <TracksSection
                title="Remember These?"
                loadFunction="get_forgotten_tracks"
            />
        </div>
    </div>
</div>
