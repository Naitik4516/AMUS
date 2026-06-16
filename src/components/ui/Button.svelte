<script lang="ts">
    import type { Snippet } from "svelte";

    interface ButtonProps {
        text: string;
        onClick: () => void;
        type?: "primary" | "secondary";
        children?: Snippet;
    }
    let {
        text,
        type = "primary",
        onClick = () => {},
        children,
    }: ButtonProps = $props();

    const buttonStyles = {
        primary:
            "bg-secondary text-dark shadow-[0_0_40px_rgba(var(--color-secondary),0.4)]",
        secondary:
            "bg-white/10 hover:bg-white/20 backdrop-blur-md text-light border border-white/10",
    };
</script>

<button
    class={"font-bold p-4 md:px-8 md:py-4 rounded-full flex items-center gap-3 hover:scale-105 transition-transform cursor-pointer " +
        buttonStyles[type]}
    onclick={onClick}
>
    {#if children}
        <span class="w-5 h-5">{@render children()}</span>
    {/if}
    {text}
</button>
