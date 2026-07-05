<script lang="ts">
    import {
        Play,
        Pause,
        SkipBack,
        SkipForward,
        Shuffle,
        Repeat,
        Repeat1,
        Heart,
        Pin,
        PinOff,
    } from "@lucide/svelte";
    import Slider from "$components/ui/Slider.svelte";
    import { player } from "$lib/player.svelte";
    import { getImageUrl } from "$lib/utils";
    import { formatDurationColon } from "$lib/utils";
    import { invoke } from "@tauri-apps/api/core";
    import Marquee from "$components/ui/Marquee.svelte";

    let volumeValue = $state(player.volume);
    let seekDragPercent = $state<number | null>(null);
    let isPinned = $state(true);

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

    async function togglePin() {
        isPinned = await invoke<boolean>("toggle_mini_player_pin");
    }
</script>

{#if player.currentTrack}
    <div
        class="relative bg-gray-900 h-screen w-screen rounded-4xl overflow-hidden"
    >
        <img
            src={await getImageUrl(player.currentTrack?.cover_art)}
            alt="Cover Art"
            class="w-screen h-screen object-cover blur-3xl -z-10 rounded-4xl"
        />

        <div class="absolute top-3 right-3 z-20">
            <button
                onclick={togglePin}
                class="text-gray-300 hover:text-white transition-colors"
                title={isPinned ? "Unpin Mini Player" : "Pin Mini Player"}
            >
                {#if isPinned}
                    <PinOff size={16} strokeWidth={2.5} />
                {:else}
                    <Pin size={16} strokeWidth={2.5} />
                {/if}
            </button>
        </div>

        <div
            class="absolute inset-0 z-10 p-5 grid grid-cols-5 gap-3 items-center"
        >
            <div class="col-span-2">
                <img
                    src={await getImageUrl(player.currentTrack?.cover_art)}
                    alt="Cover Art"
                    class="oject-cover rounded-3xl shadow-lg shadow-zinc-800/60 w-full h-full"
                />
            </div>
            <div class="col-span-3 flex flex-col gap-4 items-center">
                <div class="flex flex-col items-center overflow-hidden w-50">
                    <Marquee>
                        <div
                            class="font-extrabold text-xl text-white inline-block truncate"
                        >
                            {player.currentTrack?.title ?? ""}
                        </div>
                    </Marquee>
                    <Marquee>
                        <span class=" text-gray-300" id="text"
                            >{player.currentTrack?.artists?.[0]?.name ??
                                ""}</span
                        >
                    </Marquee>
                </div>
                <div class="flex items-center gap-6">
                    <button
                        class="text-gray-300 hover:text-white transition-colors"
                        onclick={() => player.previous()}
                    >
                        <SkipBack size={26} fill="currentColor" />
                    </button>
                    <button
                        class="text-white"
                        onclick={() => player.togglePlay()}
                    >
                        {#if player.isPlaying}
                            <Pause size={40} fill="currentColor" />
                        {:else}
                            <Play size={40} fill="currentColor" />
                        {/if}
                    </button>
                    <button
                        class="text-gray-300 hover:text-white transition-colors"
                        onclick={() => player.next()}
                    >
                        <SkipForward size={26} fill="currentColor" />
                    </button>
                </div>
                <div class="flex items-center gap-4 w-full">
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
                <div class="flex items-center justify-between w-full px-4">
                    <button
                        class="hover:text-white transition-colors"
                        class:text-gray-100={player.shuffle}
                        class:text-stone-300={!player.shuffle}
                        onclick={() => player.toggleShuffle()}
                    >
                        <Shuffle size={20} strokeWidth={2.5} />
                    </button>
                    <button
                        onclick={toggleFavorite}
                        class="ml-2 {player.currentTrack?.is_favorite
                            ? 'text-rose-600 fill-rose-600'
                            : 'text-gray-300'}  hover:text-secondary transition-colors"
                        class:text-secondary={player.currentTrack?.is_favorite}
                    >
                        <Heart
                            size={24}
                            class={player.currentTrack?.is_favorite
                                ? "text-rose-600 fill-rose-600"
                                : "text-gray-300"}
                            strokeWidth={2.5}
                        ></Heart>
                    </button>
                    <button
                        class="hover:text-white transition-colors"
                        class:text-gray-100={player.repeat !== "none"}
                        class:text-stone-300={player.repeat === "none"}
                        onclick={() => player.toggleRepeat()}
                    >
                        {#if player.repeat === "one"}
                            <Repeat1 size={20} strokeWidth={2.5} />
                        {:else}
                            <Repeat size={20} strokeWidth={2.5} />
                        {/if}
                    </button>
                </div>
            </div>
        </div>
    </div>
{/if}

<style>
    button {
        cursor: pointer;
    }
</style>
