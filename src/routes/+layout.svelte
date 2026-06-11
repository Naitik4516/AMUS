<script lang="ts">
    import "../app.css";
    import Header from "../components/Header.svelte";
    import Player from "../components/Player.svelte";
    import Sidebar from "../components/Sidebar.svelte";
    import type { LayoutProps } from "./$types";

    let { children }: LayoutProps = $props();
</script>

<Header />
<div class="flex h-full w-screen p-4">
    <Sidebar />
    {@render children()}
</div>
<Player />
