<script lang="ts">
    let { value = $bindable(), onValueChange = () => {}, ...props } = $props();
    let slider: HTMLDivElement;

    function update(e: PointerEvent) {
        const rect = slider.getBoundingClientRect();

        value = Math.round(((e.clientX - rect.left) / rect.width) * 100);

        value = Math.max(0, Math.min(100, value));

        if (onValueChange) {
            onValueChange(value);
        }
    }

    function startDrag(e: PointerEvent) {
        (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
        update(e);
    }
</script>

<div
    bind:this={slider}
    class="w-full max-w-96 h-1.5 rounded-full bg-gray-400/40 relative cursor-pointer"
    title="Seek"
    role="slider"
    aria-valuemin="0"
    aria-valuemax="100"
    aria-valuenow={value}
    tabindex="0"
    onpointerdown={startDrag}
    onpointermove={(e) => e.buttons && update(e)}
    {...props}
>
    <div
        class="h-full rounded-full bg-accent absolute top-0 left-0 pl-1.5"
        style:width={`${value}%`}
    >
        <span class="absolute h-3 w-3 bg-accent right-0 rounded-full -mt-0.75"
        ></span>
    </div>
</div>
