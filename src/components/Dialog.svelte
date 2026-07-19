<script lang="ts">
    import { fly, blur } from "svelte/transition";
    import Button from "./ui/button/button.svelte";
    import { X } from "@lucide/svelte";

    let { open = $bindable(false), title, children, Footer } = $props();
</script>

{#if open}
    <div
        class="fixed inset-0 z-10 flex items-center justify-center p-4 bg-black/20"
        transition:blur={{ duration: 300 }}
    >
        <div
            class="bg-card border rounded-3xl p-6 w-full max-w-lg shadow-2xl flex flex-col"
            transition:fly={{ y: 600, duration: 300 }}
        >
            <div class="flex justify-between mb-6">
                <h2 class="text-2xl font-bold font-satoshi">{title}</h2>
                <Button
                    variant="outline"
                    size="icon"
                    onclick={() => (open = false)}
                >
                    <X />
                </Button>
            </div>
            {@render children()}
            {#if Footer}
                <div class="flex gap-4 justify-end">
                    {@render Footer()}
                </div>
            {/if}
        </div>
    </div>
{/if}
