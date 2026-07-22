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
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import ResizeHandlers from "$components/ResizeHandlers.svelte";

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

    const startDrag = async (_event: MouseEvent) => {
        const appWindow = getCurrentWindow();
        await appWindow.startDragging();
    };
</script>

<div
    class="relative bg-zinc-900 h-screen w-screen rounded-4xl overflow-hidden select-none"
>
    {#if player.currentTrack}
        <img
            src={store.getImageSrc(player.currentTrack?.cover_art)}
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
                    src={store.getImageSrc(player.currentTrack?.cover_art)}
                    alt="Cover Art"
                    class="oject-cover rounded-3xl shadow-lg shadow-zinc-800/60 w-full h-full select-none"
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
                        <span
                            class="text-zinc-300 font-medium -ml-0.5"
                            id="text"
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
                        onclick={() => player.playPause()}
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
                        class="text-[10px] font-medium text-zinc-300 w-10 text-right"
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
                        data-tauri-no-drag
                    />
                    <span class="text-[10px] font-medium text-zinc-300 w-10">
                        {formatDurationColon(
                            player.currentTrack.duration_seconds,
                        )}
                    </span>
                </div>
                <div class="flex items-center justify-between w-full px-4">
                    <button
                        class="hover:text-white transition-colors"
                        class:text-white={player.shuffleEnabled}
                        class:text-gray-300={!player.shuffleEnabled}
                        onclick={() => player.toggleShuffle()}
                    >
                        <Shuffle size={18} />
                    </button>
                    <button
                        onclick={toggleFavorite}
                        class="ml-2 {player.currentTrack?.is_favorite
                            ? 'text-rose-600 fill-rose-600'
                            : 'text-gray-300'}  hover:text-secondary transition-colors"
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
                        class:text-white={player.repeatMode !== "OFF"}
                        class:text-gray-300={player.repeatMode === "OFF"}
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
    {:else}
        <div class="flex flex-col items-center justify-center h-full gap-3">
            <div
                class="size-16 rounded-full bg-white/5 flex items-center justify-center"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.5"
                    class="size-8 text-zinc-500"
                >
                    <path d="M9 18V5l12-2v13" />
                    <circle cx="6" cy="18" r="3" />
                    <circle cx="18" cy="16" r="3" />
                </svg>
            </div>
            <div class="text-center">
                <p class="text-zinc-400 text-sm font-medium">Nothing Playing</p>
                <p class="text-zinc-600 text-xs mt-0.5">
                    Play music from your library
                </p>
            </div>
        </div>
    {/if}
</div>

<ResizeHandlers />
