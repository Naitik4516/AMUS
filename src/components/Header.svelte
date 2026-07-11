<script lang="ts">
    import { Maximize2, Minimize2, Minus, X } from "@lucide/svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import GlobalSearch from "./GlobalSearch.svelte";

    let { isMaximized = $bindable(false) } = $props();

    function toggleMaximize() {
        const appWindow = getCurrentWindow();
        appWindow.toggleMaximize();
    }

    function minimize() {
        const appWindow = getCurrentWindow();
        appWindow.minimize();
    }

    function close() {
        const appWindow = getCurrentWindow();
        appWindow.close();
    }

    $effect(() => {
        const appWindow = getCurrentWindow();
        let unlisten: () => void;

        appWindow.isMaximized().then((max) => {
            isMaximized = max;
        });

        appWindow
            .listen("tauri://resize", async () => {
                isMaximized = await appWindow.isMaximized();
            })
            .then((unlistenFn) => {
                unlisten = unlistenFn;
            });

        return () => {
            if (unlisten) unlisten();
        };
    });
</script>

<header class="fixed top-0 inset-x-0 z-20">
    <div class="h-2 shrink-0"></div>
    <div
        data-tauri-drag-region
        class="flex items-center px-4 h-12 justify-between select-none text-white cursor-grab"
    >
        <div class="w-16 flex justify-center shrink-0 mt-1 pointer-events-none">
            <img
                src="/icon.svg"
                alt="Avatar"
                class="w-10 h-10 object-contain select-none"
            />
        </div>

        <GlobalSearch />

        <div
            class="controls flex align-top bg-card/30 backdrop-blur-lg shadow-md rounded-full border p-1"
        >
            <button id="titlebar-minimize" title="Minimize" onclick={minimize}>
                <Minus size={14} />
            </button>

            <button
                id="titlebar-maximize"
                title={isMaximized ? "Restore Down" : "Maximize"}
                onclick={toggleMaximize}
            >
                {#if isMaximized}
                    <Minimize2 size={14} />
                {:else}
                    <Maximize2 size={14} />
                {/if}
            </button>

            <button
                id="titlebar-close"
                title="Close"
                onclick={close}
                class="close-btn"
            >
                <X size={14} />
            </button>
        </div>
    </div>
</header>

<style>
    .controls button {
        appearance: none;
        padding: 0;
        margin: 0 2px;
        border: none;
        display: inline-flex;
        justify-content: center;
        align-items: center;
        width: 28px;
        height: 28px;
        color: white;
        background-color: transparent;
        cursor: pointer;
        border-radius: 12px;
        transition: background-color 0.15s ease;
    }

    .controls button:hover {
        background-color: rgba(255, 255, 255, 0.1);
    }

    .controls button.close-btn:hover {
        background-color: rgba(225, 0, 0, 0.9);
    }
</style>
