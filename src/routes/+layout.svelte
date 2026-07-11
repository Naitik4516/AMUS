<script>
    import { onMount, onDestroy } from "svelte";
    import { player } from "$lib/player.svelte";
    import { store } from "$lib/stores.svelte";

    onMount(() => {
        player.init();
        // Start resolving app data dir as early as possible so cover art
        // URLs can be built; store.init() also awaits this idempotently.
        store.ensureAppDataDir().catch((e) => {
            console.error("Failed to resolve app data dir:", e);
        });
    });
    onDestroy(() => {
        player.destroy();
    });

    let { children } = $props();
</script>

{@render children()}
