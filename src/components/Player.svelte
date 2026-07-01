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
        X,
    } from "@lucide/svelte";
    import Slider from "./ui/Slider.svelte";
    import { player } from "$lib/player.svelte";
    import { getImageUrl } from "$lib/utils";
    import TrackListSmall from "./ui/TrackListSmall.svelte";
    import { formatDurationColon } from "$lib/utils";

    import { ui } from "$lib/shortcut-handler.svelte";
    import Marquee from "./ui/Marquee.svelte";
    let volumeValue = $state(player.volume);
    let showQueue = $state(false);
    let seekDragPercent = $state<number | null>(null);

    $effect(() => {
        if (ui.queueVisible) showQueue = true;
    });

    let displayProgress = $derived(
        seekDragPercent !== null && player.currentTrack
            ? (seekDragPercent / 100) *
                  player.currentTrack.duration_seconds *
                  1000
            : player.progress,
    );

    $effect(() => {
        player.setVolume(volumeValue);
    });

    async function toggleFavorite() {
        if (player.currentTrack) {
            await player.toggleFavorite(player.currentTrack);
        }
    }
</script>

<div class="fixed bottom-0 left-0 w-full px-4 pb-2 z-15">
    <div
        class="bg-neutral-950/60 border-2 border-neutral-800/40 backdrop-blur-xl flex items-center justify-between px-6 py-4 shadow-2xl rounded-3xl relative"
    >
        {#if !player.currentTrack}
            <p class="text-sm text-gray-400">No track playing</p>
        {:else}
            <!-- Track Info -->
            <div class="flex items-center gap-4 w-1/4">
                <div
                    class="w-14 h-14 rounded-lg bg-neutral-800 shadow-md flex items-center justify-center overflow-hidden shrink-0"
                >
                    {#if player.currentTrack?.cover_art}
                        <img
                            src={await getImageUrl(
                                player.currentTrack?.cover_art,
                            )}
                            alt=""
                            class="w-full h-full object-cover"
                        />
                    {:else}
                        <Music2 size={32} />
                    {/if}
                </div>
                <div class="flex flex-col min-w-0 flex-1 overflow-hidden">
                    <Marquee>
                        <a
                            href="/library/track/{player.currentTrack?.id}"
                            class="font-bold truncate text-white hover:underline cursor-pointer inline-block"
                        >
                            {player.currentTrack?.title}
                        </a>
                    </Marquee>
                    <Marquee>
                        <p class=" text-gray-300 truncate -mt-1">
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
                    class:text-secondary={player.currentTrack?.is_favorite}
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
            <div class="flex flex-col items-center gap-2 w-1/2 max-w-2xl">
                <div class="flex items-center gap-6">
                    <button
                        class="hover:text-white transition-colors"
                        class:text-white={player.shuffle}
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
                        class:text-white={player.repeat !== "none"}
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
                        {formatDurationColon(
                            Math.floor(displayProgress / 1000),
                        )}
                    </span>
                    <Slider
                        value={(player.progress /
                            1000 /
                            player.currentTrack.duration_seconds) *
                            100}
                        onDragChange={(val) => (seekDragPercent = val)}
                        onValueChange={(val) => {
                            seekDragPercent = null;
                            player.seek(val);
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
            <div class="flex items-center gap-4 w-1/4 justify-end">
                <button
                    onclick={() => (showQueue = !showQueue)}
                    class="text-gray-300 hover:text-white transition-colors"
                    class:text-secondary={showQueue}
                >
                    <ListMusic size={20} />
                </button>

                <div
                    class="flex items-center gap-2 group"
                    onwheel={(e) => {
                        e.preventDefault();
                        const delta = e.deltaY > 0 ? -5 : 5;
                        volumeValue = Math.max(
                            0,
                            Math.min(100, volumeValue + delta),
                        );
                    }}
                >
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

                            <TrackListSmall
                                track={player.currentTrack}
                                titleColor="text-accent"
                                className="rounded-xl"
                            />
                        {/if}

                        <!-- Next in Queue -->
                        {#if player.userQueue.length > 0}
                            <h4
                                class="py-2 text-[13px] font-bold uppercase tracking-wider text-stone-400"
                            >
                                Next in Queue
                            </h4>
                            {#each player.userQueue as track, i}
                                <TrackListSmall
                                    {track}
                                    className="rounded-xl"
                                />
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
                                <TrackListSmall
                                    {track}
                                    className="rounded-xl"
                                />
                            {/each}
                        {/if}

                        {#if !player.currentTrack && player.userQueue.length === 0 && player.playNext.length === 0}
                            <p
                                class="px-4 py-8 text-center text-sm text-zinc-500"
                            >
                                No tracks in queue
                            </p>
                        {/if}
                    </div>
                </div>
            {/if}
        {/if}
    </div>
</div>
