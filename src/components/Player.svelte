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
    import { getImageUrl } from "$lib/utils";

    let volumeValue = $state(player.volume);
    let showQueue = $state(true);

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

<div class="fixed bottom-0 left-0 w-full px-5 pb-3 z-15">
    <div
        class="bg-neutral-950/60 border-2 border-neutral-800/40 backdrop-blur-lg flex items-center justify-between px-6 py-4 shadow-2xl rounded-2xl relative"
    >
        <!-- Track Info -->
        <div class="flex items-center gap-4 w-1/4">
            <div
                class="w-14 h-14 rounded-lg bg-neutral-800 shadow-md flex items-center justify-center overflow-hidden shrink-0"
            >
                {#if player.currentTrack?.cover_art}
                    {#await getImageUrl(player.currentTrack.cover_art)}
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
                    class="text-xs text-gray-300 truncate hover:underline cursor-pointer"
                >
                    {player.currentTrack.artists
                        .map((a) => a.name)
                        .join(", ") || "Unknown Artist"}
                </p>
            </div>
            <button
                onclick={toggleFavorite}
                class="ml-2 text-gray-300 hover:text-secondary transition-colors"
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
                    class:text-gray-300={!player.shuffle}
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
                    onclick={() => player.togglePlay()}
                >
                    {#if player.isPlaying}
                        <Pause size={24} fill="currentColor" />
                    {:else}
                        <Play size={24} fill="currentColor" />
                    {/if}
                </button>
                <button
                    class="text-gray-300 hover:text-white transition-colors"
                    onclick={() => player.next()}
                >
                    <SkipForward size={22} fill="currentColor" />
                </button>
                <button
                    class="hover:text-white transition-colors"
                    class:text-secondary={player.repeat !== "none"}
                    class:text-gray-300={player.repeat === "none"}
                    onclick={() => player.toggleRepeat()}
                >
                    {#if player.repeat === "one"}
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
                    {formatDuration(Math.floor(player.progress / 1000))}
                </span>
                <Slider
                    value={(player.progress /
                        1000 /
                        player.currentTrack.duration_seconds) *
                        100}
                    onValueChange={(val: number) => player.seek(val)}
                />
                <span class="text-[10px] font-medium text-gray-400 w-10">
                    {formatDuration(player.currentTrack.duration_seconds)}
                </span>
            </div>
        </div>

        <!-- Volume & Queue -->
        <div class="flex items-center gap-4 w-1/4 justify-end">
            <button
                onclick={() => (showQueue = !showQueue)}
                class="text-gray-300 hover:text-white transition-colors"
                class:text-secondary={showQueue}
            >
                <ListMusic size={20} />
            </button>

            <div class="flex items-center gap-2 group">
                <button
                    class="text-gray-300 group-hover:text-white transition-colors"
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
                class="absolute bottom-full right-1 mb-4 w-90 bg-card/60 backdrop-blur-lg border-2 border-border/70 rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[75vh]"
            >
                <div
                    class="p-4 border-b border-neutral-800 flex justify-between items-center bg-neutral-900/50"
                >
                    <h3 class="font-bold text-white text-lg">Queue</h3>
                    <button
                        onclick={() => (showQueue = false)}
                        class="text-gray-300 hover:text-white"
                    >
                        <X size={18} />
                    </button>
                </div>
                <div class="overflow-y-auto px-4 pb-4">
                    <!-- Now Playing -->
                    {#if player.currentTrack}
                        <h4
                            class="py-2 text-[13px] font-switzer font-bold uppercase tracking-wider text-stone-400"
                        >
                            Now Playing
                        </h4>
                        <div
                            class="w-full flex items-center gap-3 p-2 rounded-xl py-2 bg-white/5 transition-colors text-left cursor-default"
                            tabindex="-1"
                            title={`${player.currentTrack.title} - ${player.currentTrack.artists.map((a: { name: string }) => a.name).join(", ") || "Unknown Artist"}`}
                            role="button"
                        >
                            <div
                                class="w-10 h-10 rounded-lg bg-zinc-800 flex items-center justify-center shrink-0 overflow-hidden relative"
                            >
                                {#if player.currentTrack.cover_art}
                                    {#await getImageUrl(player.currentTrack.cover_art)}
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
                                    <Music size={14} class="text-neutral-600" />
                                {/if}
                                {#if player.isPlaying}
                                    <div
                                        class="absolute inset-0 bg-black/40 flex items-center justify-center"
                                    >
                                        <div class="flex gap-0.5 items-end h-3">
                                            <div
                                                class="w-0.5 bg-accent animate-bounce h-2"
                                            ></div>
                                            <div
                                                class="w-0.5 bg-accent animate-bounce h-3"
                                                style="animation-delay: 0.2s"
                                            ></div>
                                            <div
                                                class="w-0.5 bg-accent animate-bounce h-1.5"
                                                style="animation-delay: 0.4s"
                                            ></div>
                                        </div>
                                    </div>
                                {/if}
                            </div>
                            <div class="flex-1 min-w-0">
                                <p
                                    class="text-sm font-bold truncate text-accent"
                                >
                                    {player.currentTrack.title}
                                </p>
                                <p class="text-xs text-gray-400 truncate">
                                    {player.currentTrack.artists
                                        .map((a: { name: string }) => a.name)
                                        .join(", ") || "Unknown Artist"}
                                </p>
                            </div>
                        </div>
                    {/if}

                    <!-- Next in Queue -->
                    {#if player.userQueue.length > 0}
                        <h4
                            class="py-2 text-[13px] font-bold uppercase tracking-wider text-stone-400"
                        >
                            Next in Queue
                        </h4>
                        {#each player.userQueue as track, i}
                            <!-- svelte-ignore a11y_click_events_have_key_events -->
                            <div
                                class="w-full flex items-center gap-3 p-2 rounded-xl hover:bg-white/10 transition-colors text-left group cursor-pointer"
                                onclick={() => player.play(track)}
                                tabindex="-1"
                                title={`${track.title} - ${track.artists.map((a: { name: string }) => a.name).join(", ") || "Unknown Artist"}`}
                                role="menuitem"
                            >
                                <div
                                    class="w-10 h-10 rounded-lg bg-neutral-800 flex items-center justify-center shrink-0 overflow-hidden relative"
                                >
                                    {#if track.cover_art}
                                        {#await getImageUrl(track.cover_art)}
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
                                </div>
                                <div class="flex-1 min-w-0">
                                    <p class="text-sm font-semibold truncate">
                                        {track.title}
                                    </p>
                                    <p class="text-xs text-gray-400 truncate">
                                        {track.artists
                                            .map(
                                                (a: { name: string }) => a.name,
                                            )
                                            .join(", ") || "Unknown Artist"}
                                    </p>
                                </div>
                                <button
                                    class="opacity-0 group-hover:opacity-100 p-1 text-gray-400 hover:text-red-500 transition-all"
                                    onclick={(e) => {
                                        e.stopPropagation();
                                        player.removeFromQueue(track.id);
                                    }}
                                >
                                    <X size={14} />
                                </button>
                            </div>
                        {/each}
                    {/if}

                    <!-- Next from Playlist/Album/Artist -->
                    {#if player.playNext.length > 0}
                        <p
                            class="py-2 text-[13px] font-bold uppercase tracking-wider text-stone-400"
                        >
                            Next from Suggestions
                        </p>
                        {#each player.playNext.slice(0, 5) as track, i}
                            <!-- svelte-ignore a11y_click_events_have_key_events -->
                            <div
                                class="w-full flex items-center gap-3 p-2 rounded-xl hover:bg-white/10 transition-colors text-left group cursor-pointer opacity-90"
                                onclick={() => player.play(track)}
                                tabindex="-1"
                                title={`${track.title} - ${track.artists.map((a: { name: string }) => a.name).join(", ") || "Unknown Artist"}`}
                                role="button"
                            >
                                <div
                                    class="w-10 h-10 rounded-lg bg-zinc-800 flex items-center justify-center shrink-0 overflow-hidden relative"
                                >
                                    {#if track.cover_art}
                                        {#await getImageUrl(track.cover_art)}
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
                                </div>
                                <div class="flex-1 min-w-0">
                                    <p class="text-sm truncate font-semibold">
                                        {track.title}
                                    </p>
                                    <p class="text-xs text-gray-400 truncate">
                                        {track.artists
                                            .map(
                                                (a: { name: string }) => a.name,
                                            )
                                            .join(", ") || "Unknown Artist"}
                                    </p>
                                </div>
                            </div>
                        {/each}
                    {/if}

                    {#if !player.currentTrack && player.userQueue.length === 0 && player.playNext.length === 0}
                        <p class="px-4 py-8 text-center text-sm text-zinc-500">
                            No tracks in queue
                        </p>
                    {/if}
                </div>
            </div>
        {/if}
    </div>
</div>

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
