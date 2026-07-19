<script>
    import { onMount, onDestroy } from "svelte";
    import { player } from "$lib/player.svelte";
    import { store } from "$lib/stores.svelte";
    import { startup } from "$lib/startup.svelte";
    import StartupError from "$lib/components/StartupError.svelte";

    onMount(async () => {
        await startup.check();
        if (startup.error) return;

        player.init();
        store.ensureAppDataDir().catch((e) => {
            console.error("Failed to resolve app data dir:", e);
        });
    });
    onDestroy(() => {
        player.destroy();
    });

    let { children } = $props();
</script>

{#if startup.checked && startup.error}
    <StartupError error={startup.error} />
{:else if !startup.checked}
    <div class="fixed inset-0 flex items-center justify-center bg-black">
        <div
            class="w-8 h-8 border-2 border-muted-foreground/30 border-t-foreground rounded-full animate-spin"
        ></div>
    </div>
{:else}
    {@render children()}
{/if}
