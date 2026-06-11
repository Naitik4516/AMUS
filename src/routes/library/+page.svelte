<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { Play, Plus, RefreshCw } from "@lucide/svelte";
    import { open } from '@tauri-apps/plugin-dialog';

    import { player, type Track } from "../../lib/player.svelte";

    let tracks = $state<Track[]>([]);
    let loading = $state(true);

    async function loadTracks() {
        loading = true;
        try {
            tracks = await invoke("get_tracks");
        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    }

    async function scanLibrary() {
        try {
            await invoke("scan_library");
            await loadTracks();
        } catch (e) {
            console.error(e);
        }
    }

    async function playTrack(track: Track) {
        try {
            await player.play(track, tracks);
        } catch (e) {
            console.error(e);
        }
    }


    async function addSource() {
        // In a real app, use a file dialog. For now, prompt or hardcode.
        const path = await open({
          multiple: true,
          directory: true,
        });
        if (path) {
            try {
              for (const p of Array.isArray(path) ? path : [path]) {
                await invoke("add_source", { path: p });
              }
                await scanLibrary();
            } catch (e) {
                console.error(e);
            }
        }
    }

    onMount(loadTracks);
</script>

<div class="p-8 w-full overflow-y-auto h-[calc(100vh-120px)]">
    <div class="flex justify-between items-center mb-8">
        <h1 class="text-3xl font-bold">Your Library</h1>
        <div class="flex gap-4">
            <button
                class="flex items-center gap-2 bg-secondary text-gray-900 px-4 py-2 rounded-full font-semibold hover:bg-gray-200"
                onclick={addSource}
            >
                <Plus size={20} /> Add Source
            </button>
            <button
                class="flex items-center gap-2 bg-gray-800 text-white px-4 py-2 rounded-full font-semibold hover:bg-gray-700"
                onclick={scanLibrary}
            >
                <RefreshCw size={20} class={loading ? 'animate-spin' : ''} /> Scan
            </button>
        </div>
    </div>

    {#if loading && tracks.length === 0}
        <p class="text-gray-400">Loading tracks...</p>
    {:else if tracks.length === 0}
        <div class="text-center py-20">
            <p class="text-xl text-gray-400 mb-4">No tracks found.</p>
            <button class="text-secondary underline" onclick={addSource}>Add a source directory to get started.</button>
        </div>
    {:else}
        <div class="grid gap-2">
            <div class="grid grid-cols-[auto_1fr_1fr_1fr_auto] gap-4 px-4 py-2 text-gray-400 font-semibold border-b border-gray-800">
                <div class="w-10">#</div>
                <div>Title</div>
                <div>Artist</div>
                <div>Album</div>
                <div class="w-20 text-right">Duration</div>
            </div>
            {#each tracks as track, i}
                <div
                    class="grid grid-cols-[auto_1fr_1fr_1fr_auto] gap-4 px-4 py-3 items-center hover:bg-white/5 rounded-lg group cursor-pointer"
                    onclick={() => playTrack(track)}
                >
                    <div class="w-10 text-gray-500 group-hover:text-white">
                        <span class="group-hover:hidden">{i + 1}</span>
                        <Play size={16} class="hidden group-hover:block text-secondary" />
                    </div>
                    <div class="font-medium">{track.title}</div>
                    <div class="text-gray-400">{track.artist}</div>
                    <div class="text-gray-400">{track.album}</div>
                    <div class="w-20 text-right text-gray-500">
                        {Math.floor(track.duration_seconds / 60)}:{(track.duration_seconds % 60).toString().padStart(2, '0')}
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</div>
