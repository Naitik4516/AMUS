<script lang="ts">
    import "../../app.css";
    import "../../styles/main.css";
    import Header from "$components/Header.svelte";
    import Player from "$components/Player.svelte";
    import Sidebar from "$components/Sidebar.svelte";
    import ScanProgress from "$components/ScanProgress.svelte";
    import { Toaster } from "$components/ui/sonner/index.js";
    import type { LayoutProps } from "./$types";
    import { player } from "$lib/player.svelte";
    import { afterNavigate } from "$app/navigation";
    import { getCurrentWindow, PhysicalSize } from "@tauri-apps/api/window";
    import { onMount } from "svelte";
    import { settings, flags, initSettings } from "$lib/settings.svelte";
    import { updater } from "$lib/update.svelte";
    import { toast } from "svelte-sonner";
    import LocomotiveScroll from "locomotive-scroll";
    import "locomotive-scroll/locomotive-scroll.css";
    import { listen } from "@tauri-apps/api/event";
    import {
        initShortcuts,
        findAction,
        getHandler,
        globalShortcutFlags,
    } from "$lib/shortcuts.svelte";
    import { installHandlers } from "$lib/shortcut-handler.svelte";
    import ShortcutSettingsModal from "$components/shortcuts/ShortcutSettingsModal.svelte";
    let shortcutModalOpen = $state(false);
    $effect(() => {
        function handler() { shortcutModalOpen = true; }
        window.addEventListener("open-shortcut-settings", handler);
        return () => window.removeEventListener("open-shortcut-settings", handler);
    });

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

    $effect(() => {
        let active = true;
        let cleanupGlobal: (() => void) | undefined;

        const handler = (e: KeyboardEvent) => {
            if (shortcutModalOpen) return;
            const action = findAction(e);
            if (action) {
                e.preventDefault();
                e.stopPropagation();
                const fn = getHandler(action.id);
                fn?.();
            }
        };

        const unlistenMouseBack = (e: MouseEvent) => {
            if (e.button === 3) { history.back(); }
            if (e.button === 4) { history.forward(); }
        };

        initShortcuts().then(() => {
            if (!active) return;
            installHandlers();

            window.addEventListener("keydown", handler);
            window.addEventListener("mouseup", unlistenMouseBack);

            listen<string>("global-shortcut", (ev) => {
                if (ev.payload.startsWith("global_") && globalShortcutFlags[ev.payload] !== true) {
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

    const MIN_W = 700;
    const MIN_H = 700;

    function startResize(edge: string, e: MouseEvent) {
        e.preventDefault();
        e.stopPropagation();
        const appWindow = getCurrentWindow();
        const startX = e.screenX;
        const startY = e.screenY;

        appWindow.outerSize().then((startSize) => {
            function onMouseMove(e: MouseEvent) {
                const dx = e.screenX - startX;
                const dy = e.screenY - startY;
                let newW = startSize.width;
                let newH = startSize.height;
                if (edge.includes("right"))
                    newW = Math.max(MIN_W, startSize.width + dx);
                if (edge.includes("bottom"))
                    newH = Math.max(MIN_H, startSize.height + dy);
                appWindow.setSize(new PhysicalSize(newW, newH));
            }
            function onMouseUp() {
                document.removeEventListener("mousemove", onMouseMove);
                document.removeEventListener("mouseup", onMouseUp);
            }
            document.addEventListener("mousemove", onMouseMove);
            document.addEventListener("mouseup", onMouseUp);
        });
    }
</script>

<div class="flex flex-col h-screen w-screen">
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
        class="w-screen flex-1 min-h-0 {flags.ready &&
        settings.useLocomotiveScroll
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

    <div
        role="presentation"
        class="fixed bottom-0 left-0 right-0 h-1.5 cursor-s-resize z-999"
        onmousedown={(e) => startResize("bottom", e)}
    ></div>
    <div
        role="presentation"
        class="fixed top-0 right-0 bottom-0 w-1.5 cursor-e-resize z-999"
        onmousedown={(e) => startResize("right", e)}
    ></div>
    <div
        role="presentation"
        class="fixed bottom-0 right-0 w-4 h-4 cursor-se-resize z-999"
        onmousedown={(e) => startResize("bottom-right", e)}
    ></div>
</div>
{#if player.currentTrack}
    <Player />
{/if}

<ShortcutSettingsModal bind:open={shortcutModalOpen} />
