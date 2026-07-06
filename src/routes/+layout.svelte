<script>
    import { onMount, onDestroy } from "svelte";
    import { player } from "$lib/player.svelte";

    onMount(() => {
        player.init();
    });
    onDestroy(() => {
        player.destroy();
    });

    let { children } = $props();
</script>

{@render children()}
