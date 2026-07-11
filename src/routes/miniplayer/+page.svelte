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
    import { formatDurationColon } from "$lib/utils";
    import { store } from "$lib/stores.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import Marquee from "$components/ui/Marquee.svelte";

    let volumeValue = $state(player.volume);
    let isPinned = $state(true);

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
            src={store.getImageSrc(player.currentTrack?.cover_art)}
            alt="Cover Art"
            class="w-screen h-screen object-cover blur-3xl -z-10 rounded-4xl"
        />

        <div class="absolute top-3 right-3 z-20">
            <button
                onclick={togglePin}
                class="text-muted-foreground hover:text-white transition-colors"
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
                    src={store.getImageSrc(player.currentTrack?.cover_art)}
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
                        <span class="text-gray-300 font-medium" id="text"
                            >{player.currentTrack?.artists?.[0]?.name ??
                                ""}</span
                        >
                    </Marquee>
                </div>
                <div class="flex items-center gap-6">
                    <button
                        class="text-muted-foreground hover:text-white transition-colors"
                        onclick={() => player.previous()}
                    >
                        <SkipBack size={26} fill="currentColor" />
                    </button>
                    <button
                        class="text-white"
                        onclick={() => player.playPause()}
                    >
                        {#if player.isPlaying}
                            <Pause size={40} fill="currentColor" />
                        {:else}
                            <Play size={40} fill="currentColor" />
                        {/if}
                    </button>
                    <button
                        class="text-muted-foreground hover:text-white transition-colors"
                        onclick={() => player.next()}
                    >
                        <SkipForward size={26} fill="currentColor" />
                    </button>
                </div>
                <div class="flex items-center gap-4 w-full">
                    <span
                        class="text-[10px] font-medium text-gray-400 w-10 text-right"
                    >
                        {formatDurationColon(player.position)}
                    </span>
                    <Slider
                        value={player.progress}
                        onValueChange={(val) => {
                            if (player.currentTrack) {
                                player.seek(
                                    val * player.currentTrack.duration_seconds,
                                );
                            }
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
                        class:text-white={player.shuffleEnabled}
                        class:text-muted-foreground={!player.shuffleEnabled}
                        onclick={() => player.toggleShuffle()}
                    >
                        <Shuffle size={18} />
                    </button>
                    <button
                        onclick={toggleFavorite}
                        class="ml-2 {player.currentTrack?.is_favorite
                            ? 'text-rose-600 fill-rose-600'
                            : 'text-muted-foreground'}  hover:text-secondary transition-colors"
                    >
                        <Heart
                            size={24}
                            class={player.currentTrack?.is_favorite
                                ? "text-rose-600 fill-rose-600"
                                : "text-muted-foreground"}
                            strokeWidth={2.5}
                        ></Heart>
                    </button>
                    <button
                        class="hover:text-white transition-colors"
                        class:text-white={player.repeatMode !== "OFF"}
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
            </div>
        </div>
    </div>
{/if}

<style>
    button {
        cursor: pointer;
    }
</style>
