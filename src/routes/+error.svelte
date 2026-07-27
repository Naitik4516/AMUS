<script>
    import { page } from "$app/stores";
    import Button from "$components/ui/button/button.svelte";

    let error = $derived($page.error);
    let status = $derived($page.status);
</script>

<div
    class="flex flex-col items-center justify-center min-h-screen p-8 text-center bg-red-950"
>
    <div class="max-w-lg">
        <h1 class="text-9xl font-black text-red-400 tracking-tight">
            {status}
        </h1>
        <p class="text-xl mt-4 text-muted-foreground">
            {#if status === 404}
                Page not found
            {:else}
                Something went wrong while loading this page
            {/if}
        </p>

        {#if error?.message}
            <details class="mt-6 text-left">
                <summary
                    class="cursor-pointer text-sm text-muted-foreground hover:text-foreground transition-colors"
                >
                    Details
                </summary>
                <pre
                    class="mt-2 text-sm font-mono text-muted-foreground bg-secondary/30 p-4 rounded-2xl overflow-auto max-h-64">{error.message}</pre>
            </details>
        {/if}

        <div class="flex gap-2 justify-center mt-8">
            <Button
                onclick={() => window.location.reload()}
                size="lg"
                class="font-bold">Try Again</Button
            >
            <Button variant="outline" onclick={() => history.back()} size="lg">
                Go Back
            </Button>
        </div>
    </div>
</div>
