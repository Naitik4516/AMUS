<script lang="ts">
    import type { Track } from "$lib/types";
    import { getImageUrl } from "$lib/utils";
    import { Play } from "@lucide/svelte";
    import { player } from "$lib/player.svelte";
    import { fade, fly } from "svelte/transition";
    import { bounceInOut } from "svelte/easing";
    let { data }: { data: Track} = $props();

    let hovering = $state(false);
</script>

<div
    class="relative overflow-hidden rounded-2xl bg-gray-900 shadow-lg hover:shadow-xl transition-shadow duration-300 h-auto min-w-60 w-60 "
    onmouseenter={() => (hovering = true)}
    onmouseleave={() => (hovering = false)}
    role="feed"
>
    <img
        src={data.cover_art
            ? await getImageUrl(data.cover_art)
            : "/PhonographRecord.png"}
        alt={data.title}
        class="w-full h-full object-cover"
        width="240"
    />
    <div
        class="absolute bottom-0 inset-x-0 bg-linear-to-t/hsl from-gray-950/80 from-50% to-gray-950/5 p-2 bg-blend-color"
    >
        <h3 class="font-semibold drop-shadow-md">
            {data.title}
        </h3>
    </div>
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
