<script lang="ts">
    import Header from "$components/Header.svelte";
    import Player from "$components/Player.svelte";
    import ScanProgress from "$components/ScanProgress.svelte";
    import Sidebar from "$components/Sidebar.svelte";
    import { Toaster } from "$components/ui/sonner/index.js";
    import { player } from "$lib/player.svelte";
    import { flags, initSettings, settings } from "$lib/settings.svelte";
    import { installHandlers } from "$lib/shortcut-handler.svelte";
    import {
        findAction,
        getHandler,
        globalShortcutFlags,
        initShortcuts,
    } from "$lib/shortcuts.svelte";
    import { store } from "$lib/stores.svelte";
    import { updater } from "$lib/update.svelte";
    import { listen } from "@tauri-apps/api/event";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { onMount } from "svelte";
    import { toast } from "svelte-sonner";
    import "../../app.css";
    import "../../styles/main.css";
    import type { LayoutProps } from "./$types";
    import { afterNavigate } from "$app/navigation";
    import { gsap } from "gsap";
    import { ScrollTrigger } from "gsap/ScrollTrigger";
    import Lenis from "lenis";
    import "lenis/dist/lenis.css";
    import type { Action } from "svelte/action";
    import Button from "$components/ui/button/button.svelte";
    import { MoveUp } from "@lucide/svelte";
    import { slide } from "svelte/transition";

    type ResizeDirection =
        | "East"
        | "North"
        | "NorthEast"
        | "NorthWest"
        | "South"
        | "SouthEast"
        | "SouthWest"
        | "West";

    const SCROLL_THRESHOLD = 400;

    let { children }: LayoutProps = $props();

    let isMaximized = $state(false);
    let showFAB = $state(false);

    let scrollContainer: Element | undefined = $state();
    let lenis: Lenis;

    $effect(() => {
        if (flags.ready && player.isReady) {
            player.setAutoplay(settings.autoplayEnabled);
        }
    });

    const setupSmoothScroll: Action = (node) => {
        $effect(() => {
            if (!scrollContainer) {
                console.warn("Scroll container not found for smooth scrolling");
                return;
            }
            lenis = new Lenis({
                wrapper: scrollContainer,
                content: node,
                lerp: 0.1,
                duration: 1.2,
                prevent: (node) => {
                    return node.classList.contains("vlist");
                },
            });

            lenis.on("scroll", (e) => {
                ScrollTrigger.update();
                showFAB = e.scroll > SCROLL_THRESHOLD;
            });

            const updateTick = (time: number) => {
                lenis.raf(time * 1000);
            };
            gsap.ticker.add(updateTick);
            gsap.ticker.lagSmoothing(0);

            ScrollTrigger.scrollerProxy(scrollContainer, {
                scrollTop(value) {
                    if (arguments.length) {
                        lenis.scrollTo(value!, { immediate: true });
                    }
                    return lenis.scroll;
                },

                getBoundingClientRect() {
                    return {
                        top: 0,
                        left: 0,
                        width: window.innerWidth,
                        height: window.innerHeight,
                    };
                },
            });

            ScrollTrigger.addEventListener("refresh", () => lenis.resize());
            ScrollTrigger.refresh();

            return () => {
                lenis.destroy();
                gsap.ticker.remove(updateTick);
            };
        });
    };

    const scrollToTop = () => {
        if (scrollContainer && lenis) {
            lenis.scrollTo(0);
        }
    };

    function startResize(direction: ResizeDirection, e: MouseEvent) {
        e.preventDefault();
        e.stopPropagation();
        getCurrentWindow().startResizeDragging(direction);
    }

    $effect(() => {
        let active = true;
        let cleanupGlobal: (() => void) | undefined;

        const handler = (e: KeyboardEvent) => {
            const action = findAction(e);
            if (action) {
                e.preventDefault();
                e.stopPropagation();
                const fn = getHandler(action.id);
                fn?.();
            }
        };

        const unlistenMouseBack = (e: MouseEvent) => {
            if (e.button === 3) {
                history.back();
            }
            if (e.button === 4) {
                history.forward();
            }
        };

        initShortcuts().then(() => {
            if (!active) return;
            installHandlers();

            window.addEventListener("keydown", handler);
            window.addEventListener("mouseup", unlistenMouseBack);

            listen<string>("global-shortcut", (ev) => {
                if (
                    ev.payload.startsWith("global_") &&
                    globalShortcutFlags[ev.payload] !== true
                ) {
                    return;
                }
                const fn = getHandler(ev.payload);
                fn?.();
            }).then((unlisten) => {
                if (!active) {
                    unlisten();
                } else {
                    cleanupGlobal = unlisten;
                }
            });
        });

        return () => {
            active = false;
            window.removeEventListener("keydown", handler);
            window.removeEventListener("mouseup", unlistenMouseBack);
            if (cleanupGlobal) cleanupGlobal();
        };
    });

    onMount(() => {
        gsap.registerPlugin(ScrollTrigger);

        window.onunhandledrejection = (e) => {
            console.error("Unhandled rejection:", e.reason);
            toast.error("An unexpected error occurred");
        };

        initSettings();
        store.init();

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
        if (scrollContainer) {
            lenis.scrollTo(0, { immediate: true });
        }
    });
</script>

<Sidebar />
<Header bind:isMaximized />
<ScanProgress />
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
    class="w-full h-screen overflow-hidden pt-5 bg-radial from-background to-neutral-950 {isMaximized
        ? 'rounded-none'
        : 'rounded-3xl'}"
>
    <div
        bind:this={scrollContainer}
        class="main-scroller h-full overflow-y-auto {isMaximized
            ? 'mr-1'
            : 'mb-1.5 mr-1.5'}"
    >
        <div
            use:setupSmoothScroll
            class="pt-18 pl-30 {player.currentTrack ? 'pb-32' : ''}"
        >
            {@render children()}
        </div>
    </div>
</div>

{#if showFAB}
    <div
        class="fixed {player.currentTrack
            ? 'bottom-30'
            : 'bottom-5'} right-6 z-50"
        transition:slide
    >
        <Button variant="outline" onclick={scrollToTop} size="icon-xl">
            <MoveUp />
        </Button>
    </div>
{/if}

{#if !isMaximized}
    <div
        role="presentation"
        class="fixed top-0 left-0 right-0 h-1.5 cursor-s-resize z-999"
        onmousedown={(e) => startResize("North", e)}
    ></div>
    <div
        role="presentation"
        class="fixed bottom-0 left-0 right-0 h-1.5 cursor-s-resize z-999"
        onmousedown={(e) => startResize("South", e)}
    ></div>
    <div
        role="presentation"
        class="fixed top-0 right-0 bottom-0 w-1.5 cursor-e-resize z-999"
        onmousedown={(e) => startResize("East", e)}
    ></div>
    <div
        role="presentation"
        class="fixed top-0 left-0 bottom-0 w-1.5 cursor-w-resize z-999"
        onmousedown={(e) => startResize("West", e)}
    ></div>
    <div
        role="presentation"
        class="fixed bottom-0 right-0 w-4 h-4 cursor-se-resize z-999"
        onmousedown={(e) => startResize("SouthEast", e)}
    ></div>
{/if}

{#if player.currentTrack}
    <Player />
{/if}
