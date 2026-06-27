<script lang="ts">
    let {
        checked = $bindable(false),
        disabled = false,
        onchange,
    }: {
        checked?: boolean;
        disabled?: boolean;
        onchange?: (checked: boolean) => void;
    } = $props();

    function handleClick() {
        if (disabled) return;
        checked = !checked;
        onchange?.(checked);
    }
</script>

<button
    onclick={handleClick}
    disabled={disabled}
    class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed {checked
        ? 'bg-accent'
        : 'bg-neutral-700'}"
    title={checked ? 'Disable' : 'Enable'}
    type="button"
    role="switch"
    aria-checked={checked}
>
    <span
        class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {checked
            ? 'translate-x-6'
            : 'translate-x-1'}"
    ></span>
</button>
