<script lang="ts">
    import {
        X,
        Minus,
        Square,
        Search,
        Music,
        Maximize2,
        Minimize2,
    } from "@lucide/svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { invoke } from "@tauri-apps/api/core";
    import type { Track } from "$lib/types";
    import TrackListSmall from "./ui/TrackListSmall.svelte";

    let isMaximized = $state(false);
    let searchQuery = $state("");
    let results = $state<Track[]>([]);
    let showResults = $state(false);
    let searchTimeout: any;

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

    async function handleSearch() {
        if (searchTimeout) clearTimeout(searchTimeout);

        if (!searchQuery.trim()) {
            results = [];
            showResults = false;
            return;
        }

        searchTimeout = setTimeout(async () => {
            try {
                results = await invoke("search_tracks", {
                    query: searchQuery,
                    limit: 10,
                });
                showResults = true;
            } catch (e) {
                console.error("Search failed", e);
            }
        }, 300);
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

<header
    data-tauri-drag-region
    class="sticky top-0 flex items-center px-4 h-0 mt-10 justify-between select-none text-white z-20"
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
            class="flex items-center gap-2 bg-card/40 backdrop-blur-md rounded-full px-4 h-full border-2 transition-colors duration-300 hover:bg-card/75 focus-within:bg-card"
        >
            <Search size={16} class="text-gray-400" />
            <input
                type="text"
                placeholder="What do you want to listen to?"
                bind:value={searchQuery}
                oninput={handleSearch}
                onfocus={() => searchQuery.trim() && (showResults = true)}
                class="w-60 py-4 outline-none bg-transparent text-white placeholder-gray-400 text-sm focus:w-120 transition-all duration-300"
            />
            {#if searchQuery}
                <button
                    onclick={() => {
                        searchQuery = "";
                        results = [];
                        showResults = false;
                    }}
                >
                    <X size={14} class="text-gray-400 hover:text-white" />
                </button>
            {/if}
        </div>

        {#if showResults && results.length > 0}
            <div
                class="absolute top-full left-0 right-0 mt-2 bg-card/40 backdrop-blur-xl border-2 border-border/60 rounded-4xl shadow-2xl overflow-hidden py-1 z-50"
                onblur={() => (showResults = false)}

            >
                {#each results as track}
                    <TrackListSmall
                        {track}
                        onclick={() => {
                            searchQuery = "";
                            results = [];
                            showResults = false;
                        }}
                    />
                {/each}
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
