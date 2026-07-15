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
    } from "@lucide/svelte";
    let { name, size = 16, class: className = "" } = $props();

    let properties = $derived({ name, size, className, ariaHidden: true });
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
            d="M5 7h14M9.5 7V5.2c0-.7.5-1.2 1.2-1.2h4.6c.7 0 1.2.5 1.2 1.2V7m-8.7 0 1 12.3c.1.8.7 1.4 1.5 1.4h6.4c.8 0 1.4-.6 1.5-1.4L17.7 7"
        />
        <path d="M10.3 11v6M13.7 11v6" />
    </svg>
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
        <path d="M21.7 19.5 13 4.8c-.5-.9-1.7-.9-2.2 0L2.1 19.5c-.5.9.1 2 1.1 2h17.6c1 0 1.6-1.1 1.1-2Z" />
        <path d="M12 10.6v3.4" />
        <path d="M12 16.7v.01" />
    </svg>
{/if}
