<script lang="ts">
    import { Volume, Volume1, Volume2, VolumeX } from "@lucide/svelte";
    import Slider from "./ui/Slider.svelte";
    import { player } from "$lib/player.svelte";

    let { width = "w-24", iconSize = 20 } = $props();
</script>

<div
    class="flex items-center gap-2"
    onwheel={(e) => {
        e.preventDefault();
        const delta = e.deltaY > 0 ? -0.05 : 0.05;
        player.setVolume(Math.max(0, Math.min(1, player.volume + delta)));
    }}
>
    <button
        class="text-gray-300 hover:text-white transition-colors"
        onclick={() => player.toggleMute()}
        aria-label="Toggle mute"
    >
        {#if player.volume === 0}
            <VolumeX size={iconSize} />
        {:else if player.volume < 0.33}
            <Volume size={iconSize} />
        {:else if player.volume < 0.66}
            <Volume1 size={iconSize} />
        {:else}
            <Volume2 size={iconSize} />
        {/if}
    </button>
    <div class={width}>
        <Slider value={player.volume} onValueChange={(val) => player.setVolume(val)} />
    </div>
</div>
