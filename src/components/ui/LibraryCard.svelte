<script lang="ts">
    import type { Component, Snippet } from "svelte";

    interface LibraryCardProps {
        label: string;
        Icon: Component | Snippet;
        href: string;
        isSnippet?: boolean;
    }
    let { label, Icon, href, isSnippet }: LibraryCardProps = $props();
</script>

<a
    {href}
    class="flex gap-4 items-center w-full h-30 px-4 py-2 bg-white/5 border-3 border-secondary/20 hover:border-secondary/60 hover:bg-white/10 rounded-4xl transition-colors duration-300 cursor-pointer backdrop-blur-sm font-bold text-lg"
>
    <span>
        {#if isSnippet}
            {@render Icon()}
        {:else}
            <Icon size={36} />
        {/if}
    </span>
    <div class="text-center">
        {label}
    </div>
</a>
