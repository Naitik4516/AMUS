<script lang="ts">
    import { getCurrentWindow } from "@tauri-apps/api/window";
    type ResizeDirection =
        | "East"
        | "North"
        | "NorthEast"
        | "NorthWest"
        | "South"
        | "SouthEast"
        | "SouthWest"
        | "West";

    let window = getCurrentWindow();

    async function startResize(direction: ResizeDirection, e: MouseEvent) {
        e.preventDefault();
        e.stopPropagation();
        await window.startResizeDragging(direction);
    }
</script>

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
<div
    role="presentation"
    class="fixed bottom-0 left-0 w-4 h-4 cursor-sw-resize z-999"
    onmousedown={(e) => startResize("SouthWest", e)}
></div>
<div
    role="presentation"
    class="fixed top-0 right-0 w-4 h-4 cursor-ne-resize z-999"
    onmousedown={(e) => startResize("NorthEast", e)}
></div>
<div
    role="presentation"
    class="fixed top-0 left-0 w-4 h-4 cursor-nw-resize z-999"
    onmousedown={(e) => startResize("NorthWest", e)}
></div>
