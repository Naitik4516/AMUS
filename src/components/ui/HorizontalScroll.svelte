<script lang="ts">
    let { title, children } = $props();
</script>

<h2 class="text-5xl font-extrabold text-white font-switzer mb-4">
    {title}
</h2>
<div
    class="relative flex gap-10 justify-evenly overflow-x-auto scrollbar-hide px-2 w-full"
>
    {@render children()}
</div>

<style>
    .scrollbar-hide::-webkit-scrollbar {
        display: none;
    }
    .scrollbar-hide {
        -ms-overflow-style: none;
        scrollbar-width: none;
    }
</style>
