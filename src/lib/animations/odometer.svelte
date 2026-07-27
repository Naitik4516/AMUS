<script lang="ts">
    import { gsap } from "gsap";
    import { tick } from "svelte";

    let {
        value = 0,
        duration = 1,
        stagger = 0.05,
        ease = "power2.out",
        fontSize = 20,
        fontWeight = 500,
        color = "inherit",
        class: className = "",
    } = $props();

    function toWholeString(n: number | string) {
        const num = Math.max(0, Math.trunc(Number(n) || 0));
        return num.toString();
    }

    let digitStr = $derived(toWholeString(value));
    let digits = $derived(digitStr.split("").map(Number));

    let rowHeight = $derived(fontSize * 1.2);

    let stripEls: HTMLElement[] = $state([]);
    let prevDigits: number[] = [];

    async function animateTo(newDigits: number[]) {
        await tick();

        newDigits.forEach((d, i) => {
            const el = stripEls[i];
            if (!el) return;

            const targetY = -(d * rowHeight);

            const isNewColumn = prevDigits[i] === undefined;
            if (isNewColumn) {
                gsap.set(el, { y: targetY });
                gsap.from(el, {
                    opacity: 0,
                    duration: duration * 0.6,
                    ease,
                });
                return;
            }

            gsap.to(el, {
                y: targetY,
                duration,
                ease,
                delay: i * stagger,
            });
        });

        prevDigits = newDigits;
    }

    $effect(() => {
        animateTo(digits);
    });
</script>

<div
    class="odometer {className}"
    style:font-size="{fontSize}px"
    style:font-weight={fontWeight}
    style:color
    style:height="{rowHeight}px"
    style:line-height="{rowHeight}px"
>
    {#each digits as digit, i (digitStr.length - i)}
        <div class="odometer-col" style:width="{fontSize * 0.62}px">
            <div class="odometer-strip" bind:this={stripEls[i]}>
                {#each Array(10) as _, n}
                    <div class="odometer-digit" style:height="{rowHeight}px">
                        {n}
                    </div>
                {/each}
            </div>
        </div>
    {/each}
</div>

<style>
    .odometer {
        display: inline-flex;
        overflow: hidden;
        font-variant-numeric: tabular-nums;
        font-family: inherit;
        white-space: nowrap;
        vertical-align: text-bottom;
    }

    .odometer-col {
        position: relative;
        height: 100%;
        overflow: hidden;
        flex-shrink: 0;
    }

    .odometer-strip {
        position: absolute;
        top: 0;
        left: 0;
        display: flex;
        flex-direction: column;
        width: 100%;
        will-change: transform;
    }

    .odometer-digit {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 100%;
    }
</style>
