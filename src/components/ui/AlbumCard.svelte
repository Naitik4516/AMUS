<script lang="ts">
    import { Music } from "@lucide/svelte";
    import { getCoverUrl } from "$lib/utils";

    let { id, name, cover_art } = $props<{
        id: number;
        name: string;
        cover_art?: string | null;
    }>();
</script>

<a
    href="/library/albums/{id}?name={name}"
    class="group flex flex-col gap-3 p-4 rounded-2xl bg-dark-alt hover:bg-neutral-800 transition-all duration-300 border border-transparent hover:border-neutral-700 w-50 h-60 shadow-xl"
>
    <div
        class="aspect-square w-full rounded-2xl overflow-hidden bg-neutral-800 shadow-lg relative"
    >
        {#if cover_art}
            {#await getCoverUrl(cover_art)}
                <div class="absolute inset-0 bg-neutral-800 animate-pulse"></div>
            {:then url}
                <img
                    src={url}
                    alt={name}
                    class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
                />
            {/await}
        {:else}
            <div class="absolute inset-0 flex items-center justify-center">
                <Music size={48} class="text-neutral-700" />
            </div>
        {/if}

        <div
            class="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center"
        >
            <div
                class="bg-secondary text-black p-3 rounded-full shadow-xl transform translate-y-4 group-hover:translate-y-0 transition-transform"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="24"
                    height="24"
                    viewBox="0 0 24 24"
                    fill="currentColor"><path d="M8 5v14l11-7z" /></svg
                >
            </div>
        </div>
    </div>

    <div class="flex flex-col">
        <h3 class="font-bold truncate text-white">{name}</h3>
        <p class="text-sm text-gray-400">Album</p>
    </div>
</a>
