<script lang="ts">
    import type { Genre } from "$lib/types";
    import { Play, Music } from "@lucide/svelte";
    import { store } from "$lib/stores.svelte";

    let { data }: { data: Genre } = $props();
    let trackCount = $derived(store.tracksByGenre(data.id).length);
    let thumbnailSrc = $derived(data.thumbnail ? store.getImageSrc(data.thumbnail) : null);
</script>

<a
    href="/library/genres/{data.id}"
    class="group flex flex-col gap-3 p-6 rounded-3xl bg-card/80 transition-all duration-300 ring-2 ring-zinc-800/70 hover:ring-3 w-64 h-auto shadow-xl hover:shadow-card"
>
    <div class="aspect-square w-full rounded-3xl overflow-hidden relative inset-shadow-sm bg-muted flex items-center justify-center">
        {#if thumbnailSrc}
            <img
                src={thumbnailSrc}
                alt={data.name}
                class="w-full h-full object-cover"
            />
        {:else}
            <Music size={48} class="text-muted-foreground opacity-40" />
        {/if}

        <div class="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
            <div class="bg-accent text-black p-4 rounded-full shadow-xl transform translate-y-4 group-hover:translate-y-0 transition-transform">
                <Play size={20} fill="var(--color-accent-foreground)" />
            </div>
        </div>
    </div>

    <div class="flex flex-col mt-2">
        <h3 class="font-bold font-satoshi truncate text-white text-lg">{data.name}</h3>
        <p class="text-sm text-gray-400">{trackCount} track{trackCount !== 1 ? "s" : ""}</p>
    </div>
</a>
