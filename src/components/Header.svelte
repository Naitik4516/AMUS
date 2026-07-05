<script lang="ts">
    import {
        X,
        Minus,
        Search,
        Music,
        Maximize2,
        Minimize2,
    } from "@lucide/svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { invoke } from "@tauri-apps/api/core";
    import { goto } from "$app/navigation";
    import { getImageUrl } from "$lib/utils";
    import { player } from "$lib/player.svelte";
    import type { GlobalSearchResult } from "$lib/types";

    let { isMaximized = $bindable(false) } = $props();

    let searchQuery = $state("");
    let results = $state<GlobalSearchResult[]>([]);
    let showResults = $state(false);
    let searchTimeout: any;
    let selectedIndex = $state(0);
    let resultsContainer: HTMLDivElement | undefined = $state();

    $effect(() => {
        const appWindow = getCurrentWindow();
        let unlisten: () => void;

        appWindow.isMaximized().then((max) => {
            isMaximized = max;
        });

        appWindow
            .listen("tauri://resize", async () => {
                isMaximized = await appWindow.isMaximized();
            })
            .then((unlistenFn) => {
                unlisten = unlistenFn;
            });

        return () => {
            if (unlisten) unlisten();
        };
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

    async function handleSearch() {
        if (searchTimeout) clearTimeout(searchTimeout);

        const trimmed = searchQuery.trim();
        if (trimmed.length < 2) {
            results = [];
            showResults = false;
            return;
        }

        selectedIndex = 0;

        searchTimeout = setTimeout(async () => {
            try {
                results = await invoke("global_search", {
                    query: trimmed,
                    limit: 20,
                });
                showResults = true;
            } catch (e) {
                console.error("Search failed", e);
            }
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

    function handleResultClick(result: GlobalSearchResult) {
        if (result.result_type === "track" && result.track) {
            player.play(result.track);
        } else if (result.result_type === "artist" && result.artist) {
            goto(`/library/artists/${result.artist.id}`);
        } else if (result.result_type === "album" && result.album) {
            goto(`/library/albums/${result.album.id}`);
        } else if (result.result_type === "playlist" && result.playlist) {
            goto(`/library/playlists/${result.playlist.id}`);
        }
        closeDropdown();
    }

    function toggleMaximize() {
        const appWindow = getCurrentWindow();
        appWindow.toggleMaximize();
    }

    function minimize() {
        const appWindow = getCurrentWindow();
        appWindow.minimize();
    }

    function close() {
        const appWindow = getCurrentWindow();
        appWindow.close();
    }
</script>

<header class="sticky top-0 z-20">
    <div class="h-2 shrink-0"></div>
    <div
        data-tauri-drag-region
        class="flex items-center px-4 h-12 justify-between select-none text-white"
    >
        <div class="w-16 flex justify-center shrink-0 mt-1 pointer-events-none">
            <img
                src="/icon.svg"
                alt="Avatar"
                class="w-10 h-10 object-contain select-none"
            />
        </div>

        <div class="relative">
            <div
                class="flex items-center gap-2 bg-card/40 backdrop-blur-md rounded-full px-4 py-1 mt-5 h-full border-2 transition-colors duration-300 hover:bg-card/75 focus-within:bg-card"
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
                    class="absolute top-full left-0 right-0 mt-2 backdrop-blur-xl border-2 border-border rounded-4xl shadow-2xl overflow-hidden py-1 z-50 max-h-[50vh] h-auto overflow-y-auto bg-linear-to-b from-transparent via-gray-400/10 to-transparent scrollbar-none"
                    bind:this={resultsContainer}
                    onblur={() => (showResults = false)}
                    role="listbox"
                >
                    {#if results.length === 0}
                        <div
                            class="px-4 py-3 text-sm text-gray-400 text-center"
                        >
                            No results found
                        </div>
                    {:else}
                        {#each results as result, i}
                            <button
                                class="w-full flex items-center gap-4 px-2 py-2 transition-colors group text-left {i ===
                                selectedIndex
                                    ? 'bg-white/10'
                                    : 'hover:bg-white/5'}"
                                onclick={() => handleResultClick(result)}
                                role="option"
                                aria-selected={i === selectedIndex}
                            >
                                <div
                                    class="w-10 h-10 relative rounded-lg bg-neutral-800 flex items-center justify-center overflow-hidden shrink-0"
                                >
                                    {#if result.result_type === "track" && result.track?.cover_art}
                                        <img
                                            src={await getImageUrl(
                                                result.track.cover_art,
                                            )}
                                            alt={result.track.title}
                                            class="w-full h-full object-cover"
                                        />
                                    {:else if result.result_type === "album" && result.album?.cover_art}
                                        <img
                                            src={await getImageUrl(
                                                result.album.cover_art,
                                            )}
                                            alt={result.album.name}
                                            class="w-full h-full object-cover"
                                        />
                                    {:else if result.result_type === "artist" && result.artist?.profile_image}
                                        <img
                                            src={await getImageUrl(
                                                result.artist.profile_image,
                                                "artist",
                                            )}
                                            alt={result.artist.name}
                                            class="w-full h-full object-cover"
                                        />
                                    {:else}
                                        <Music
                                            size={20}
                                            class="text-gray-400"
                                        />
                                    {/if}
                                </div>
                                <div class="flex-1 min-w-0">
                                    {#if result.result_type === "track" && result.track}
                                        <p
                                            class="text-sm font-semibold truncate text-white"
                                        >
                                            {result.track.title}
                                        </p>
                                        <p
                                            class="text-xs text-gray-300 truncate"
                                        >
                                            {result.track.artists
                                                .map((a) => a.name)
                                                .join(", ") || "Unknown Artist"}
                                        </p>
                                    {:else if result.result_type === "artist" && result.artist}
                                        <p
                                            class="text-sm font-semibold truncate text-white"
                                        >
                                            {result.artist.name}
                                        </p>
                                        <p class="text-xs text-gray-300">
                                            Artist
                                        </p>
                                    {:else if result.result_type === "album" && result.album}
                                        <p
                                            class="text-sm font-semibold truncate text-white"
                                        >
                                            {result.album.name}
                                        </p>
                                        <p class="text-xs text-gray-300">
                                            Album
                                            {#if result.album.year}
                                                &middot; {result.album.year}
                                            {/if}
                                        </p>
                                    {:else if result.result_type === "playlist" && result.playlist}
                                        <p
                                            class="text-sm font-semibold truncate text-white"
                                        >
                                            {result.playlist.name}
                                        </p>
                                        <p class="text-xs text-gray-300">
                                            Playlist
                                        </p>
                                    {/if}
                                </div>
                            </button>
                        {/each}
                    {/if}
                </div>
            {/if}
        </div>

        <div
            class="controls flex align-top bg-card/30 backdrop-blur-lg shadow-md rounded-full border p-1"
        >
            <button id="titlebar-minimize" title="Minimize" onclick={minimize}>
                <Minus size={14} />
            </button>

            <button
                id="titlebar-maximize"
                title={isMaximized ? "Restore Down" : "Maximize"}
                onclick={toggleMaximize}
            >
                {#if isMaximized}
                    <Minimize2 size={14} />
                {:else}
                    <Maximize2 size={14} />
                {/if}
            </button>

            <button
                id="titlebar-close"
                title="Close"
                onclick={close}
                class="close-btn"
            >
                <X size={14} />
            </button>
        </div>
    </div>
</header>

<style>
    .controls button {
        appearance: none;
        padding: 0;
        margin: 0 2px;
        border: none;
        display: inline-flex;
        justify-content: center;
        align-items: center;
        width: 28px;
        height: 28px;
        color: white;
        background-color: transparent;
        cursor: pointer;
        border-radius: 12px;
        transition: background-color 0.15s ease;
    }

    .controls button:hover {
        background-color: rgba(255, 255, 255, 0.1);
    }

    .controls button.close-btn:hover {
        background-color: rgba(225, 0, 0, 0.9); /* Standard close red accent */
    }
</style>
