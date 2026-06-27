<script lang="ts">
    import type { Snippet } from "svelte";
    import { Spring } from "svelte/motion";
    interface ButtonProps {
        text: string;
        type?: "primary" | "secondary";
        children?: Snippet;
        size?: "small" | "medium" | "large";
        [key: string]: any;
    }
    let { text, type = "primary", children, size = "medium", ...props }: ButtonProps = $props();

    const buttonStyles = {
        primary:
            "bg-accent text-black shadow-[0_0_40px_rgba(var(--color-secondary),0.4)] hover:ring-2 focus:brightness-90",
        secondary:
            "bg-white/10 hover:bg-white/20 backdrop-blur-md text-light border border-white/10",
    };

    const buttonSizes = {
        small: "px-4 py-2 text-sm",
        medium: "px-6 py-3 text-base",
        large: "px-8 py-4 text-lg",
    };

    const scale = new Spring(1);

    const handleMouseDown = () => {
        scale.set(0.9);
    };

    const handleMouseUp = () => {
        setTimeout(() => {
            scale.set(1);
        }, 100);
    };
</script>

<button
    class="font-bold rounded-full flex items-center gap-3 transition-colors cursor-pointer {buttonSizes[size]} {buttonStyles[type]}"
    onmousedown={handleMouseDown}
    onmouseup={handleMouseUp}
    style:scale={scale.current}
    {...props}
>
    {#if children}
        <span class="w-5 h-5">{@render children()}</span>
    {/if}
    {text}
</button>
