<script lang="ts">
    import { scale } from "svelte/transition";

    interface Props {
        value?: number;
        onValueChange?: (val: number) => void;
        [key: string]: any;
    }

    let {
        value = $bindable(0),
        onValueChange = () => {},
        ...props
    }: Props = $props();

    let slider: HTMLDivElement;
    let hovering = $state(false);
    let dragging = $state(false);
    let dragValue = $state(0);

    let displayValue = $derived(dragging ? dragValue : value);

    function getValueFromPointer(e: PointerEvent): number {
        const rect = slider.getBoundingClientRect();
        const raw = (e.clientX - rect.left) / rect.width;
        return Math.max(0, Math.min(1, Math.round(raw * 100) / 100));
    }

    function startDrag(e: PointerEvent) {
        (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
        dragging = true;
        dragValue = getValueFromPointer(e);
    }

    function moveDrag(e: PointerEvent) {
        if (!dragging || !e.buttons) return;
        dragValue = getValueFromPointer(e);
    }

    function commitDrag(e: PointerEvent) {
        if (!dragging) return;
        dragValue = getValueFromPointer(e);
        dragging = false;
        value = dragValue;
        onValueChange(dragValue);
    }
</script>

<div
    bind:this={slider}
    class="w-full max-w-96 h-1.5 rounded-full bg-gray-400/40 relative cursor-pointer"
    title="Seek"
    role="slider"
    aria-valuemin="0"
    aria-valuemax="1"
    aria-valuenow={displayValue}
    tabindex="0"
    onpointerdown={startDrag}
    onpointermove={moveDrag}
    onpointerup={commitDrag}
    onpointercancel={commitDrag}
    onmouseenter={() => (hovering = true)}
    onmouseleave={() => (hovering = false)}
    {...props}
>
    <div
        class="{hovering
            ? 'h-2 bg-white'
            : 'h-full bg-gray-200'} rounded-full absolute top-0 left-0 pl-1.5 transition-all ease-out duration-200"
        style:width={`${displayValue * 100}%`}
        role="presentation"
    >
        {#if hovering || dragging}
            <span
                class="absolute bg-white right-0 rounded-full h-3 w-3 -mt-0.75"
                transition:scale
            ></span>
        {/if}
    </div>
</div>
