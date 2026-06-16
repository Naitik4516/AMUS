<script lang="ts">
    import { onMount } from "svelte";
    import {
        getSourceDirs,
        removeSource,
        importAudioLibrary,
    } from "$lib/commands.svelte";
    import { FolderOpen, Trash2, RefreshCw, Plus } from "@lucide/svelte";

    let sources = $state<string[]>([]);
    let loading = $state(true);

    async function loadSources() {
        loading = true;
        try {
            sources = await getSourceDirs();
        } catch (e) {
            console.error("Failed to load sources", e);
        }
        loading = false;
    }

    async function handleRemoveSource(path: string) {
        try {
            await removeSource(path);
            sources = sources.filter((s) => s !== path);
        } catch (e) {
            console.error("Failed to remove source", e);
        }
    }

    async function handleAddSource() {
        await importAudioLibrary();
        await loadSources();
    }

    onMount(loadSources);
</script>

<div class="p-8">
    <div class="flex items-center justify-between mb-8">
        <h1 class="text-3xl font-black text-white">Library Sources</h1>
        <button
            onclick={handleAddSource}
            class="flex items-center gap-2 px-4 py-2 bg-secondary text-black font-bold rounded-full hover:scale-105 transition-transform"
        >
            <Plus size={20} /> Add Folder
        </button>
    </div>

    <p class="text-gray-400 mb-6">
        Manage the folders that Amus scans for audio files.
    </p>

    {#if loading}
        <div class="flex items-center justify-center py-20">
            <div
                class="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-secondary"
            ></div>
        </div>
    {:else if sources.length === 0}
        <div
            class="flex flex-col items-center justify-center py-20 text-gray-500"
        >
            <FolderOpen size={64} class="mb-4 opacity-20" />
            <p class="text-xl font-medium">No sources configured</p>
            <p class="text-sm mb-6">Add a folder to start scanning your music.</p>
            <button
                onclick={handleAddSource}
                class="flex items-center gap-2 px-6 py-3 bg-secondary text-black font-bold rounded-full hover:scale-105 transition-transform"
            >
                <FolderOpen size={20} /> Select Folder
            </button>
        </div>
    {:else}
        <div class="space-y-3">
            {#each sources as source}
                <div
                    class="flex items-center justify-between bg-neutral-900/80 border border-neutral-800 rounded-xl px-5 py-4 group hover:border-neutral-700 transition-colors"
                >
                    <div class="flex items-center gap-4 min-w-0">
                        <FolderOpen size={20} class="text-secondary shrink-0" />
                        <span class="text-white truncate" title={source}>
                            {source}
                        </span>
                    </div>
                    <button
                        onclick={() => handleRemoveSource(source)}
                        class="shrink-0 p-2 text-gray-500 hover:text-red-400 hover:bg-red-400/10 rounded-lg transition-colors"
                        title="Remove source"
                    >
                        <Trash2 size={18} />
                    </button>
                </div>
            {/each}
        </div>

        <div class="mt-8 flex items-center gap-4">
            <button
                onclick={handleAddSource}
                class="flex items-center gap-2 px-4 py-2 bg-neutral-800 text-white font-bold rounded-full hover:bg-neutral-700 transition-colors"
            >
                <Plus size={18} /> Add Folder
            </button>
            <button
                onclick={() => importAudioLibrary().then(loadSources)}
                class="flex items-center gap-2 px-4 py-2 bg-neutral-800 text-white font-bold rounded-full hover:bg-neutral-700 transition-colors"
            >
                <RefreshCw size={18} /> Rescan All
            </button>
        </div>
    {/if}
</div>
