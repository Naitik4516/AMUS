<script lang="ts">
    import { listen } from "@tauri-apps/api/event";

    interface ScanProgressEvent {
        current: number;
        total: number;
        message: string;
    }

    let progress = $state(0);
    let message = $state("");

    $effect(() => {
        const unlisten = listen<ScanProgressEvent>("scan_progress", (event) => {
            progress = (event.payload.current / event.payload.total) * 100;
            message = event.payload.message;
        });

        return () => {
            unlisten.then((fn) => fn());
        };
    });
</script>

<div
    class="absolute bottom-4 right-4 bg-white/5 backdrop-blur-sm p-5 rounded-2xl w-60 max-h-30 flex flex-col gap-2 z-10"
    hidden={progress === 0 || progress === 100}
>
    <div class="text-md font-bold">Scanning Music Library...</div>
    <div class="w-full items-center inline-flex gap-2">
        <div class="font-mono pt-1">{progress}%</div>
        <div class="italic">{message}</div>
    </div>
    <div class="w-full bg-white/10 h-2 rounded-full overflow-hidden">
        <div class="bg-primary h-full" style={`width: ${progress}%`}></div>
    </div>
</div>
