<script lang="ts">
    import { Play } from "@lucide/svelte";
    import { getImageUrl } from "$lib/utils";
    import type { Album } from "$lib/types";

    let { data }: { data: Album } = $props();
</script>

<a
    href="/library/albums/{data.id}?album.name={data.name}"
    class="group flex flex-col gap-3 p-4 rounded-2xl hover:bg-secondary transition-all duration-300 border border-transparent hover:border-neutral-700 w-auto h-auto max-h-72 shadow-xl"
>
    <div
        class="aspect-square w-full rounded-3xl overflow-hidden bg-card border-border shadow-lg relative"
    >
        {#if data.cover_art}
            {#await getImageUrl(data.cover_art)}
                <div
                    class="absolute inset-0 bg-card animate-pulse"
                ></div>
            {:then url}
                <img
                    src={url}
                    alt={data.name}
                    class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
                />
            {/await}
        {:else}
            <div class="absolute inset-0 flex items-center justify-center p-5">
                <img src="/PhonographRecord.png" alt="Album Icon" />
            </div>
        {/if}

        <div
            class="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center"
        >
            <div
                class="bg-accent text-black p-4 rounded-full shadow-xl transform translate-y-4 group-hover:translate-y-0 transition-transform"
            >
                <Play size={28} fill="black"  />
            </div>
        </div>
    </div>

    <div class="flex flex-col">
        <h3 class="font-bold truncate text-white">{data.name}</h3>
        <p class="text-sm text-gray-400">Album</p>
    </div>
</a>
