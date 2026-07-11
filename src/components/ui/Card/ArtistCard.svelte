<script lang="ts">
    import { User } from "@lucide/svelte";
    import { store } from "$lib/stores.svelte";
    import type { Artist } from "$lib/types";

    let { data }: { data: Artist } = $props();
</script>

<a
    href="/library/artists/{data.id}"
    class="group flex flex-col items-center text-center gap-4 px-5 py-3 "
>
    <div
        class="w-50 h-50 rounded-full overflow-hidden bg-zinc-800 shadow-xl relative"
    >
        {#if data.profile_image}
                <img
                    src={store.getImageSrc(data.profile_image, "artist")}
                    alt={data.name}
                    class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
                />
        {:else}
            <div class="w-50 h-50 flex items-center justify-center">
                <User size={48} class="text-zinc-700" />
            </div>
        {/if}

        <div
            class="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center"
        >
            <div
                class="bg-accent text-black p-3 rounded-full shadow-xl transform translate-y-4 group-hover:translate-y-0 transition-transform"
            >
                <User size={24} />
            </div>
        </div>
    </div>

    <h3 class="font-semibold truncate text-white">{data.name}</h3>
</a>
