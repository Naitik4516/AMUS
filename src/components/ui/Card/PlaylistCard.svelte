<script lang="ts">
    import { Music, Play } from "@lucide/svelte";
    import { store } from "$lib/stores.svelte";

    let { data } = $props();
</script>

<a
    href="/library/playlists/{data.id}?data.name={data.name}"
    class="group flex flex-col gap-3 p-5 rounded-3xl bg-card/60 transition-all duration-300 border-2 border-transparent hover:border-border/80 w-60 h-70 shadow-xl"
>
    <!-- Cover Art Grid -->
    <div
        class="aspect-square w-full rounded-2xl overflow-hidden bg-white/5 relative"
    >
        {#if !data.coverArts || data.coverArts.length === 0}
            <div class="absolute inset-0 flex items-center justify-center">
                <Music size={48} class="text-gray-400" />
            </div>
        {:else if data.coverArts.length < 4}
            {#if store.getImageSrc(data.coverArts[0])}
                <img
                    src={store.getImageSrc(data.coverArts[0])}
                    alt={data.name}
                    class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
                />
            {/if}
        {:else}
            <div
                class="grid grid-cols-2 grid-rows-2 w-full h-full group-hover:scale-105 transition-transform duration-500"
            >
                {#each data.coverArts.slice(0, 4) as art}
                    {#if store.getImageSrc(art)}
                        <img
                            src={store.getImageSrc(art)}
                            alt=""
                            class="w-full h-full object-cover"
                        />
                    {:else}
                        <div
                            class="w-full h-full bg-neutral-800 animate-pulse"
                        ></div>
                    {/if}
                {/each}
            </div>
        {/if}

        <!-- Play Button Overlay -->
        <div
            class="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center"
        >
            <div
                class="bg-accent text-black p-4 rounded-full shadow-xl transform translate-y-4 group-hover:translate-y-0 transition-transform"
            >
                <Play size={20} fill="var(--color-accent-foreground)" />
            </div>
        </div>
    </div>

    <!-- Metadata -->
    <div class="flex flex-col">
        <h3 class="font-bold truncate text-white">{data.name}</h3>
        <p class="text-sm text-gray-400">Playlist</p>
    </div>
</a>
