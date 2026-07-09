<script lang="ts">
    import { goto } from "$app/navigation";
    import { player } from "$lib/player.svelte";
    import type { SearchableItem } from "$lib/types";
    import { store } from "$lib/stores.svelte";
    import { slide } from "svelte/transition";
    import { cubicOut } from "svelte/easing";
    import { Search, X, Music } from "@lucide/svelte";
    import Fuse from "fuse.js";

    let searchQuery = $state("");
    let results = $state<SearchableItem[]>([]);
    let showResults = $state(false);
    let searchTimeout: any;
    let selectedIndex = $state(0);
    let resultsContainer: HTMLDivElement | undefined = $state();

    let fuse: Fuse<SearchableItem>;

    let searchIndex = $derived([
        ...store.tracks.map((track) => ({
            ...track,
            type: "track" as const,
        })),
        ...store.artists.map((artist) => ({
            ...artist,
            type: "artist" as const,
        })),
        ...store.albums.map((album) => ({
            ...album,
            type: "album" as const,
        })),
        ...store.playlists.map((playlist) => ({
            ...playlist,
            type: "playlist" as const,
        })),
    ]);

    $effect(() => {
        fuse = new Fuse(searchIndex, {
            keys: [
                { name: "title", weight: 2 }, // Matches Track.title
                { name: "name", weight: 2 }, // Matches Artist.name, Album.name, Playlist.name

                { name: "artists.name", weight: 1 }, // Matches Track.artists[].name
                { name: "album.name", weight: 1 }, // Matches Track.album.name
                { name: "album_artist.name", weight: 1 }, // Matches Album.album_artist[].name
            ],
            threshold: 0.2,
            useExtendedSearch: true,
        });
        console.log("Fuse.js initialized with items:", fuse.getIndex().size());
    });

    $effect(() => {
        if (showResults && results.length > 0 && resultsContainer) {
            const child = resultsContainer.children[selectedIndex] as
                HTMLElement | undefined;
            child?.scrollIntoView({ block: "nearest" });
        }
    });

    function closeDropdown() {
        results = [];
        showResults = false;
        searchQuery = "";
    }

    function handleSearch() {
        if (searchTimeout) clearTimeout(searchTimeout);

        const trimmed = searchQuery.trim();
        if (trimmed.length < 1) {
            results = [];
            showResults = false;
            return;
        }

        selectedIndex = 0;

        searchTimeout = setTimeout(() => {
            results = fuse.search(trimmed).map((r) => r.item);
            showResults = true;
        }, 300);
    }

    function handleKeyDown(e: KeyboardEvent) {
        if (!showResults || results.length === 0) {
            if (e.key === "Escape") closeDropdown();
            return;
        }

        if (e.key === "ArrowDown") {
            e.preventDefault();
            selectedIndex = (selectedIndex + 1) % results.length;
        } else if (e.key === "ArrowUp") {
            e.preventDefault();
            selectedIndex =
                (selectedIndex - 1 + results.length) % results.length;
        } else if (e.key === "Enter") {
            e.preventDefault();
            const r = results[selectedIndex];
            if (r) handleResultClick(r);
        } else if (e.key === "Escape") {
            closeDropdown();
        }
    }

    function handleResultClick(result: SearchableItem) {
        console.log("Clicked result:", result);
        if (result.type === "track") {
            player.play([result], { type: "Search" });
        } else if (result.type === "artist") {
            goto(`/library/artists/${result.id}`);
        } else if (result.type === "album") {
            goto(`/library/albums/${result.id}`);
        } else if (result.type === "playlist") {
            goto(`/library/playlists/${result.id}`);
        }
        closeDropdown();
    }

    function getCoverUrl(result: SearchableItem): string | null {
        if (result.type === "track") {
            return store.getImageSrc(result.cover_art);
        }
        if (result.type === "album") {
            return store.getImageSrc(result.cover_art);
        }
        if (result.type === "artist") {
            return store.getImageSrc(result.profile_image, "artist");
        }
        return null;
    }
</script>

<div class="relative">
    <div
        class="flex items-center gap-2 bg-card/40 backdrop-blur-xl shadow-lg px-4 py-1 mt-5 h-full border-2 transition-colors duration-300 hover:bg-card/20 focus-within:bg-card/40 focus:within:shadow-card/60 {showResults
            ? 'rounded-t-4xl border-b-0 '
            : 'rounded-full'} "
    >
        <Search size={16} class="text-gray-400" />
        <input
            id="global-search-input"
            type="text"
            placeholder="What do you want to listen to?"
            bind:value={searchQuery}
            oninput={handleSearch}
            onkeydown={handleKeyDown}
            onfocus={() =>
                searchQuery.trim().length >= 1 && (showResults = true)}
            onblur={closeDropdown}
            class="w-60 py-4 outline-none bg-transparent text-white placeholder-gray-400 text-sm focus:w-120 transition-all duration-300"
        />
        {#if searchQuery}
            <button onclick={() => (searchQuery = "")}>
                <X size={14} class="text-gray-400 hover:text-white" />
            </button>
        {/if}
    </div>

    {#if showResults}
        <div
            class="absolute top-full left-0 right-0 p-2 backdrop-blur-2xl border-2 border-t-0 rounded-b-4xl shadow-2xl overflow-hidden z-50 max-h-[60vh] h-auto overflow-y-auto bg-card/40 scrollbar-none"
            bind:this={resultsContainer}
            onblur={() => (showResults = false)}
            role="listbox"
            transition:slide={{ duration: 200, easing: cubicOut }}
        >
            {#if results.length === 0}
                <div class="px-4 py-3 text-sm text-gray-400 text-center">
                    No results found
                </div>
            {:else}
                {#each results as result, i}
                    <button
                        class="w-full flex items-center gap-4 px-2 py-2 mb-1 transition-colors group text-left rounded-xl {i ===
                        selectedIndex
                            ? 'bg-white/5 shadow-md'
                            : 'hover:bg-white/5'}"
                        onclick={() => handleResultClick(result)}
                        role="option"
                        aria-selected={i === selectedIndex}
                    >
                        <div
                            class="w-10 h-10 relative {result.type === 'artist'
                                ? 'rounded-full'
                                : 'rounded-lg'} bg-neutral-800 flex items-center justify-center overflow-hidden shrink-0"
                        >
                            {#if getCoverUrl(result)}
                                <img
                                    src={getCoverUrl(result)!}
                                    alt=""
                                    class="w-full h-full object-cover"
                                />
                            {:else}
                                <Music size={20} class="text-gray-400" />
                            {/if}
                        </div>
                        <div class="flex-1 min-w-0">
                            {#if result.type === "track"}
                                <p
                                    class="text-sm font-semibold truncate text-white"
                                >
                                    {result.title}
                                </p>
                                <p class="text-xs text-gray-300 truncate">
                                    {result.artists
                                        .map((a) => a.name)
                                        .join(", ") || "Unknown Artist"}
                                </p>
                            {:else if result.type === "artist"}
                                <p
                                    class="text-sm font-semibold truncate text-white"
                                >
                                    {result.name}
                                </p>
                                <p class="text-xs text-gray-300">Artist</p>
                            {:else if result.type === "album"}
                                <p
                                    class="text-sm font-semibold truncate text-white"
                                >
                                    {result.name}
                                </p>
                                <p class="text-xs text-gray-300">
                                    Album
                                    {#if result.year}
                                        &middot; {result.year}
                                    {/if}
                                </p>
                            {:else if result.type === "playlist"}
                                <p
                                    class="text-sm font-semibold truncate text-white"
                                >
                                    {result.name}
                                </p>
                                <p class="text-xs text-gray-300">Playlist</p>
                            {/if}
                        </div>
                    </button>
                {/each}
            {/if}
        </div>
    {/if}
</div>
