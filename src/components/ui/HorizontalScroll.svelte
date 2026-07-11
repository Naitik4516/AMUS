<script lang="ts">
    import { onMount } from "svelte";
    import { gsap } from "gsap";

    let { title, data, Card } = $props();

    let scrollContainer: HTMLDivElement | null = $state(null);
    let showLeftButton = $state(false);
    let showRightButton = $state(false);

    let isDown = false;
    let startX = 0;
    let scrollLeftStart = 0;

    let lastX = 0;
    let velocity = 0;

    const updateScrollState = () => {
        if (!scrollContainer) return;
        const { scrollLeft, scrollWidth, clientWidth } = scrollContainer;
        showLeftButton = scrollLeft > 5;
        showRightButton = scrollLeft + clientWidth < scrollWidth - 5;
    };

    const scroll = (direction: "left" | "right") => {
        if (!scrollContainer) return;

        const offset = scrollContainer.clientWidth * 0.75;
        const targetScroll =
            direction === "left"
                ? scrollContainer.scrollLeft - offset
                : scrollContainer.scrollLeft + offset;

        gsap.to(scrollContainer, {
            scrollLeft: targetScroll,
            duration: 0.6,
            ease: "power2.out",
            onUpdate: updateScrollState,
        });
    };

    const handleMouseDown = (e: MouseEvent) => {
        if (!scrollContainer) return;
        if (e.button !== 0) return;
        isDown = true;

        gsap.killTweensOf(scrollContainer);

        scrollContainer.classList.add("active");
        startX = e.clientX;
        lastX = e.clientX;
        velocity = 0; // Reset momentum
        scrollLeftStart = scrollContainer.scrollLeft;

        e.preventDefault();
    };

    const handleMouseMove = (e: MouseEvent) => {
        if (!isDown || !scrollContainer) return;
        e.preventDefault();

        const currentX = e.clientX;
        const walk = currentX - startX;

        velocity = currentX - lastX;
        lastX = currentX;

        scrollContainer.scrollLeft = scrollLeftStart - walk;
        updateScrollState();
    };

    const handleMouseLeaveOrUp = () => {
        if (!isDown) return;
        isDown = false;

        if (scrollContainer) {
            scrollContainer.classList.remove("active");

            if (Math.abs(velocity) > 1) {
                const momentumDestination =
                    scrollContainer.scrollLeft - velocity * 12;

                gsap.to(scrollContainer, {
                    scrollLeft: momentumDestination,
                    duration: 0.75,
                    ease: "power3.out",
                    onUpdate: updateScrollState,
                });
            }
        }
    };

    onMount(() => {
        updateScrollState();
        window.addEventListener("resize", updateScrollState);
        return () => window.removeEventListener("resize", updateScrollState);
    });
</script>

<div class="relative w-full px-5">
    <h2 class="text-5xl font-extrabold text-white font-switzer mb-4">
        {title}
    </h2>
    <div
        class="mask-container"
        class:hide-left={!showLeftButton}
        class:hide-right={!showRightButton}
    >
        {#if showLeftButton}
            <button
                class="nav-btn left"
                onclick={() => scroll("left")}
                aria-label="Scroll left"
            >
                ‹
            </button>
        {/if}

        <div
            bind:this={scrollContainer}
            class="scroll-container pl-2"
            onscroll={updateScrollState}
            onmousedown={handleMouseDown}
            onmouseleave={handleMouseLeaveOrUp}
            onmouseup={handleMouseLeaveOrUp}
            onmousemove={handleMouseMove}
        >
            {#each data as item (item.id)}
                <div class="scroll-item">
                    <Card data={item} />
                </div>
            {/each}
        </div>

        {#if showRightButton}
            <button
                class="nav-btn right"
                onclick={() => scroll("right")}
                aria-label="Scroll right"
            >
                ›
            </button>
        {/if}
    </div>
</div>

<style>
    .mask-container {
        position: relative;
        width: 100%;
        --mask-left: linear-gradient(to right, transparent, #000 40px);
        --mask-right: linear-gradient(to left, transparent, #000 40px);

        mask-image: var(--mask-left), var(--mask-right);
        mask-composite: intersect;
        -webkit-mask-image: var(--mask-left), var(--mask-right);
        -webkit-mask-composite: source-in;
    }

    .mask-container.hide-left {
        mask-image: var(--mask-right);
        -webkit-mask-image: var(--mask-right);
    }

    .mask-container.hide-right {
        mask-image: var(--mask-left);
        -webkit-mask-image: var(--mask-left);
    }

    .mask-container.hide-left.hide-right {
        mask-image: none;
        -webkit-mask-image: none;
    }

    .scroll-container {
        display: flex;
        gap: 32px;
        overflow-x: auto;
        padding: 10px 20px;
        scrollbar-width: none;
        cursor: grab;
        user-select: none;
        -webkit-user-select: none;
        scroll-behavior: auto !important;
    }

    .scroll-container::-webkit-scrollbar {
        display: none;
    }

    .scroll-item {
        flex: 0 0 auto;
        pointer-events: auto;
    }

    .nav-btn {
        position: absolute;
        top: 50%;
        transform: translateY(-50%);
        z-index: 10;
        width: 50px;
        height: 50px;
        border-radius: 50%;
        background-color: rgba(0, 0, 0, 0.8);
        color: #fff;
        border: gray;
        font-size: 24px;
        line-height: 1;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        transition:
            background-color 0.2s,
            transform 0.2s;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
    }

    .nav-btn:hover {
        background-color: rgba(0, 0, 0, 0.9);
        transform: translateY(-50%) scale(1.05);
    }

    .left {
        left: 10px;
    }
    .right {
        right: 10px;
    }
</style>
