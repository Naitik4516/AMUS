<script lang="ts">
    import "../app.css";
    import Header from "../components/Header.svelte";
    import Player from "../components/Player.svelte";
    import Sidebar from "../components/Sidebar.svelte";
    import ScanProgress from "../components/ScanProgress.svelte";
    import { Toaster } from "$components/ui/sonner/index.js";
    import type { LayoutProps } from "./$types";
    import { player } from "$lib/player.svelte";
    import { afterNavigate } from "$app/navigation";
    import { onMount } from "svelte";
    import { settings, flags, initSettings } from "$lib/settings.svelte";
    import { updater } from "$lib/update.svelte";
    import { toast } from "svelte-sonner";
    import LocomotiveScroll from "locomotive-scroll";
    import "locomotive-scroll/locomotive-scroll.css";

    let { children }: LayoutProps = $props();

    let scrollContainer: HTMLDivElement | undefined = $state();
    let scrollContent: HTMLDivElement | undefined = $state();
    let scrollInstance: LocomotiveScroll | undefined;

    $effect(() => {
        if (!flags.ready) return;
        document.documentElement.classList.toggle(
            "smooth-scroll",
            settings.useLocomotiveScroll,
        );
    });

    function initScroll() {
        scrollInstance?.destroy();
        scrollInstance = new LocomotiveScroll({
            lenisOptions: {
                wrapper: scrollContainer,
                content: scrollContent,
                duration: 1.2,
                smoothWheel: true,
                orientation: "vertical",
                gestureOrientation: "vertical",
            },
            autoStart: true,
        });
    }

    function destroyScroll() {
        scrollInstance?.destroy();
        scrollInstance = undefined;
    }

    $effect(() => {
        if (!flags.ready) return;
        if (settings.useLocomotiveScroll && scrollContainer && scrollContent) {
            initScroll();
        } else {
            destroyScroll();
        }
    });

    onMount(() => {
        initSettings();

        if (settings.autoCheckUpdates) {
            updater
                .checkForUpdates()
                .then((found) => {
                    if (found && updater.updateAvailable) {
                        toast(
                            `Update v${updater.updateAvailable.version} available`,
                            {
                                description:
                                    updater.updateAvailable.body ??
                                    "A new version is ready to install.",
                                action: {
                                    label: "Install",
                                    onClick: async () => {
                                        await updater.downloadAndInstall();
                                    },
                                },
                                duration: 10000,
                            },
                        );
                    }
                })
                .catch((error) => {
                    console.error("Error checking for updates:", error);
                });
        }
    });

    afterNavigate(() => {
        if (!scrollInstance || !scrollContainer) return;
        scrollInstance.scrollTo(0, { immediate: true });
        scrollInstance.addScrollElements(scrollContainer);
        scrollInstance.resize();
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
    class="w-screen h-screen {flags.ready && settings.useLocomotiveScroll
        ? 'overflow-hidden'
        : 'overflow-y-auto'}"
>
    <div
        bind:this={scrollContent}
        class="pt-8 pl-30 {player.currentTrack ? 'pb-32' : ''}"
    >
        <div>
            {@render children()}
        </div>
    </div>
</div>
{#if player.currentTrack}
    <Player />
{/if}
