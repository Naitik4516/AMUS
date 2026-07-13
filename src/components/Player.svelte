<script lang="ts">
    import {
        Play,
        Pause,
        SkipBack,
        SkipForward,
        Shuffle,
        Repeat,
        Repeat1,
        Volume,
        Volume1,
        Volume2,
        VolumeX,
        ListMusic,
        Heart,
        Music2,
    } from "@lucide/svelte";
    import Slider from "./ui/Slider.svelte";
    import { player } from "$lib/player.svelte";
    import { formatDurationColon } from "$lib/utils";

    import { ui } from "$lib/shortcut-handler.svelte";
    import Marquee from "./ui/Marquee.svelte";
    import QueueView from "./QueueView.svelte";
    import { store } from "$lib/stores.svelte";

    let showQueue = $state(false);

    $effect(() => {
        if (ui.queueVisible) showQueue = true;
    });

    async function toggleFavorite() {
        if (player.currentTrack) {
            await player.toggleFavorite(player.currentTrack);
        }
    }
</script>

{#if player.currentTrack}
    <div class="fixed bottom-0 left-0 w-full px-4 pb-3 z-15">
        <div
            class="bg-zinc-950/50 border-2 border-neutral-800/40 backdrop-blur-xl grid grid-cols-3 items-center justify-between px-6 py-3 shadow-lg rounded-3xl relative"
            ondblclick={() => player.stop()}
            role="contentinfo"
        >
            <!-- Track Info -->
            <div class="flex items-center gap-4 pr-10">
                <div
                    class="w-14 h-14 rounded-lg bg-neutral-800 shadow-md flex items-center justify-center overflow-hidden shrink-0"
                >
                    {#if player.currentTrack?.cover_art}
                        <img
                            src={store.getImageSrc(player.currentTrack.cover_art)}
                            alt=""
                            class="w-full h-full object-cover"
                        />
                    {:else}
                        <Music2 size={32} />
                    {/if}
                </div>
                <div class="flex flex-col overflow-hidden">
                    <Marquee>
                        <a
                            href="/library/track/{player.currentTrack?.id}"
                            class="font-bold truncate text-white hover:underline cursor-pointer inline-block"
                        >
                            {player.currentTrack?.title}
                        </a>
                    </Marquee>
                    <Marquee>
                        <p class=" text-gray-300 truncate -mt-2">
                            {#each player.currentTrack?.artists as artist, ai (artist.id)}
                                {#if ai > 0}
                                    <span>, </span>
                                {/if}
                                <a
                                    href="/library/artists/{artist.id}"
                                    class=" hover:text-white font-medium text-xs"
                                    >{artist.name}</a
                                >
                            {/each}
                        </p>
                    </Marquee>
                </div>
                <button
                    onclick={toggleFavorite}
                    class="ml-2 {player.currentTrack?.is_favorite
                        ? 'text-rose-600 fill-rose-600'
                        : 'text-gray-300'}  hover:text-secondary transition-colors"
                >
                    <Heart
                        size={18}
                        class={player.currentTrack?.is_favorite
                            ? "text-rose-600 fill-rose-600"
                            : "text-gray-300"}
                    ></Heart>
                </button>
            </div>

            <!-- Controls -->
            <div class="flex flex-col items-center gap-2">
                <div class="flex items-center gap-6">
                    <button
                        class="hover:text-white transition-colors"
                        class:text-white={player.shuffleEnabled}
                        class:text-muted-foreground={!player.shuffleEnabled}
                        onclick={() => player.toggleShuffle()}
                    >
                        <Shuffle size={18} />
                    </button>
                    <button
                        class="text-gray-300 hover:text-white transition-colors"
                        onclick={() => player.previous()}
                    >
                        <SkipBack size={22} fill="currentColor" />
                    </button>
                    <button
                        class="bg-white text-black rounded-full p-3 hover:scale-105 transition-transform shadow-lg"
                        onclick={() => player.playPause()}
                    >
                        {#if player.isPlaying}
                            <Pause size={24} fill="currentColor" />
                        {:else}
                            <Play size={24} fill="currentColor" />
                        {/if}
                    </button>
                    <button
                        class="text-gray-200 hover:text-white transition-colors"
                        onclick={() => player.next()}
                    >
                        <SkipForward size={22} fill="currentColor" />
                    </button>
                    <button
                        class="hover:text-white transition-colors"
                        class:text-accent={player.repeatMode !== "OFF"}
                        class:text-muted-foreground={player.repeatMode ===
                            "OFF"}
                        onclick={() => player.cycleRepeat()}
                    >
                        {#if player.repeatMode === "ONE"}
                            <Repeat1 size={18} />
                        {:else}
                            <Repeat size={18} />
                        {/if}
                    </button>
                </div>
                <div class="w-full flex items-center justify-center gap-3">
                    <span
                        class="text-[10px] font-medium text-gray-400 w-10 text-right"
                    >
                        {formatDurationColon(player.position)}
                    </span>
                    <Slider
                        value={player.progress}
                        onValueChange={(val) => {
                            if (player.currentTrack) {
                                let seekVal =
                                    val * player.duration;

                                player.seek(seekVal);
                            }
                        }}
                    />
                    <span class="text-[10px] font-medium text-gray-400 w-10">
                        {formatDurationColon(
                            player.currentTrack.duration_seconds,
                        )}
                    </span>
                </div>
            </div>

            <!-- Volume & Queue -->
            <div class="flex items-center gap-4 justify-end">
                <button
                    onclick={() => (showQueue = !showQueue)}
                    class="text-gray-300 hover:text-white transition-colors"
                    class:text-accent={showQueue}
                >
                    <ListMusic size={20} />
                </button>

                <div
                    class="flex items-center gap-2 group"
                    onwheel={(e) => {
                        e.preventDefault();
                        const delta = e.deltaY > 0 ? -0.05 : 0.05;
                        player.setVolume(
                            Math.max(0, Math.min(1, player.volume + delta)),
                        );
                    }}
                >
                    <button
                        class="text-gray-300 group-hover:text-white transition-colors"
                        onclick={() =>
                            player.volume != 0
                                ? player.setVolume(0)
                                : player.setVolume(1)}
                    >
                        {#if player.volume === 0}
                            <VolumeX size={18} />
                        {:else if player.volume < 0.33}
                            <Volume size={18} />
                        {:else if player.volume < 0.66}
                            <Volume1 size={18} />
                        {:else}
                            <Volume2 size={18} />
                        {/if}
                    </button>
                    <div class="w-24">
                        <Slider bind:value={player.volume} />
                    </div>
                </div>
            </div>

            <QueueView bind:showQueue />
        </div>
    </div>
{/if}
