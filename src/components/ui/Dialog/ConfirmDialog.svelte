<script lang="ts">
    import Dialog from "$components/Dialog.svelte";
    import { Button } from "$components/ui/button/index.js";

    let {
        open = $bindable(false),
        title = "Confirm",
        message = "",
        confirmLabel = "Delete",
        onConfirm = () => {},
    }: {
        open: boolean;
        title?: string;
        message?: string;
        confirmLabel?: string;
        onConfirm?: () => void | Promise<void>;
    } = $props();

    let confirming = $state(false);
</script>

<Dialog bind:open {title}>
    <p class="text-zinc-300 text-sm leading-relaxed">{message}</p>

    {#snippet Footer()}
        <Button variant="ghost" onclick={() => (open = false)}>Cancel</Button>
        <Button
            variant="destructive"
            disabled={confirming}
            onclick={async () => {
                confirming = true;
                try {
                    await onConfirm();
                } finally {
                    confirming = false;
                    open = false;
                }
            }}
        >
            {confirmLabel}
        </Button>
    {/snippet}
</Dialog>
