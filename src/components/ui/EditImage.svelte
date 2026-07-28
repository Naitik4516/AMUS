<script lang="ts">
    import { Pen, X } from "@lucide/svelte";
    import Button from "./button/button.svelte";

    interface EditImageProps {
        onclick: () => void;
        removeCover?: () => void;
        class?: string;
        children: () => any;
    }

    let {
        onclick,
        removeCover,
        class: className,
        children,
    }: EditImageProps = $props();
</script>

<div class="relative group {className}">
    {@render children()}
    <div
        class="absolute bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center inset-0"
    >
        <button {onclick}>
            <Pen size={28} fill="white" />
        </button>

        {#if removeCover}
            <Button
                variant="ghost"
                size="icon-sm"
                onclick={removeCover}
                class="absolute top-1 right-1"
            >
                <X />
            </Button>
        {/if}
    </div>
</div>
