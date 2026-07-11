<script>
    import { onMount, onDestroy } from "svelte";
    import { player } from "$lib/player.svelte";
    import { store } from "$lib/stores.svelte";
    import { appDataDir } from "@tauri-apps/api/path";

    onMount(() => {
        player.init();
        appDataDir().then((dir) => {
            store.appDataDirPath = dir;
        });
    });
    onDestroy(() => {
        player.destroy();
    });

    let { children } = $props();
</script>

{@render children()}
