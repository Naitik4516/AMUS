<script lang="ts">
    import HeroSection from "$components/ui/Home/HeroSection.svelte";
    import TracksSection from "$components/ui/Home/TracksSection.svelte";
    import ArtistsSection from "$components/ui/Home/ArtistsSection.svelte";
    import AlbumsSection from "$components/ui/Home/AlbumsSection.svelte";
    import type { InvokeArgs } from "@tauri-apps/api/core";
    import type { PageProps } from "./$types";
    import { store } from "$lib/stores.svelte";

    let { data }: PageProps = $props();


    type LoadFunction =
        | "get_recently_played"
        | "get_most_played_tracks"
        | "get_forgotten_tracks"
        | "get_unplayed_tracks";

    const trackSections: {
        title: string;
        loadFunction: LoadFunction;
        args?: InvokeArgs;
    }[] = [
        { title: "Continue Listening", loadFunction: "get_recently_played" },
        {
            title: "On repeat",
            loadFunction: "get_most_played_tracks",
            args: { timeframe: "this_month" },
        },
    ];
</script>

<div class="pb-12 pr-6">
    <div>
        <HeroSection hasMusic={data.hasMusic} />
    </div>

    <div class="flex flex-col gap-10 py-10">
        {#each trackSections as section}
            <div>
                <TracksSection
                    title={section.title}
                    loadFunction={section.loadFunction}
                    args={section.args}
                />
            </div>
        {/each}

        <div>
            <TracksSection title="Recently Added" tracks={store.recentlyAddedTracks.slice(0,10)} />
        </div>

        <div>
            <TracksSection title="Favorites" tracks={store.favoriteTracks.slice(0,10)} />
        </div>

        <div>
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
