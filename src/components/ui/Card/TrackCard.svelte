<script lang="ts">
    import type { Track } from "$lib/types";
    import { getImageUrl } from "$lib/utils";
    import { Play } from "@lucide/svelte";
    import { player } from "$lib/player.svelte";
    import { fade, fly } from "svelte/transition";
    import { bounceInOut } from "svelte/easing";
    import { Music2 } from "@lucide/svelte";
    import { store } from "$lib/stores.svelte";

    let { data }: { data: Track } = $props();

    let hovering = $state(false);
</script>

<div
    class="relative overflow-hidden rounded-4xl bg-secondary  shadow-lg hover:shadow-xl transition-shadow duration-300 h-auto min-w-60 w-60"
    onmouseenter={() => (hovering = true)}
    onmouseleave={() => (hovering = false)}
    role="feed"
>
    {#if data.cover_art}
        <img
            src={store.getTrackCoverUrl(data) ?? await getImageUrl(data.cover_art)}
            alt={data.title}
            class="w-full h-full object-cover {hovering ? 'scale-105' : ''} transition-transform"
        />
        <div
            class="absolute bottom-0 inset-x-0 bg-linear-to-t/hsl from-gray-950/80 from-50% to-gray-950/5 p-2 bg-blend-color"
        >
            <h4 class="font-semibold drop-shadow-md text-center">
                {data.title}
            </h4>
        </div>
    {:else}
        <div class="w-full h-full min-h-60 flex items-center justify-center">
            <Music2 size={54} class="text-gray-400" strokeWidth={3} />
        </div>
        <div
            class="absolute bottom-4 inset-x-0"
        >
            <h4 class="font-semibold text-center text-lg">
                {data.title}
            </h4>
        </div>
    {/if}

    {#if hovering}
        <div
            class="h-full w-full inset-0 absolute flex align-middle justify-center bg-neutral-800/30"
            transition:fade={{ duration: 100 }}
        >
            <button
                transition:fly={{ easing: bounceInOut }}
                class="m-auto bg-gray-200 rounded-full p-4 text-black"
                onclick={() => player.play([data], { type: "Direct" }, 0)}
            >
                <Play size={26} fill="black" />
            </button>
        </div>
    {/if}
</div>
