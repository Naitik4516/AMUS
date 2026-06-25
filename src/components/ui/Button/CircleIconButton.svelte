<script lang="ts">
    let { size = 3, type, onclick, children } = $props();

    let color = $derived.by(() => {
        switch (type) {
            case "primary":
                return "bg-accent text-black hover:bg-accent/80";
            default:
                return "bg-zinc-800 text-foreground hover:bg-neutral-700/80";
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
