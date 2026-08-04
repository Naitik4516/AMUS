<script lang="ts">
    import { onMount } from "svelte";
    import gsap from "gsap";

    let { children } = $props();

    let container: Element | null = null;
    let text: Element | null = null;

    let shouldScroll = $state(false);
    let distance = $state(0);
    let duration = $derived(Math.max(5, distance / 30));
    let endDelay = $derived(Math.max(0.3, Math.min(10, 1000 / distance)));

    let tl: gsap.core.Timeline | null = null;

    function startAnimation() {
        if (!text || distance <= 0) return;
        tl?.kill();

        tl = gsap
            .timeline({ repeat: -1, delay: 2 })
            .to(text, { duration: 0, delay: endDelay })
            .to(text, { x: -distance, duration, ease: "none" })
            .to(text, { duration: 0, delay: endDelay })
            .to(text, { x: 0, duration, ease: "none" });
    }

    function pause() {
        tl?.pause();
    }

    function resume() {
        tl?.resume();
    }

    function update() {
        if (!container || !text) return;

        shouldScroll = text.scrollWidth > container.clientWidth;
        distance = text.scrollWidth - container.clientWidth;

        if (shouldScroll) {
            startAnimation();
        } else {
            tl?.kill();
            if (text) gsap.set(text, { x: 0 });
        }
    }

    onMount(() => {
        if (!container || !text) return;
        update();

        const resize = new ResizeObserver(update);
        resize.observe(container);
        resize.observe(text);

        return () => {
            resize.disconnect();
            tl?.kill();
        };
    });
</script>

<div
    bind:this={container}
    onmouseenter={pause}
    onmouseleave={resume}
    role="presentation"
    class="w-full mask-x-from-95%"
>
    <span bind:this={text} class="inline-block px-2" aria-hidden="true">
        {@render children()}
    </span>
</div>
