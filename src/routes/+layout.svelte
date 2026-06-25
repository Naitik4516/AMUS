<script lang="ts">
    import "../app.css";
    import { page } from "$app/stores";
    import Header from "../components/Header.svelte";
    import Player from "../components/Player.svelte";
    import Sidebar from "../components/Sidebar.svelte";
    import ScanProgress from "../components/ScanProgress.svelte";
    import ToastPortal from "../components/ui/ToastPortal.svelte";
    import type { LayoutProps } from "./$types";
    import { player } from "$lib/player.svelte";
    let { children }: LayoutProps = $props();

    let isPopup = $derived($page.url.pathname === "/popup");
</script>

{#if !isPopup}
    <Header />
    <ScanProgress />
    <Sidebar />
    <ToastPortal />
{/if}
<div
    class="flex flex-col w-screen overflow-y-auto"
    class:h-[calc(100vh-80px)]={!isPopup}
    class:h-screen={isPopup}
    class:p-4={!isPopup}
    class:pl-34={!isPopup}
>
    <div>
        {@render children()}
    </div>
    {#if !isPopup && player.currentTrack}
        <Player />
        <div class="h-100"></div>
    {/if}
</div>
