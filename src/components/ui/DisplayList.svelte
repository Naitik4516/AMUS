<script lang="ts">
    import { User, Search } from "@lucide/svelte";
    import type { Album, Artist, Playlist } from "$lib/types";
    import { Virtualizer } from "virtua/svelte";
    import Button from "./button/button.svelte";
    import {  fade, fly } from "svelte/transition";
    import { MoveUp } from "@lucide/svelte";

    interface DisplayListProps {
        listItems: (Album | Artist | Playlist)[];
        title: string;
        Card: any;
    }

    let { listItems, title, Card }: DisplayListProps = $props();

    let searchQuery = $state("");
    let showFAB = $state(false);
    let grid: Element | null = $state(null);

    let filteredItems = $derived(
        listItems.filter((a) =>
            a.name.toLowerCase().includes(searchQuery.toLowerCase()),
        ),
    );

    const cellWidth = 280;
    const cellHeight = 320;

    let gridWidth = $state(500);
    let cols = $derived(Math.max(1, Math.floor(gridWidth / cellWidth)));

    const data = $derived.by(() => {
        const result = [];
        for (let i = 0; i < filteredItems.length; i += cols) {
            result.push(filteredItems.slice(i, i + cols));
        }
        return result;
    });
</script>

<div class="px-2 w-full flex flex-col">
    <div class="flex items-center justify-between py-4 mt-5">
        <h1 class="text-7xl font-black font-switzer text-white">{title}</h1>

        <div
            class="flex items-center gap-2 bg-secondary rounded-full px-6 py-5 w-60 ring-gray-600 focus-within:ring-2 focus-within:w-80 transition-all duration-300 mr-14"
        >
            <Search size={18} class="text-gray-400" />
            <input
                type="text"
                placeholder="Search {title.toLowerCase()}..."
                bind:value={searchQuery}
                class="bg-transparent border-none outline-none text-sm text-white w-full"
            />
        </div>
    </div>

    {#if listItems.length === 0}
        <div
            class="flex flex-col items-center justify-center py-20 text-gray-500"
        >
            <User size={64} class="mb-4 opacity-20" />
            <p class="text-xl font-medium">No artists found</p>
            <p class="text-sm">Scan your music library to see artists here.</p>
        </div>
    {:else if filteredItems.length === 0}
        <div
            class="flex flex-col items-center justify-center py-20 text-gray-500"
        >
            <Search size={48} class="mb-4 opacity-20" />
            <p class="text-xl font-medium">No results found</p>
            <p class="text-sm">Try a different search term.</p>
        </div>
    {:else}
        <div
            class="virtualizer mask-y-from-90% pt-8 scroll-smooth w-full h-[84vh] overflow-y-auto"
            bind:clientWidth={gridWidth}
        >
            <Virtualizer
                {data}
                getKey={(r) => r[0].id}
                itemSize={cellHeight}
                onscroll={(offset) => (showFAB = offset > 300)}
                bind:this={grid}
            >
                {#snippet children(rowItems, rowIndex)}
                    <div
                        style="display: flex; height: {cellHeight}px;"
                        transition:fade={{ delay: 50 }}
                    >
                        {#each rowItems as item, colIndex (colIndex)}
                            <div style="padding: 8px; flex: 1;">
                                <Card data={item} />
                            </div>
                        {/each}
                        <!-- Fill empty space if the last row isn't full -->
                        {#if rowItems.length < cols}
                            {#each Array(cols - rowItems.length) as _}
                                <div style="flex: 1;"></div>
                            {/each}
                        {/if}
                    </div>
                {/snippet}
            </Virtualizer>
        </div>
    {/if}
</div>

{#if showFAB}
    <div
        class="fixed {true ? 'bottom-30' : 'bottom-5'} right-8 z-50"
        transition:fly={{ duration: 300, y: 150 }}
    >
        <Button
            variant="outline"
            onclick={() =>
                grid && grid.scrollTo({ top: 0, behavior: "smooth" })}
            size="icon-xl"
            class="backdrop-blur-md tooltip"
            title="Scroll to top"
        >
            <MoveUp />
        </Button>
    </div>
{/if}
