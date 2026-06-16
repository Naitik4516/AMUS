<script lang="ts">
    import { User } from "@lucide/svelte";
    import { getArtistPicUrl } from "$lib/utils";

    let { id, name, profile_picture } = $props<{
        id: number;
        name: string;
        profile_picture?: string | null;
    }>();
</script>

<a
    href="/library/artists/{id}"
    class="group flex flex-col items-center text-center gap-3 p-4 rounded-2xl bg-dark-alt hover:bg-neutral-800 transition-all duration-300 border border-transparent hover:border-neutral-700 w-50 h-60 shadow-xl"
>
    <div
        class="aspect-square w-full rounded-full overflow-hidden bg-neutral-800 shadow-xl relative"
    >
        {#if profile_picture}
            {#await getArtistPicUrl(profile_picture)}
                <div class="w-full h-full bg-neutral-800 animate-pulse"></div>
            {:then url}
                <img
                    src={url}
                    alt={name}
                    class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
                />
            {/await}
        {:else}
            <div class="w-full h-full flex items-center justify-center">
                <User size={48} class="text-neutral-700" />
            </div>
        {/if}

        <div
            class="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center"
        >
            <div
                class="bg-secondary text-black p-3 rounded-full shadow-xl transform translate-y-4 group-hover:translate-y-0 transition-transform"
            >
                <User size={24} />
            </div>
        </div>
    </div>

    <div class="flex flex-col">
        <h3 class="font-bold truncate text-white">{name}</h3>
        <p class="text-sm text-gray-400">Artist</p>
    </div>
</a>
