<script lang="ts">
    import { ChevronRight } from "@lucide/svelte";
    let {
        label,
        Icon,
        onclick,
        children,
    }: {
        label: string;
        Icon?: any;
        onclick?: () => void;
        children?: import("svelte").Snippet;
    } = $props();

    let showSubmenu = $state(false);
</script>

<!-- svelte-ignore a11y_mouse_events_have_key_events -->
<div
    class="relative"
    onmouseenter={() => (showSubmenu = true)}
    onmouseleave={() => (showSubmenu = false)}
    role="none"
>
    <button
        class="flex gap-2 items-center w-full px-4 py-2 hover:bg-white/10 rounded-lg text-left text-sm transition-colors duration-300 cursor-pointer text-white"
        {onclick}
    >
        {#if Icon}
            <Icon size={16} />
        {/if}
        <span>{label}</span>
        {#if children}
            <div class="flex-grow"></div>
            <ChevronRight size={14} />
        {/if}
    </button>
    {#if children && showSubmenu}
        <div class="absolute left-full -top-2 ml-1">
            {@render children()}
        </div>
    {/if}
</div>
