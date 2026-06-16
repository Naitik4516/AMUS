<script lang="ts">
    import {
        X,
        Minus,
        Square,
        Search,
        Music,
        User,
        Disc,
    } from "@lucide/svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { invoke } from "@tauri-apps/api/core";
    import { player } from "$lib/player.svelte";
    import type { Track } from "$lib/types";
    import { goto } from "$app/navigation";

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

    function playTrack(track: Track) {
        player.play(track);
        showResults = false;
        searchQuery = "";
    }
</script>

<header
    data-tauri-drag-region
    class="h-16 flex items-center justify-between p-3 select-none text-white relative z-20"
>
    <div class="w-16 flex justify-center shrink-0 mt-1 pointer-events-none">
        <img
            src="/favicon.png?"
            alt="Avatar"
            class="w-10 h-10 object-contain select-none"
        />
    </div>

    <div class="relative">
        <div
            class="flex items-center gap-2 bg-stone-800/60 backdrop-blur-sm rounded-full px-4 mt-1 w-96 h-full border-2 border-transparent transition-colors duration-300 hover:border-neutral-600 focus-within:border-secondary/50"
        >
            <Search size={16} class="text-gray-400" />
            <input
                type="text"
                placeholder="What do you want to listen to?"
                bind:value={searchQuery}
                oninput={handleSearch}
                onfocus={() => searchQuery.trim() && (showResults = true)}
                class="w-full py-4 outline-none bg-transparent text-white placeholder-gray-400 text-sm"
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
                class="fixed inset-0"
                onclick={() => (showResults = false)}
            ></div>
            <div
                class="absolute top-full left-0 right-0 mt-2 bg-neutral-900 border border-neutral-800 rounded-2xl shadow-2xl overflow-hidden py-2 z-50"
            >
                {#each results as track}
                    <button
                        class="w-full flex items-center gap-4 px-4 py-3 hover:bg-neutral-800 transition-colors group text-left"
                        onclick={() => playTrack(track)}
                    >
                        <div
                            class="w-10 h-10 rounded bg-neutral-800 flex items-center justify-center overflow-hidden"
                        >
                            <Music size={20} class="text-neutral-700" />
                        </div>
                        <div class="flex-1 min-w-0">
                            <p
                                class="text-sm font-bold truncate text-white group-hover:text-secondary"
                            >
                                {track.title}
                            </p>
                            <p class="text-xs text-gray-400 truncate">
                                {track.artist.map((a) => a.name).join(", ") || "Unknown Artist"}
                            </p>
                        </div>
                    </button>
                {/each}
            </div>
        {/if}
    </div>

    <div class="controls flex items-center">
        <button id="titlebar-minimize" title="Minimize" onclick={minimize}>
            <Minus size={14} />
        </button>

        <button
            id="titlebar-maximize"
            title={isMaximized ? "Restore Down" : "Maximize"}
            onclick={toggleMaximize}
        >
            {#if isMaximized}
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class="icon icon-tabler icons-tabler-outline icon-tabler-squares"
                >
                    <path stroke="none" d="M0 0h24v24H0z" fill="none" />
                    <path
                        d="M8 10a2 2 0 0 1 2 -2h9a2 2 0 0 1 2 2v9a2 2 0 0 1 -2 2h-9a2 2 0 0 1 -2 -2l0 -9"
                    />
                    <path
                        d="M16 8v-3a2 2 0 0 0 -2 -2h-9a2 2 0 0 0 -2 2v9a2 2 0 0 0 2 2h3"
                    />
                </svg>
            {:else}
                <Square size={13} />
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
