<script lang="ts">
    import { User } from "@lucide/svelte";
    import { getImageUrl } from "$lib/utils";
    import type { Artist } from "$lib/types";

    let { data }: { data: Artist } = $props();
</script>

<a
    href="/library/artists/{data.id}"
    class="group flex flex-col items-center text-center gap-4 px-5 py-3 rounded-4xl hover:bg-zinc-800/20 transition-all duration-300 border border-transparent hover:border-zinc-700/40 hover:shadow-xl"
>
    <div
        class="w-50 h-50 rounded-full overflow-hidden bg-zinc-800 shadow-xl relative"
    >
        {#if data.profile_image}
            {#await getImageUrl(data.profile_image, "artist")}
                <div class="w-full h-full bg-zinc-800 animate-pulse"></div>
            {:then url}
                <img
                    src={url}
                    alt={data.name}
                    class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
                />
            {/await}
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
