<script lang="ts">
    import Button from "$components/ui/button/button.svelte";
    import Icon from "$components/ui/Icon.svelte";
    import { openUrl } from "@tauri-apps/plugin-opener";

    interface Props {
        error: string | null;
    }
    let { error }: Props = $props();

    let showDetails = $state(false);
    let resetting = $state(false);

    async function handleReset() {
        if (resetting) return;
        resetting = true;
        const { invoke } = await import("@tauri-apps/api/core");
        try {
            await invoke("reset_app_data");
        } catch (e) {
            console.error("reset failed:", e);
            resetting = false;
        }
    }

    async function handleRestart() {
        const { invoke } = await import("@tauri-apps/api/core");
        try {
            await invoke("plugin:process|restart");
        } catch (e) {
            console.error("restart failed:", e);
        }
    }
</script>

<div class="fixed inset-0 z-50 flex flex-col items-center justify-center p-8 bg-background">
    <div class="flex flex-col items-center text-center max-w-xl gap-6">
        <div class="w-16 h-16 rounded-full bg-red-500/20 flex items-center justify-center">
            <Icon name="triangle-alert" class="w-8 h-8 text-red-400" />
        </div>

        <h1 class="text-3xl font-bold text-foreground">Something went wrong</h1>
        <p class="text-muted-foreground text-lg leading-relaxed">
            AMUS encountered a problem during startup and could not load your music library.
        </p>

        {#if error}
            <button
                onclick={() => (showDetails = !showDetails)}
                class="text-sm text-muted-foreground hover:text-foreground underline underline-offset-2 transition-colors"
            >
                {showDetails ? "Hide details" : "Show error details"}
            </button>
            {#if showDetails}
                <pre class="w-full text-xs text-left text-muted-foreground bg-secondary/30 p-4 rounded-2xl overflow-auto max-h-48">{error}</pre>
            {/if}
        {/if}

        <div class="flex flex-wrap gap-3 justify-center mt-2">
            <Button
                size="lg"
                onclick={handleRestart}
            >
                Try Again
            </Button>

            <Button
                size="lg"
                variant="outline"
                disabled={resetting}
                onclick={handleReset}
            >
                {resetting ? "Resetting..." : "Reset App Data"}
            </Button>

            <Button
                size="lg"
                variant="ghost"
                onclick={() => openUrl("https://github.com/Naitik4516/AMUS/issues/new")}
            >
                Report Issue
            </Button>
        </div>

        <p class="text-sm text-muted-foreground mt-4 max-w-md">
            If nothing helps, wait for the next update or
            <a
                href="https://github.com/Naitik4516/AMUS/releases"
                target="_blank"
                rel="noreferrer"
                class="underline underline-offset-2 hover:text-foreground transition-colors"
            >downgrade to a previous stable release</a>.
        </p>
    </div>
</div>
