<script lang="ts">
    import { Play } from "@lucide/svelte";
    import { store } from "$lib/stores.svelte";
    import type { Album } from "$lib/types";

    let { data }: { data: Album } = $props();
</script>

<a
    href="/library/albums/{data.id}"
    class="group flex flex-col gap-3 p-4 rounded-2xl hover:bg-zinc-950/20 transition-all duration-300 border border-transparent hover:border-zinc-800 h-auto min-w-64 w-64 group-hover:shadow-xl"
>
    <div class="aspect-square w-full overflow-hidden rounded-2xl relative">
        {#if data.cover_art}
            <img
                src={store.getImageSrc(data.cover_art)}
                alt={data.name}
                class="w-full h-full group-hover:scale-105 transition-transform duration-500"
            />
        {:else}
            <img
                src="/PhonographRecord.webp"
                alt="Album Icon"
                class="w-full h-full"
            />
        {/if}

        <div
            class="absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center"
        >
            <div
                class="bg-accent text-black p-4 rounded-full shadow-xl transform translate-y-4 group-hover:translate-y-0 transition-transform"
            >
                <Play size={28} fill="black" />
            </div>
        </div>
    </div>

    <div class="flex flex-col ml-1.5">
        <h3 class="font-semibold truncate text-white">{data.name}</h3>
        <div class="flex text-sm text-gray-300 gap-2 font-mono mt-1 px-0.5">
            <p>Album</p>
            {#if data.year}
                <p>•{data.year}</p>
            {/if}
        </div>
    </div>
</a>
