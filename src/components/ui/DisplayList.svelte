<script lang="ts">
    import { User, Search } from "@lucide/svelte";

    interface DisplayListProps {
        listItems: Array<{ name: string; [key: string]: any }>;
        title: string;
        Card: any;
    }

    let { listItems, title, Card }: DisplayListProps = $props();
    let searchQuery = $state("");

    let filteredItems = $derived(
        listItems.filter((a) =>
            a.name.toLowerCase().includes(searchQuery.toLowerCase()),
        ),
    );
</script>

<div class="p-8 w-full flex flex-col h-full">
    <div class="flex items-center justify-between mb-8">
        <h1 class="text-6xl font-black text-white">{title}</h1>

        <div
            class="flex items-center gap-2 bg-secondary/70 rounded-full px-6 py-4 w-50 border-2 border-transparent focus-within:border-border focus-within:w-70 hover:bg-secondary/80 transition-all duration-300 mr-10"
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
            class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 gap-6 px-10 container overflow-scroll"
        >
            {#each filteredItems as item}
                <Card data={item} />
            {/each}
        </div>
    {/if}
</div>
