<script lang="ts">
    import type { Track } from "$lib/types";
    import { Play } from "@lucide/svelte";
    import { player } from "$lib/player.svelte";
    import { fade, fly } from "svelte/transition";
    import TrackCoverArt from "../TrackCoverArt.svelte";

    let { data }: { data: Track } = $props();

    let hovering = $state(false);
</script>

<div
    class="relative overflow-hidden rounded-4xl bg-secondary shadow-lg hover:shadow-xl transition-shadow duration-300 h-64 w-64"
    onmouseenter={() => (hovering = true)}
    onmouseleave={() => (hovering = false)}
    role="feed"
>
    <TrackCoverArt cover_art={data.cover_art} {hovering} />

    <div
        class="absolute bottom-0 inset-x-0 bg-linear-to-t/hsl from-black/60 to-transparent h-20"
    ></div>
    <h4
        class="absolute bottom-3 font-bold text-[18px] drop-shadow-md drop-shadow-black text-center px-3 capitalize inset-x-0 truncate"
    >
        {data.title}
    </h4>

    {#if hovering}
        <div
            class="inset-0 absolute flex align-middle justify-center bg-black/40"
            transition:fade={{ duration: 150 }}
        >
            <button
                transition:fly={{ delay: 100, duration: 300, y: 20 }}
                class="m-auto bg-gray-200 hover:bg-gray-300 transition-colors rounded-full p-4 text-black"
                onclick={() => player.play([data], { type: "Direct" }, 0)}
            >
                <Play size={26} fill="black" />
            </button>
        </div>
    {/if}
</div>
