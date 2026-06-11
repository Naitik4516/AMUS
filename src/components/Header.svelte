<script lang="ts">
    import { X, Minus, Square, Search } from "@lucide/svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    

    let isMaximized = $state(false);

    $effect(() => {
        const appWindow = getCurrentWindow();
        let unlisten: () => void;

        // Fetch initial window state
        appWindow.isMaximized().then((max) => {
            isMaximized = max;
        });

        // Listen for window resize/maximize triggers to keep UI perfectly in sync
        appWindow
            .listen("tauri://resize", async () => {
                isMaximized = await appWindow.isMaximized();
            })
            .then((unlistenFn) => {
                unlisten = unlistenFn;
            });

        // Cleanup listener when the component destroys
        return () => {
            if (unlisten) unlisten();
        };
    });

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
</script>

<header
    data-tauri-drag-region
    class="h-16 flex items-center justify-between p-3 select-none bg-neutral-900 text-white"
>
    <div class="w-16 flex justify-center shrink-0 mt-1 pointer-events-none">
        <img
            src="/favicon.png?"
            alt="Avatar"
            class="w-10 h-10 object-contain select-none"
        />
    </div>

    <div
        class="flex items-center gap-2 bg-neutral-800 rounded-full px-4 py-2 w-96 border border-transparent transition-colors duration-300 hover:border-neutral-600 focus-within:border-secondary"
    >
        <Search size={16} class="text-gray-400" />
        <input
            type="text"
            placeholder="What do you want to listen to?"
            class="w-full outline-none bg-transparent text-white placeholder-gray-400 text-sm"
        />
    </div>

    <div class="controls flex items-center">
        <button id="titlebar-minimize" title="Minimize" onclick={minimize}>
            <Minus size={14} />
        </button>

        <button
            id="titlebar-maximize"
            title={isMaximized ? "Restore Down" : "Maximize"}
            onclick={toggleMaximize}
        >
            {#if isMaximized}
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-squares">
	<path stroke="none" d="M0 0h24v24H0z" fill="none" />
	<path d="M8 10a2 2 0 0 1 2 -2h9a2 2 0 0 1 2 2v9a2 2 0 0 1 -2 2h-9a2 2 0 0 1 -2 -2l0 -9" />
	<path d="M16 8v-3a2 2 0 0 0 -2 -2h-9a2 2 0 0 0 -2 2v9a2 2 0 0 0 2 2h3" />
                </svg>
            {:else}
                <Square size={13} />
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
        width: 32px;
        height: 32px;
        color: white;
        background-color: transparent;
        cursor: pointer;
        border-radius: 4px;
        transition: background-color 0.15s ease;
    }

    .controls button:hover {
        background-color: rgba(255, 255, 255, 0.1);
    }

    .controls button.close-btn:hover {
        background-color: #ef4444; /* Standard close red accent */
    }
</style>
