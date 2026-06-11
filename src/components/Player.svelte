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
    } from "@lucide/svelte";
    import Slider from "./ui/Slider.svelte";
    import { player } from "../lib/player.svelte";

    let volumeValue = $state(player.volume);

    $effect(() => {
        player.setVolume(volumeValue);
    });
</script>

{#if player.currentTrack}
    <div class="fixed bottom-0 left-0 w-full p-5 z-50">
        <div
            class="surface flex items-center justify-between px-6 py-4 shadow-lg rounded-2xl"
        >
            <!-- Track Info -->
            <div class="flex items-center gap-4 w-1/4">
                <img
                    src="/PhonographRecord.png"
                    alt="Album Cover"
                    class="w-12 h-12 rounded-md"
                />
                <div class="overflow-hidden">
                    <p class="font-semibold truncate">{player.currentTrack.title}</p>
                    <p class="text-sm text-gray-400 truncate">{player.currentTrack.artist}</p>
                </div>
            </div>

            <!-- Controls -->
            <div class="flex flex-col items-center gap-2 w-1/2">
                <div class="flex items-center gap-5">
                    <button 
                        class="hover:text-white transition-colors"
                        class:text-secondary={player.shuffle}
                        class:text-gray-400={!player.shuffle}
                        onclick={() => player.toggleShuffle()}
                    >
                        <Shuffle size={20} />
                    </button>
                    <button 
                        class="text-gray-400 hover:text-white transition-colors"
                        onclick={() => player.previous()}
                    >
                        <SkipBack size={20} />
                    </button>
                    <button
                        class="bg-secondary text-gray-900 rounded-full p-3.5 hover:bg-gray-200 transition-colors"
                        onclick={() => player.togglePlay()}
                    >
                        {#if player.isPlaying}
                            <Pause size={20} />
                        {:else}
                            <Play size={20} />
                        {/if}
                    </button>
                    <button 
                        class="text-gray-400 hover:text-white transition-colors"
                        onclick={() => player.next()}
                    >
                        <SkipForward size={20} />
                    </button>
                    <button 
                        class="hover:text-white transition-colors"
                        class:text-secondary={player.repeat !== 'none'}
                        class:text-gray-400={player.repeat === 'none'}
                        onclick={() => player.toggleRepeat()}
                    >
                        {#if player.repeat === 'one'}
                            <Repeat1 size={20} />
                        {:else}
                            <Repeat size={20} />
                        {/if}
                    </button>
                </div>
                <div class="w-full flex items-center gap-3">
                    <span class="text-xs text-gray-400 w-10 text-right">
                        {Math.floor(player.progress / 60)}:{(player.progress % 60).toString().padStart(2, '0')}
                    </span>
                    <Slider 
                        value={(player.progress / player.currentTrack.duration_seconds) * 100} 
                        onValueChange={(val: number) => player.seek(val)}
                    />
                    <span class="text-xs text-gray-400 w-10">
                        {Math.floor(player.currentTrack.duration_seconds / 60)}:{(player.currentTrack.duration_seconds % 60).toString().padStart(2, '0')}
                    </span>
                </div>
            </div>

            <!-- Volume -->
            <div class="flex items-center gap-3 w-1/4 justify-end">
                <button
                    onclick={() => (volumeValue != 0 ? (volumeValue = 0) : (volumeValue = 100))}
                >
                    {#if volumeValue === 0}
                        <VolumeX size={20} color="gray" />
                    {:else if volumeValue < 33}
                        <Volume size={20} />
                    {:else if volumeValue < 66}
                        <Volume1 size={20} />
                    {:else}
                        <Volume2 size={20} />
                    {/if}
                </button>
                <div class="w-24">
                    <Slider bind:value={volumeValue} />
                </div>
            </div>
        </div>
    </div>
{/if}

