<script lang="ts">
    import type { Track } from "$lib/types";
    import { MicVocal } from "@lucide/svelte";
    import { store } from "$lib/stores.svelte";

    let { track }: { track: Track } = $props();
</script>

<div class="flex flex-col gap-2">
    {#if track.artists.length > 0}
        <ul class="max-h-60 overflow-y-auto">
            {#each track.artists as artist}
                <li>
                    <a
                        class="flex items-center w-full gap-2 rounded-xl px-3 py-1 text-zinc-200 transition-colors hover:bg-gray-400/10 hover:text-white cursor-pointer"
                        href={`/library/artists/${artist.id}`}
                    >
                        <div
                            class="aspect-square w-9 rounded-2xl overflow-hidden bg-neutral-800 shadow-lg relative"
                        >
                            {#if artist.profile_image}
                                {#if store.getImageSrc(artist.profile_image, "artist")}
                                    <img
                                        src={store.getImageSrc(artist.profile_image, "artist")}
                                        alt={artist.name}
                                        class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
                                    />
                                {:else}
                                    <div
                                        class="absolute inset-0 flex items-center justify-center"
                                    >
                                        <MicVocal
                                            size={20}
                                            class="text-neutral-600"
                                        />
                                    </div>
                                {/if}
                            {:else}
                                <div
                                    class="absolute inset-0 flex items-center justify-center"
                                >
                                    <MicVocal
                                        size={20}
                                        class="text-neutral-600"
                                    />
                                </div>
                            {/if}
                        </div>
                        <div class="flex-1 truncate text-left">
                            {artist.name}
                        </div>
                    </a>
                </li>
            {/each}
        </ul>
    {:else}
        <p class="text-sm text-zinc-500 px-3">No artists found</p>
    {/if}
</div>
