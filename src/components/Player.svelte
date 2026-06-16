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
        Music,
        X,
    } from "@lucide/svelte";
    import Slider from "./ui/Slider.svelte";
    import { player } from "$lib/player.svelte";
    import { getCoverUrl } from "$lib/utils";

    let volumeValue = $state(player.volume);
    let showQueue = $state(false);

    $effect(() => {
        player.setVolume(volumeValue);
    });

    async function toggleFavorite() {
        if (player.currentTrack) {
            await player.toggleFavorite(player.currentTrack);
        }
    }

    function formatDuration(seconds: number) {
        const mins = Math.floor(seconds / 60);
        const secs = seconds % 60;
        return `${mins}:${secs.toString().padStart(2, "0")}`;
    }
</script>

{#if player.currentTrack}
    <div class="fixed bottom-0 left-0 w-full p-5 z-15">
        <div
            class="surface flex items-center justify-between px-6 py-4 shadow-2xl rounded-2xl relative"
        >
            <!-- Track Info -->
            <div class="flex items-center gap-4 w-1/4">
                <div
                    class="w-14 h-14 rounded-lg bg-neutral-800 shadow-md flex items-center justify-center overflow-hidden shrink-0"
                >
                    {#if player.currentTrack.cover_art}
                        {#await getCoverUrl(player.currentTrack.cover_art)}
                            <div
                                class="w-full h-full bg-neutral-800 animate-pulse"
                            ></div>
                        {:then url}
                            <img
                                src={url}
                                alt=""
                                class="w-full h-full object-cover"
                            />
                        {/await}
                    {:else}
                        <Music size={24} class="text-neutral-700" />
                    {/if}
                </div>
                <div class="overflow-hidden">
                    <p
                        class="font-bold truncate text-white hover:underline cursor-pointer"
                    >
                        {player.currentTrack.title}
                    </p>
                    <p
                        class="text-xs text-gray-400 truncate hover:underline cursor-pointer"
                    >
                        {player.currentTrack.artist
                            .map((a) => a.name)
                            .join(", ") || "Unknown Artist"}
                    </p>
                </div>
                <button
                    onclick={toggleFavorite}
                    class="ml-2 text-gray-400 hover:text-secondary transition-colors"
                    class:text-secondary={player.currentTrack.is_favorite}
                >
                    <Heart
                        size={18}
                        fill={player.currentTrack.is_favorite
                            ? "currentColor"
                            : "none"}
                    />
                </button>
            </div>

            <!-- Controls -->
            <div class="flex flex-col items-center gap-2 w-1/2 max-w-2xl">
                <div class="flex items-center gap-6">
                    <button
                        class="hover:text-white transition-colors"
                        class:text-secondary={player.shuffle}
                        class:text-gray-400={!player.shuffle}
                        onclick={() => player.toggleShuffle()}
                    >
                        <Shuffle size={18} />
                    </button>
                    <button
                        class="text-gray-400 hover:text-white transition-colors"
                        onclick={() => player.previous()}
                    >
                        <SkipBack size={22} fill="currentColor" />
                    </button>
                    <button
                        class="bg-white text-black rounded-full p-3 hover:scale-105 transition-transform shadow-lg"
                        onclick={() => player.togglePlay()}
                    >
                        {#if player.isPlaying}
                            <Pause size={24} fill="currentColor" />
                        {:else}
                            <Play size={24} fill="currentColor" />
                        {/if}
                    </button>
                    <button
                        class="text-gray-400 hover:text-white transition-colors"
                        onclick={() => player.next()}
                    >
                        <SkipForward size={22} fill="currentColor" />
                    </button>
                    <button
                        class="hover:text-white transition-colors"
                        class:text-secondary={player.repeat !== "none"}
                        class:text-gray-400={player.repeat === "none"}
                        onclick={() => player.toggleRepeat()}
                    >
                        {#if player.repeat === "one"}
                            <Repeat1 size={18} />
                        {:else}
                            <Repeat size={18} />
                        {/if}
                    </button>
                </div>
                <div class="w-full flex items-center gap-3">
                    <span
                        class="text-[10px] font-medium text-gray-500 w-10 text-right"
                    >
                        {formatDuration(Math.floor(player.progress / 1000))}
                    </span>
                    <Slider
                        value={(player.progress /
                            1000 /
                            player.currentTrack.duration_seconds) *
                            100}
                        onValueChange={(val: number) => player.seek(val)}
                    />
                    <span class="text-[10px] font-medium text-gray-500 w-10">
                        {formatDuration(player.currentTrack.duration_seconds)}
                    </span>
                </div>
            </div>

            <!-- Volume & Queue -->
            <div class="flex items-center gap-4 w-1/4 justify-end">
                <button
                    onclick={() => (showQueue = !showQueue)}
                    class="text-gray-400 hover:text-white transition-colors"
                    class:text-secondary={showQueue}
                >
                    <ListMusic size={20} />
                </button>

                <div class="flex items-center gap-2 group">
                    <button
                        class="text-gray-400 group-hover:text-white transition-colors"
                        onclick={() =>
                            volumeValue != 0
                                ? (volumeValue = 0)
                                : (volumeValue = 100)}
                    >
                        {#if volumeValue === 0}
                            <VolumeX size={18} />
                        {:else if volumeValue < 33}
                            <Volume size={18} />
                        {:else if volumeValue < 66}
                            <Volume1 size={18} />
                        {:else}
                            <Volume2 size={18} />
                        {/if}
                    </button>
                    <div class="w-24">
                        <Slider bind:value={volumeValue} />
                    </div>
                </div>
            </div>

            <!-- Queue Dropup -->
            {#if showQueue}
                <div
                    class="absolute bottom-full right-6 mb-4 w-80 bg-white/5 backdrop-blur-md border border-neutral-800 rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[70vh]"
                >
                    <div
                        class="p-4 border-b border-neutral-800 flex justify-between items-center bg-neutral-900/50"
                    >
                        <h3 class="font-bold text-white">Queue</h3>
                        <button
                            onclick={() => (showQueue = false)}
                            class="text-gray-400 hover:text-white"
                        >
                            <X size={18} />
                        </button>
                    </div>
                    <div class="overflow-y-auto py-2">
                        {#each player.queue as track, i}
                            <div
                                class="w-full flex items-center gap-3 px-4 py-2 hover:bg-white/10 transition-colors text-left group cursor-pointer"
                                class:bg-neutral-800={i === player.currentIndex}
                                onclick={() => player.play(track, player.queue)}
                                tabindex="-1"
                                title={`${track.title} - ${track.artist.map((a) => a.name).join(", ") || "Unknown Artist"}`}
                                role="button"
                            >
                                <div
                                    class="w-8 h-8 rounded bg-neutral-800 flex items-center justify-center shrink-0 overflow-hidden relative"
                                >
                                    {#if track.cover_art}
                                        {#await getCoverUrl(track.cover_art)}
                                            <div
                                                class="absolute inset-0 bg-neutral-800 animate-pulse"
                                            ></div>
                                        {:then url}
                                            <img
                                                src={url}
                                                alt=""
                                                class="w-full h-full object-cover"
                                            />
                                        {/await}
                                    {:else}
                                        <Music
                                            size={14}
                                            class="text-neutral-600"
                                        />
                                    {/if}

                                    {#if i === player.currentIndex && player.isPlaying}
                                        <div
                                            class="absolute inset-0 bg-black/40 flex items-center justify-center"
                                        >
                                            <div
                                                class="flex gap-0.5 items-end h-3"
                                            >
                                                <div
                                                    class="w-0.5 bg-secondary animate-bounce h-2"
                                                ></div>
                                                <div
                                                    class="w-0.5 bg-secondary animate-bounce h-3"
                                                    style="animation-delay: 0.2s"
                                                ></div>
                                                <div
                                                    class="w-0.5 bg-secondary animate-bounce h-1.5"
                                                    style="animation-delay: 0.4s"
                                                ></div>
                                            </div>
                                        </div>
                                    {/if}
                                </div>
                                <div class="flex-1 min-w-0">
                                    <p
                                        class="text-sm font-bold truncate"
                                        class:text-secondary={i ===
                                            player.currentIndex}
                                    >
                                        {track.title}
                                    </p>
                                    <p class="text-xs text-gray-500 truncate">
                                        {track.artist
                                            .map((a) => a.name)
                                            .join(", ") || "Unknown Artist"}
                                    </p>
                                </div>
                                <button
                                    class="opacity-0 group-hover:opacity-100 p-1 text-gray-500 hover:text-red-500 transition-all"
                                    onclick={(e) => {
                                        e.stopPropagation();
                                        player.removeFromQueue(track.id);
                                    }}
                                >
                                    <X size={14} />
                                </button>
                            </div>
                        {/each}
                    </div>
                </div>
            {/if}
        </div>
    </div>
{/if}

<style>
    @keyframes bounce {
        0%,
        100% {
            transform: scaleY(0.5);
        }
        50% {
            transform: scaleY(1);
        }
    }
    .animate-bounce {
        animation: bounce 0.6s infinite ease-in-out;
        transform-origin: bottom;
    }
</style>
