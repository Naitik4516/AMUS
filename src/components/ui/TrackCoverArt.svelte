<script lang="ts">
    import { Music2 } from "@lucide/svelte";
    import { store } from "$lib/stores.svelte";

    let {
        cover_art,
        hovering,
        class: className = "",
        ...props
    }: {
        cover_art?: string;
        hovering?: boolean;
        class?: string;
    } = $props();
    let loading = $state(true);
</script>

<div class="relative w-full h-full overflow-hidden {className}">
    {#if cover_art}
        <img
            src={store.getImageSrc(cover_art)}
            alt="Track cover art"
            class="w-full h-full object-cover hover:scale-105 transition-transform duration-400"
            class:scale-105={hovering}
            class:hidden={loading}
            onload={() => (loading = false)}
            onerror={() => (loading = false)}
            {...props}
        />
    {/if}

    {#if !cover_art || loading}
        <div
            class="w-full h-full bg-gray-300/5 flex items-center justify-center align-middle"
        >
            <Music2 class="text-gray-400 size-2/5" strokeWidth={3} />
        </div>
    {/if}
</div>
