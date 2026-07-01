<script lang="ts">
    import { onMount } from "svelte";

    let { children } = $props();

    let container;
    let text;

    let shouldScroll = $state(false);
    let distance = $state(0);
    let duration = $derived(Math.max(11, distance / 15));

    function update() {
        if (!container || !text) return;

        shouldScroll = text.scrollWidth > container.clientWidth;
        distance = text.scrollWidth - container.clientWidth;
    }

    onMount(() => {
        update();

        const resize = new ResizeObserver(update);
        resize.observe(container);
        resize.observe(text);

        return () => resize.disconnect();
    });
</script>

<div bind:this={container} class="container">
    <span
        bind:this={text}
        class:scroll={shouldScroll}
        style="--distance: {distance}px; --duration: {duration}s;"
        class="inline-block px-1"
    >
        {@render children()}
    </span>
</div>

<style>
    .scroll {
        animation: marquee var(--duration) linear infinite;
        animation-delay: 2s;
    }

    .container {
        -webkit-mask-image: linear-gradient(
            to right,
            transparent 0,
            black 12px,
            black calc(100% - 12px),
            transparent 100%
        );

        mask-image: linear-gradient(
            to right,
            transparent 0,
            black 12px,
            black calc(100% - 12px),
            transparent 100%
        );
    }
    @keyframes marquee {
        0%,
        15% {
            transform: translateX(0);
        }

        45%,
        60% {
            transform: translateX(calc(var(--distance) * -1));
        }

        100% {
            transform: translateX(0);
        }
    }
</style>
