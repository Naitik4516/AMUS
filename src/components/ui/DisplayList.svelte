<script lang="ts">
    import { Search } from "@lucide/svelte";
    import type { Album, Artist, Genre, Playlist } from "$lib/types";
    import { Virtualizer, type VirtualizerHandle } from "virtua/svelte";
    import Button from "./button/button.svelte";
    import { fly } from "svelte/transition";
    import { MoveUp } from "@lucide/svelte";
    import type { Snippet } from "svelte";
    import SortControl from "./SortControl.svelte";
    import type {
        CollectionSortDir,
        CollectionSortField,
    } from "$lib/utils";
    import { sortCollectionItems } from "$lib/utils";

    interface SortOption {
        value: CollectionSortField;
        label: string;
    }

    interface DisplayListProps {
        listItems: (Album | Artist | Playlist | Genre)[];
        title: string;
        Card: any;
        fallBack: Snippet;
        cellHeight?: number;
        cellWidth?: number;
        sortOptions?: SortOption[];
        sortKey?: string;
    }

    let {
        listItems,
        title,
        Card,
        fallBack,
        cellHeight = 340,
        cellWidth = 265,
        sortOptions = [
            { value: "name", label: "Name" },
            { value: "added_at", label: "Date Added" },
            { value: "last_played_at", label: "Recently Played" },
            { value: "total_plays", label: "Most Played" },
            { value: "track_count", label: "Track Count" },
        ],
        sortKey,
    }: DisplayListProps = $props();

    let searchQuery = $state("");
    let showFAB = $state(false);
    let grid: VirtualizerHandle | null = $state(null);
    let sortField = $state<CollectionSortField>("name");
    let sortDir = $state<CollectionSortDir>("asc");

    let visibleItems = $derived.by(() => {
        const filtered = listItems.filter((a) =>
            a.name.toLowerCase().includes(searchQuery.toLowerCase()),
        );
        return sortCollectionItems(filtered, sortField, sortDir);
    });

    let gridWidth = $state(500);
    let cols = $derived(Math.max(1, Math.floor(gridWidth / cellWidth)));

    const data = $derived.by(() => {
        const result = [];
        for (let i = 0; i < visibleItems.length; i += cols) {
            result.push(visibleItems.slice(i, i + cols));
        }
        return result;
    });
</script>

<div class="px-2 w-full flex flex-col">
    <div class="flex items-center justify-between py-4 mt-5">
        <h1 class="text-7xl font-black font-switzer text-white">{title}</h1>

        <div class="flex items-center gap-3 mr-14">
            <SortControl options={sortOptions} {sortKey} bind:field={sortField} bind:dir={sortDir} />

            <div
                class="flex items-center gap-2 bg-secondary rounded-full px-6 py-5 w-60 ring-gray-600 focus-within:ring-2 focus-within:w-80 transition-all duration-300"
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
    </div>

    {#if listItems.length === 0}
        <div
            class="flex flex-col items-center justify-center py-20 text-gray-500"
        >
            {@render fallBack()}
        </div>
    {:else if visibleItems.length === 0}
        <div
            class="flex flex-col items-center justify-center py-20 text-gray-500"
        >
            <Search size={48} class="mb-4 opacity-20" />
            <p class="text-xl font-medium">No results found</p>
            <p class="text-sm">Try a different search term.</p>
        </div>
    {:else}
        <div
            class="virtualizer mask-y-from-90% pt-8 scroll-smooth w-full h-[calc(90vh-120px)] overflow-y-auto"
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
                    <div style="display: flex; height: {cellHeight}px;">
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
        class="fixed bottom-30 right-8 z-50"
        transition:fly={{ duration: 300, y: 150 }}
    >
        <Button
            variant="outline"
            onclick={() => grid && grid.scrollTo(0)}
            size="icon-xl"
            class="backdrop-blur-md tooltip"
            title="Scroll to top"
        >
            <MoveUp />
        </Button>
    </div>
{/if}
