<script lang="ts">
    import Banner from "$components/ui/Banner.svelte";
    import type { PageProps } from "./$types";

    let { data }: PageProps = $props();
</script>

<main class="container">
    <Banner hasMusic={data.hasMusic} />
</main>
