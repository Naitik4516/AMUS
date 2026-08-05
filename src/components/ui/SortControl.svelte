<script lang="ts">
    import { Check, ChevronDown, ChevronUp } from "@lucide/svelte";
    import type { CollectionSortDir, CollectionSortField } from "$lib/utils";
    import { loadSortPref, saveSortPref } from "$lib/utils";
    import { fly } from "svelte/transition";

    interface SortOption {
        value: CollectionSortField;
        label: string;
    }

    let {
        options,
        sortKey,
        field = $bindable("name" as CollectionSortField),
        dir = $bindable("asc" as CollectionSortDir),
    }: {
        options: SortOption[];
        sortKey?: string;
        field?: CollectionSortField;
        dir?: CollectionSortDir;
    } = $props();

    let open = $state(false);

    let initialized = $state(false);
    $effect(() => {
        if (!sortKey || initialized) return;
        initialized = true;
        const pref = loadSortPref(sortKey, { field, dir });
        if (options.some((o) => o.value === pref.field)) {
            field = pref.field;
        }
        dir = pref.dir;
    });

    $effect(() => {
        if (!sortKey) return;
        saveSortPref(sortKey, { field, dir });
    });

    const currentLabel = $derived(
        options.find((o) => o.value === field)?.label ??
            options[0]?.label ??
            "Name",
    );
</script>

<svelte:document
    onmousedown={(e) => {
        if (open && !(e.target as HTMLElement).closest(".sort-control")) {
            open = false;
        }
    }}
/>
<svelte:window onkeydown={(e) => e.key === "Escape" && (open = false)} />

<div class="sort-control flex items-center gap-2">
    <div class="relative">
        <button
            type="button"
            onclick={() => (open = !open)}
            aria-haspopup="listbox"
            aria-expanded={open}
            class="flex items-center gap-2 bg-secondary/60 ring-border ring-1 hover:ring-gray-600/60 hover:ring-2 rounded-full px-5 py-5 text-sm font-semibold transition-all duration-300"
        >
            <span class="text-gray-400 font-medium">Sort:</span>
            <span class="text-white">{currentLabel}</span>
            <ChevronDown
                size={16}
                class="text-gray-400 transition-transform duration-200 {open
                    ? 'rotate-180'
                    : ''}"
            />
        </button>

        {#if open}
            <div
                role="listbox"
                class="absolute right-0 top-full mt-2 z-50 min-w-52 rounded-2xl border bg-secondary/60 shadow-lg backdrop-blur-xl p-2"
                transition:fly={{ duration: 120, y: -4 }}
            >
                {#each options as opt}
                    <button
                        type="button"
                        role="option"
                        aria-selected={field === opt.value}
                        onclick={() => {
                            field = opt.value;
                            open = false;
                        }}
                        class="flex w-full items-center justify-between gap-3 rounded-xl px-3 py-2 text-left text-sm transition-colors {field ===
                        opt.value
                            ? 'text-white bg-gray-300/10'
                            : 'text-zinc-300 hover:text-white hover:bg-gray-300/10'}"
                    >
                        <span>{opt.label}</span>
                        {#if field === opt.value}
                            <Check size={15} class="text-accent shrink-0" />
                        {/if}
                    </button>
                {/each}
            </div>
        {/if}
    </div>

    <button
        type="button"
        onclick={() => (dir = dir === "asc" ? "desc" : "asc")}
        title={dir === "asc" ? "Sort ascending" : "Sort descending"}
        aria-label={dir === "asc" ? "Sort ascending" : "Sort descending"}
        class="flex items-center justify-center bg-secondary/60 rounded-full p-5 w-14 h-14 transition-all duration-300 ring-border ring-1 hover:ring-gray-600/60 hover:ring-2"
    >
        {#if dir === "asc"}
            <ChevronUp size={18} class="text-white" />
        {:else}
            <ChevronDown size={18} class="text-white" />
        {/if}
    </button>
</div>
