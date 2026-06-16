<script lang="ts">
    import "../app.css";
    import Header from "../components/Header.svelte";
    import Player from "../components/Player.svelte";
    import Sidebar from "../components/Sidebar.svelte";
    import ScanProgress from "../components/ScanProgress.svelte";
    import type { LayoutProps } from "./$types";

    let { children }: LayoutProps = $props();
</script>

<Header />
<ScanProgress />
<div class=" flex h-[calc(100vh-80px)] w-screen p-4">
    <Sidebar />
    <Player />
    {@render children()}
</div>
