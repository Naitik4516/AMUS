<script lang="ts">
    let { title, children } = $props();
</script>

<h2 class="text-4xl font-extrabold text-white font-switzer mb-4">
    {title}
</h2>
<div class="relative flex gap-10 overflow-x-auto scrollbar-hide px-2">
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
