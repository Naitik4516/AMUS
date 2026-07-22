<script>
    import { player } from "$lib/player.svelte";

    let { children } = $props();
</script>

{#if player.isReady}
    {@render children()}
{/if}
