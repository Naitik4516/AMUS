<script lang="ts">
    let { title, children } = $props();
</script>

<h2 class="text-5xl font-extrabold text-white font-switzer mb-4">
    {title}
</h2>
<div
    class="relative flex gap-10 overflow-x-auto container scrollbar-none pl-2 w-full"
>
    {@render children()}
</div>

<style>
    .container {
        -webkit-mask-image: linear-gradient(
            to right,
            transparent 0,
            black 20px,
            black calc(100% - 20px),
            transparent 100%
        );

        mask-image: linear-gradient(
            to right,
            transparent 0,
            black 20px,
            black calc(100% - 20px),
            transparent 100%
        );
    }
</style>
