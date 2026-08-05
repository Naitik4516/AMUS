<script lang="ts">
    import HeroSection from "$components/ui/Home/HeroSection.svelte";
    import TracksSection from "$components/ui/Home/TracksSection.svelte";
    import ArtistsSection from "$components/ui/Home/ArtistsSection.svelte";
    import AlbumsSection from "$components/ui/Home/AlbumsSection.svelte";
    import type { InvokeArgs } from "@tauri-apps/api/core";
    import { store } from "$lib/stores.svelte";


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

    let recentlyAdded = $derived(store.recentlyAddedTracks.slice(0, 10));
    let favorites = $derived(store.favoriteTracks.slice(0, 10));
</script>

<div class="pr-6">
    <div class="mb-5" >
        <HeroSection />
    </div>

    <div class="flex flex-col gap-10 pb-5">
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
            <TracksSection title="Recently Added" tracks={recentlyAdded} />
        </div>

        <div>
            <TracksSection title="Favorites" tracks={favorites} />
        </div>

        <div>
            <ArtistsSection title="Your Top Artists" />
        </div>

        <div>
            <AlbumsSection title="Albums You Love" />
        </div>

        <div>
            <TracksSection
                title="Remember These?"
                loadFunction="get_forgotten_tracks"
            />
        </div>
    </div>
</div>
