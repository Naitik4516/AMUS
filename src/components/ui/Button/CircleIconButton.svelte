<script lang="ts">
    let { size = 3, type, onclick, children } = $props();

    let color = $derived.by(() => {
        switch (type) {
            case "primary":
                return "bg-secondary text-black hover:bg-secondary/80";
            default:
                return "bg-stone-800 text-text hover:bg-neutral-700/80";
        }
    });
</script>

<button
    class="flex items-center justify-center rounded-full transition-colors duration-300 {color}"
    style="width: {size}em; height: {size}em;"
    {onclick}
>
    {@render children()}
</button>
