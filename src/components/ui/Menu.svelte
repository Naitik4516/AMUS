<script lang="ts">
    let {
        children,
        class: className = "",
    }: { children?: import("svelte").Snippet; class?: string } = $props();
</script>

<div
    class="bg-gray-900/90 backdrop-blur-xl border border-white/10 rounded-xl p-2 shadow-2xl z-20 min-w-48 {className}"
>
    {@render children?.()}
</div>
