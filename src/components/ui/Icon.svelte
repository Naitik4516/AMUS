<script>
    import {
        CircleMinus,
        Shuffle,
        Play,
        Pause,
        Clock,
        Ellipse,
        ChevronUp,
        Plus,
        X,
        Trash,
    } from "@lucide/svelte";
    let { name, size = 16, class: className = "", ...props } = $props();

    let properties = $derived({
        size,
        class: className,
        ariaHidden: true,
        ...props,
    });
</script>

{#if name === "play"}
    <Play {...properties} />
{:else if name === "pause"}
    <Pause {...properties} />
{:else if name === "shuffle"}
    <Shuffle {...properties} />
{:else if name === "more-horizontal"}
    <Ellipse {...properties} />
{:else if name === "clock"}
    <Clock {...properties} />
{:else if name === "chevron-up"}
    <ChevronUp {...properties} />
{:else if name === "chevron-down"}
    <svg
        viewBox="0 0 24 24"
        width={size}
        height={size}
        class={className}
        fill="none"
        stroke="currentColor"
        stroke-width="2.2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
    >
        <path d="M5 9.5 12 16.5l7-7" />
    </svg>
{:else if name === "chevron-right"}
    <svg
        viewBox="0 0 24 24"
        width={size}
        height={size}
        class={className}
        fill="none"
        stroke="currentColor"
        stroke-width="2.2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
    >
        <path d="M9 5l7 7-7 7" />
    </svg>
{:else if name === "heart"}
    <svg
        viewBox="0 0 24 24"
        width={size}
        height={size}
        class={className}
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
    >
        <path
            d="M12 20.2s-7.4-4.5-9.9-9.1C.7 7.9 2.1 4.6 5.4 3.8c2-.5 3.9.3 5.1 1.9 1.1-1.6 3-2.4 5.1-1.9 3.2.8 4.6 4.1 2.9 7.3-2.5 4.6-5.9 9.1-5.9 9.1Z"
        />
    </svg>
{:else if name === "heart-filled"}
    <svg
        viewBox="0 0 24 24"
        width={size}
        height={size}
        class={className}
        fill="currentColor"
        aria-hidden="true"
    >
        <path
            d="M12 20.2s-7.4-4.5-9.9-9.1C.7 7.9 2.1 4.6 5.4 3.8c2-.5 3.9.3 5.1 1.9 1.1-1.6 3-2.4 5.1-1.9 3.2.8 4.6 4.1 2.9 7.3-2.5 4.6-5.9 9.1-5.9 9.1Z"
        />
    </svg>
{:else if name === "list-plus"}
    <svg
        viewBox="0 0 24 24"
        width={size}
        height={size}
        class={className}
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
    >
        <path d="M3.5 6.5h11M3.5 12h7.5M3.5 17.5h7.5" />
        <path d="M17.8 13.5v6M14.8 16.5h6" />
    </svg>
{:else if name === "skip-forward"}
    <svg
        viewBox="0 0 24 24"
        width={size}
        height={size}
        class={className}
        fill="currentColor"
        aria-hidden="true"
    >
        <path
            d="M5.5 5.8c0-1 1-1.6 1.9-1.1l8.4 5.2c.8.5.8 1.7 0 2.2l-8.4 5.2c-.9.5-1.9-.1-1.9-1.1V5.8Z"
        />
        <rect x="17" y="5" width="2.3" height="14" rx="0.6" />
    </svg>
{:else if name === "plus"}
    <Plus {...properties} />
{:else if name === "info"}
    <svg
        viewBox="0 0 24 24"
        width={size}
        height={size}
        class={className}
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
    >
        <circle cx="12" cy="12" r="9" />
        <path d="M12 11v5.2M12 7.8v.01" />
    </svg>
{:else if name === "user"}
    <svg
        viewBox="0 0 24 24"
        width={size}
        height={size}
        class={className}
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
    >
        <circle cx="12" cy="8.2" r="3.6" />
        <path d="M4.3 20c1-3.6 4.1-5.7 7.7-5.7s6.7 2.1 7.7 5.7" />
    </svg>
{:else if name === "disc"}
    <svg
        viewBox="0 0 24 24"
        width={size}
        height={size}
        class={className}
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
    >
        <circle cx="12" cy="12" r="9" />
        <circle cx="12" cy="12" r="2.4" />
    </svg>
{:else if name === "x"}
    <X {...properties} />
{:else if name === "sliders"}
    <svg
        viewBox="0 0 24 24"
        width={size}
        height={size}
        class={className}
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
    >
        <path
            d="M4 6.5h8M16.2 6.5H20M4 12h2.2M10.2 12H20M4 17.5h12.2M19.8 17.5H20"
        />
        <circle cx="13.2" cy="6.5" r="1.8" fill="currentColor" stroke="none" />
        <circle cx="8.2" cy="12" r="1.8" fill="currentColor" stroke="none" />
        <circle cx="18.2" cy="17.5" r="1.8" fill="currentColor" stroke="none" />
    </svg>
{:else if name === "trash"}
    <svg
        xmlns="http://www.w3.org/2000/svg"
        height={size}
        width={size}
        viewBox="0 -960 960 960"
        fill="currentColor"
        {...properties}
        ><path
            d="M280-120q-33 0-56.5-23.5T200-200v-520h-40v-80h200v-40h240v40h200v80h-40v520q0 33-23.5 56.5T680-120H280Zm400-600H280v520h400v-520ZM360-280h80v-360h-80v360Zm160 0h80v-360h-80v360ZM280-720v520-520Z"
        /></svg
    >
{:else if name === "edit"}
    <svg
        viewBox="0 0 24 24"
        width={size}
        height={size}
        class={className}
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
    >
        <path d="M4.5 19.5l.8-3.4L15.6 5.8l2.6 2.6L8 18.7l-3.5.8Z" />
    </svg>
{:else if name === "circle-minus"}
    <CircleMinus {...properties} />
{:else if name === "triangle-alert"}
    <svg
        viewBox="0 0 24 24"
        width={size}
        height={size}
        class={className}
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
    >
        <path
            d="M21.7 19.5 13 4.8c-.5-.9-1.7-.9-2.2 0L2.1 19.5c-.5.9.1 2 1.1 2h17.6c1 0 1.6-1.1 1.1-2Z"
        />
        <path d="M12 10.6v3.4" />
        <path d="M12 16.7v.01" />
    </svg>
{:else if name === "artist"}
    <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 -960 960 960"
        width={size}
        height={size}
        {...properties}
        ><path
            d="M629-189q-29-29-29-71t29-71q29-29 71-29 8 0 18 1.5t22 6.5v-168q0-17 11.5-28.5T780-560h60q17 0 28.5 11.5T880-520q0 17-11.5 28.5T840-480h-40v220q0 42-29 71t-71 29q-42 0-71-29ZM327-527q-47-47-47-113t47-113q47-47 113-47t113 47q47 47 47 113t-47 113q-47 47-113 47t-113-47ZM160-160q-17 0-28.5-11.5T120-200v-72q0-35 17.5-63t46.5-43q62-31 126-46.5T440-440q28 0 55.5 3t55.5 9q17 4 21.5 21t-9.5 31q-21 25-31.5 54.5T521-260q0 13 1.5 25.5T528-209q5 18-4.5 33.5T497-160H160Z"
        /></svg
    >
{/if}
