<script lang="ts">
    import { goto } from "$app/navigation";
    import { player } from "$lib/player.svelte";
    import { store } from "$lib/stores.svelte";
    import type { Album, Artist, Playlist, Track } from "$lib/types";
    import { Music, Search, X } from "@lucide/svelte";
    import Fuse from "fuse.js";
    import { cubicOut } from "svelte/easing";
    import { slide } from "svelte/transition";
    import { flip } from "svelte/animate";

    type SearchableItem =
        | (Track & { type: "track" })
        | (Artist & { type: "artist" })
        | (Album & { type: "album" })
        | (Playlist & { type: "playlist" });

    type FilterType = "track" | "artist" | "album" | "playlist" | null;

    const SLASH_COMMANDS: {
        cmd: string;
        filter: FilterType;
    }[] = [
        { cmd: "/tracks", filter: "track" },
        { cmd: "/artists", filter: "artist" },
        { cmd: "/albums", filter: "album" },
        { cmd: "/playlists", filter: "playlist" },
    ];

    let searchQuery = $state("");
    let results = $state<SearchableItem[]>([]);
    let showResults = $state(false);
    let searchTimeout: any;
    let selectedIndex = $state(0);
    let resultsContainer: HTMLDivElement | undefined = $state();

    let ghostSuggestion = $state("");

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
                { name: "title", weight: 2 },
                { name: "name", weight: 2 },
                { name: "artists.name", weight: 0.8 },
                { name: "album.name", weight: 0.8 },
                { name: "album_artist.name", weight: 0.5 },
            ],
            threshold: 0.3,
            useExtendedSearch: true,
        });
    });

    $effect(() => {
        if (showResults && results.length > 0 && resultsContainer) {
            const child = resultsContainer.children[selectedIndex] as
                HTMLElement | undefined;
            child?.scrollIntoView({ block: "nearest" });
        }
    });

    function parseSlashCommand(raw: string): {
        filter: FilterType;
        query: string;
    } {
        if (!raw.startsWith("/")) return { filter: null, query: raw };

        const parts = raw.split(/\s+/);
        const token = parts[0].toLowerCase();
        const match = SLASH_COMMANDS.find((c) => c.cmd === token);

        if (match) {
            const rest = parts.slice(1).join(" ");
            return { filter: match.filter, query: rest };
        }

        return { filter: null, query: "" };
    }

    function resolveGhostSuggestion(raw: string): string {
        if (!raw.startsWith("/")) return "";
        if (raw.includes(" ")) return "";
        const lower = raw.toLowerCase();
        const match = SLASH_COMMANDS.find(
            (c) => c.cmd.startsWith(lower) && c.cmd !== lower,
        );
        return match ? match.cmd.slice(raw.length) : "";
    }

    let displayParts = $derived.by(() => {
        if (!searchQuery.startsWith("/")) return null;
        const parts = searchQuery.split(/\s+/);
        const token = parts[0].toLowerCase();
        const match = SLASH_COMMANDS.find((c) => c.cmd === token);
        if (!match) return null;
        return {
            command: parts[0],
            rest: parts.slice(1).join(" "),
        };
    });

    function closeDropdown() {
        results = [];
        showResults = false;
        searchQuery = "";
        ghostSuggestion = "";
    }

    function handleSearch() {
        if (searchTimeout) clearTimeout(searchTimeout);

        ghostSuggestion = resolveGhostSuggestion(searchQuery);

        const { filter, query } = parseSlashCommand(searchQuery);

        const trimmed = query.trim();

        if (searchQuery.startsWith("/") && !searchQuery.includes(" ")) {
            results = [];
            showResults = false;
            selectedIndex = 0;
            return;
        }

        if (trimmed.length < 1) {
            results = [];
            showResults = false;
            return;
        }

        selectedIndex = 0;

        searchTimeout = setTimeout(() => {
            let raw = fuse.search(trimmed).map((r) => r.item);
            if (filter) {
                raw = raw.filter((item) => item.type === filter);
            }
            results = raw;
            showResults = true;
        }, 300);
    }

    function handleKeyDown(e: KeyboardEvent) {
        if (e.key === "Tab" && ghostSuggestion) {
            e.preventDefault();
            searchQuery = searchQuery + ghostSuggestion + " ";
            ghostSuggestion = "";

            return;
        }

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

<div
    class="flex flex-col bg-linear-to-br from-white/10 to-white/3 backdrop-blur-2xl backdrop-brightness-80 backdrop-saturate-150 ring-1 ring-white/10 hover:ring-2 shadow-[0_8px_32px_0_rgba(0,0,0,0.25)] mt-auto w-80 focus-within:w-120 {showResults
        ? 'rounded-4xl'
        : 'rounded-full transition-all'} "
>
    <div class=" flex items-center gap-2 px-4 py-1">
        <Search size={18} class="text-gray-300 shrink-0" />

        <div class="relative flex-1 min-w-0 flex items-center">
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
                class="w-full py-4 outline-none bg-transparent text-transparent caret-white placeholder-muted-foreground placeholder:drop-shadow-lg text-sm"
                autocomplete="off"
                spellcheck="false"
            />

            {#if searchQuery}
                <span
                    class="pointer-events-none absolute left-0 top-1/2 -translate-y-1/2 text-sm select-none whitespace-pre"
                    aria-hidden="true"
                >
                    {#if displayParts}
                        <span class="text-violet-400"
                            >{displayParts.command}</span
                        ><span class="text-white/80"
                            >{displayParts.rest
                                ? " " + displayParts.rest
                                : ""}</span
                        >
                    {:else}
                        <span class="text-white/80">{searchQuery}</span>
                    {/if}
                    {#if ghostSuggestion}
                        <span class="text-gray-500">{ghostSuggestion}</span>
                    {/if}
                </span>
            {/if}
        </div>

        {#if ghostSuggestion}
            <span
                class="shrink-0 text-[10px] text-gray-500 border border-gray-700 rounded px-1 py-0.5 font-mono select-none"
            >
                Tab
            </span>
        {/if}

        {#if searchQuery}
            <button onclick={() => (searchQuery = "")}>
                <X size={14} class="text-gray-400 hover:text-white" />
            </button>
        {/if}
    </div>

    {#if showResults}
        <div
            class="overflow-hidden max-h-[60vh] h-auto overflow-y-auto p-2"
            bind:this={resultsContainer}
            onblur={() => (showResults = false)}
            role="listbox"
            in:slide
        >
            {#if results.length === 0}
                <div class="px-4 py-3 text-sm text-gray-400 text-center">
                    No results found
                </div>
            {:else}
                {#each results.slice(0, 15) as result, i (result.type + "-" + result.id)}
                    <button
                        class="w-full flex items-center gap-4 px-2 py-2 mb-1 transition-colors group text-left rounded-xl {i ===
                        selectedIndex
                            ? 'bg-white/5 shadow-md'
                            : 'hover:bg-white/5'}"
                        onmousedown={() => handleResultClick(result)}
                        role="option"
                        aria-selected={i === selectedIndex}
                        animate:flip={{ duration: 200, easing: cubicOut }}
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
