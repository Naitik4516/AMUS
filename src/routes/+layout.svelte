<script lang="ts">
    import "../app.css";
    import { page } from "$app/stores";
    import Header from "../components/Header.svelte";
    import Player from "../components/Player.svelte";
    import Sidebar from "../components/Sidebar.svelte";
    import ScanProgress from "../components/ScanProgress.svelte";
    import { Toaster } from "$components/ui/sonner/index.js";
    import type { LayoutProps } from "./$types";
    import { player } from "$lib/player.svelte";
    import { afterNavigate } from "$app/navigation";

    let { children }: LayoutProps = $props();

    let scrollContainer: HTMLDivElement | undefined = $state();

    afterNavigate(() => {
        scrollContainer?.scrollTo(0, 0);
    });
</script>

<Header />
<ScanProgress />
<Sidebar />
<Toaster
    position="top-center"
    theme="dark"
    toastOptions={{
        unstyled: true,
        classes: {
            toast: "flex items-center bg-secondary/10 backdrop-blur-xl p-4 rounded-2xl w-full shadow-2xl border gap-2 transition-all pointer-events-auto mt-10",
            title: "font-bold",
            error: "bg-red-500/50 text-white",
            success: "bg-green-500/50 text-white",
        },
    }}
/>

<div
    bind:this={scrollContainer}
    class="flex flex-col w-screen h-screen overflow-y-auto pt-8 pl-30 {player.currentTrack
        ? 'pb-32'
        : ''}"
>
    <div>
        {@render children()}
    </div>
    {#if player.currentTrack}
        <Player />
    {/if}
</div>
