<script lang="ts">
    import { listen } from "@tauri-apps/api/event";
    import { invalidateAll } from "$app/navigation";

    interface ProgressEvent {
        current: number;
        total: number;
        message: string;
    }

    let scanProgress = $state({ current: 0, total: 0, message: "" });
    let fetchProgress = $state({ current: 0, total: 0, message: "" });
    let showScan = $state(false);
    let showFetch = $state(false);
    let scanTimeout: ReturnType<typeof setTimeout>;

    $effect(() => {
        const unlistenScan = listen<ProgressEvent>("scan-progress", (event) => {
            scanProgress = event.payload;
            showScan = true;
            
            if (scanProgress.current === 100 && scanProgress.total === 100) {
                invalidateAll();
                clearTimeout(scanTimeout);
                scanTimeout = setTimeout(() => {
                    showScan = false;
                    scanProgress = { current: 0, total: 0, message: "" };
                }, 3000);
            }
        });

        const unlistenFetch = listen<ProgressEvent>("fetch-progress", (event) => {
            fetchProgress = event.payload;
            showFetch = true;
            
            if (fetchProgress.current === fetchProgress.total && fetchProgress.total > 0) {
                setTimeout(() => {
                    showFetch = false;
                    fetchProgress = { current: 0, total: 0, message: "" };
                }, 3000);
            }
        });

        const unlistenUpdate = listen("library-updated", () => {
            invalidateAll();
        });

        return () => {
            unlistenScan.then((fn) => fn());
            unlistenFetch.then((fn) => fn());
            unlistenUpdate.then((fn) => fn());
        };
    });

    let scanPercent = $derived(
        scanProgress.total > 0
            ? Math.round((scanProgress.current / scanProgress.total) * 100)
            : 0,
    );
    let fetchPercent = $derived(
        fetchProgress.total > 0
            ? Math.round((fetchProgress.current / fetchProgress.total) * 100)
            : 0,
    );
</script>

<div class="fixed top-6 left-1/2 -translate-x-1/2 flex flex-col gap-4 z-[9999] pointer-events-none w-full max-w-md px-4">
    {#if showScan}
        <div
            class="bg-dark-alt/95 backdrop-blur-2xl p-4 rounded-2xl w-full shadow-2xl border border-white/10 flex flex-col gap-2 transition-all animate-fade-in-down pointer-events-auto"
        >
            <div class="flex items-center justify-between">
                <div class="text-sm font-black text-light flex items-center gap-2">
                    {#if scanPercent < 100}
                        <div class="w-2 h-2 bg-accent rounded-full animate-pulse"></div>
                        Scanning Library...
                    {:else}
                        <div class="w-2 h-2 bg-green-500 rounded-full"></div>
                        Library Up to Date
                    {/if}
                </div>
                <div class="text-xs font-mono text-secondary font-bold">{scanPercent}%</div>
            </div>
            
            <div class="text-[10px] text-light/40 truncate font-bold uppercase tracking-widest">
                {scanProgress.message}
            </div>

            <div class="w-full bg-white/5 h-1.5 rounded-full overflow-hidden">
                <div
                    class="bg-accent h-full transition-all duration-500 ease-out shadow-[0_0_15px_rgba(var(--color-secondary),0.4)]"
                    style={`width: ${scanPercent}%`}
                ></div>
            </div>
        </div>
    {/if}

    {#if showFetch}
        <div
            class="bg-dark-alt/95 backdrop-blur-2xl p-4 rounded-2xl w-full shadow-2xl border border-white/10 flex flex-col gap-2 transition-all animate-fade-in-down pointer-events-auto"
        >
            <div class="flex items-center justify-between">
                <div class="text-sm font-black text-light flex items-center gap-2">
                    {#if fetchPercent < 100}
                        <div class="w-2 h-2 bg-accent rounded-full animate-pulse"></div>
                        Updating Artist Photos...
                    {:else}
                        <div class="w-2 h-2 bg-green-500 rounded-full"></div>
                        Photos Updated
                    {/if}
                </div>
                <div class="text-xs font-mono text-primary font-bold">{fetchPercent}%</div>
            </div>

            <div class="text-[10px] text-light/40 truncate font-bold uppercase tracking-widest">
                {fetchProgress.message}
            </div>

            <div class="w-full bg-white/5 h-1.5 rounded-full overflow-hidden">
                <div
                    class="bg-accent h-full transition-all duration-500 ease-out shadow-[0_0_15px_rgba(var(--color-primary),0.4)]"
                    style={`width: ${fetchPercent}%`}
                ></div>
            </div>
        </div>
    {/if}
</div>
