<script lang="ts">
    import { Music } from "@lucide/svelte";
    import { getCoverUrl } from "$lib/utils";
    import type { Album } from "$lib/types";

    let { albums }: { albums: Album[] } = $props();
</script>

<div class="flex gap-6 overflow-x-auto pb-4 scrollbar-hide">
    {#each albums as album}
        <a
            href="/library/albums/{album.id}?name={album.name}"
            class="flex-shrink-0 w-48 group"
        >
            <div
                class="aspect-square rounded-xl bg-neutral-800 shadow-lg overflow-hidden mb-3 relative"
            >
                {#if album.cover_art}
                    {#await getCoverUrl(album.cover_art)}
                        <div class="w-full h-full bg-neutral-800 animate-pulse"></div>
                    {:then url}
                        <img
                            src={url}
                            alt={album.name}
                            class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
                        />
                    {/await}
                {:else}
                    <div class="w-full h-full flex items-center justify-center">
                        <Music size={48} class="text-neutral-700" />
                    </div>
                {/if}
                <div
                    class="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center"
                >
                    <div class="bg-secondary text-black p-3 rounded-full shadow-xl">
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z" /></svg>
                    </div>
                </div>
            </div>
            <h4 class="font-bold text-white truncate">{album.name}</h4>
            <p class="text-sm text-gray-400">Album</p>
        </a>
    {/each}
</div>

<style>
    .scrollbar-hide::-webkit-scrollbar {
        display: none;
    }
    .scrollbar-hide {
        -ms-overflow-style: none;
        scrollbar-width: none;
    }
</style>
