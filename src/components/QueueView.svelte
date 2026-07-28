<script lang="ts">
    import { X } from "@lucide/svelte";
    import TrackListSmall from "./ui/TrackListSmall.svelte";
    import { player } from "$lib/player.svelte";
    import { slide } from "svelte/transition";
    import { fade } from "svelte/transition";
    import { flip } from "svelte/animate";
    import Button from "./ui/button/button.svelte";
    import type { Track } from "$lib/types";

    let { showQueue = $bindable(false) }: { showQueue?: boolean } = $props();

    let userQueue = $derived(player.userQueue);

    let isDragging = $state(false);
    let dragSection = $state<"user" | "context" | null>(null);
    let dragItem = $state<Track | null>(null);
    let dragFromIndex = $state(0);
    let dragStartY = $state(0);
    let dragX = $state(0);
    let dragY = $state(0);
    let dropIndex = $state<number | null>(null);
    let itemHeight = $state(60);
    let containerEl = $state<HTMLDivElement | null>(null);
    let autoScrollTimer: ReturnType<typeof setInterval> | null = null;

    const DRAG_THRESHOLD = 5;
    const SCROLL_ZONE = 40;
    const SCROLL_SPEED = 20;

    function getTotalItems(section: "user" | "context"): number {
        return section === "user" ? userQueue.length : player.playNext.length;
    }

    let previewEl: HTMLDivElement | null = null;

    function createPreview(clone: HTMLElement) {
        previewEl = clone.cloneNode(true) as HTMLDivElement;
        previewEl.style.position = "fixed";
        previewEl.style.pointerEvents = "none";
        previewEl.style.zIndex = "9999";
        previewEl.style.opacity = "0.5";
        previewEl.style.width = "21rem";
        previewEl.style.borderRadius = "0.75rem";
        previewEl.style.border = "1px solid rgba(255,255,255,0.07)";
        previewEl.style.boxShadow = "0 25px 50px -12px rgba(0,0,0,0.25)";
        previewEl.style.background = "rgba(30,30,40,0.95)";
        previewEl.style.backdropFilter = "blur(8px)";
        previewEl.classList.remove(
            "invisible",
            "cursor-grab",
            "cursor-default",
        );
        document.body.appendChild(previewEl);
    }

    function destroyPreview() {
        if (previewEl) {
            document.body.removeChild(previewEl);
            previewEl = null;
        }
    }

    function startDrag(
        e: PointerEvent,
        section: "user" | "context",
        index: number,
        track: Track,
    ) {
        if (e.button !== 0) return;
        const target = e.currentTarget as HTMLElement;
        itemHeight = target.offsetHeight || 60;

        dragSection = section;
        dragItem = track;
        dragFromIndex = index;
        dragStartY = e.clientY;
        dragX = e.clientX;
        dragY = e.clientY;
        dropIndex = index;
        isDragging = false;

        createPreview(target);

        document.addEventListener("pointermove", onDocumentMove);
        document.addEventListener("pointerup", onDocumentUp);
        document.addEventListener("pointercancel", onDocumentCancel);
    }

    function onDocumentMove(e: PointerEvent) {
        if (dragSection === null) return;
        const deltaY = e.clientY - dragStartY;

        if (!isDragging && Math.abs(deltaY) > DRAG_THRESHOLD) {
            isDragging = true;
        }
        if (!isDragging) return;

        dragX = e.clientX;
        dragY = e.clientY;

        if (previewEl) {
            previewEl.style.left = `${e.clientX - 168}px`;
            previewEl.style.top = `${e.clientY - 20}px`;
        }

        const total = getTotalItems(dragSection);
        const deltaItems = Math.round(deltaY / itemHeight);
        dropIndex = Math.max(
            0,
            Math.min(total - 1, dragFromIndex + deltaItems),
        );

        handleAutoScroll(e);
    }

    function onDocumentUp(_e: PointerEvent) {
        document.removeEventListener("pointermove", onDocumentMove);
        document.removeEventListener("pointerup", onDocumentUp);
        document.removeEventListener("pointercancel", onDocumentCancel);
        if (
            isDragging &&
            dragSection !== null &&
            dropIndex !== null &&
            dropIndex !== dragFromIndex
        ) {
            if (dragSection === "user") {
                player.reorderQueue(dragItem!.queue_id!, dropIndex);
            } else {
                player.reorderContextQueue(dragFromIndex, dropIndex);
            }
        }
        resetDragState();
        stopAutoScroll();
    }

    function onDocumentCancel() {
        document.removeEventListener("pointermove", onDocumentMove);
        document.removeEventListener("pointerup", onDocumentUp);
        document.removeEventListener("pointercancel", onDocumentCancel);
        resetDragState();
        stopAutoScroll();
    }

    function resetDragState() {
        isDragging = false;
        dragSection = null;
        dragItem = null;
        dropIndex = null;
        destroyPreview();
    }

    function handleAutoScroll(e: PointerEvent) {
        if (!containerEl) return;
        const rect = containerEl.getBoundingClientRect();
        const relativeY = e.clientY - rect.top;

        if (relativeY < SCROLL_ZONE) {
            startAutoScroll(-SCROLL_SPEED);
        } else if (relativeY > rect.height - SCROLL_ZONE) {
            startAutoScroll(SCROLL_SPEED);
        } else {
            stopAutoScroll();
        }
    }

    function startAutoScroll(speed: number) {
        if (autoScrollTimer) return;
        autoScrollTimer = setInterval(() => {
            if (containerEl) containerEl.scrollTop += speed;
        }, 50);
    }

    function stopAutoScroll() {
        if (autoScrollTimer) {
            clearInterval(autoScrollTimer);
            autoScrollTimer = null;
        }
    }

    function shouldShowIndicator(
        section: "user" | "context",
        index: number,
    ): boolean {
        return !!(
            isDragging &&
            dragSection === section &&
            dropIndex === index &&
            dropIndex !== dragFromIndex
        );
    }

    $inspect(isDragging, dragItem);
</script>

{#snippet DNDTrackList(tracks: Track[], section: "user" | "context")}
    {#each tracks as track, i (track.queue_id ?? track.id)}
        <div
            in:fade={{ duration: 200 }}
            out:fade={{ duration: 200 }}
            animate:flip={{ duration: 250 }}
        >
            {#if shouldShowIndicator(section, i)}
                <div class="h-0.75 bg-accent/40 rounded-md mx-2 my-1"></div>
            {/if}
            <div
                class="flex justify-between items-center rounded-xl h-16 px-1 my-1 gap-1 hover:bg-white/5 transition-colors drag-item group {isDragging &&
                dragSection === section &&
                dragFromIndex === i
                    ? 'invisible'
                    : isDragging
                      ? 'cursor-default'
                      : 'cursor-grab'}"
                onpointerdown={(e) => startDrag(e, section, i, track)}
                role="listitem"
            >
                <TrackListSmall
                    {track}
                    onclick={() => {
                        if (isDragging) return;
                        if (section === "user") {
                            player.contextPosition = i;
                        } else {
                            player.playFromContextIndex(i);
                        }
                    }}
                    styled={false}
                />
                <Button
                    variant="ghost"
                    size="icon"
                    class="text-gray-400 hover:text-red-600  group-hover:opacity-100 opacity-0"
                    onclick={() => {
                        const id =
                            section === "user" ? track.queue_id! : track.id;
                        player.removeFromQueue(id, section);
                    }}
                    title={section === "user"
                        ? "Remove from Queue"
                        : "Play Next"}
                >
                    <X size={28} />
                </Button>
            </div>
        </div>
    {/each}
{/snippet}

{#if showQueue}
    <div
        bind:this={containerEl}
        class="absolute bottom-full right-1 mb-4 w-96 bg-card/60 backdrop-blur-2xl border-2 border-border/70 rounded-2xl shadow-2xl flex flex-col max-h-[75vh] overflow-hidden {isDragging
            ? 'select-none'
            : ''}"
        transition:slide
    >
        <div
            class="p-4 border-b border-neutral-800 flex justify-between items-center bg-neutral-900/50"
        >
            <h3 class="font-bold text-white text-lg">Queue</h3>
            <button
                onclick={() => (showQueue = false)}
                class="text-gray-300 hover:text-white"
            >
                <X size={18} />
            </button>
        </div>
        <div class="flex flex-col gap-2 px-3 pb-4 overflow-y-scroll">
            {#if player.currentTrack}
                <section>
                    <h4
                        class="py-2 text-[13px] font-switzer font-bold uppercase tracking-wider text-stone-300"
                    >
                        Now Playing
                    </h4>

                    <TrackListSmall
                        track={player.currentTrack}
                        styled={true}
                        onclick={() => {}}
                    />
                </section>
            {/if}

            {#if userQueue.length > 0}
                <section>
                    <div class="flex items-center justify-between">
                        <h4
                            class="py-2 text-[13px] font-bold uppercase tracking-wider text-stone-300"
                        >
                            Next in Queue
                        </h4>
                        <button
                            onclick={() => player.clearQueue()}
                            class="text-sm font-semibold text-gray-400 hover:text-red-400 transition-colors font-switzer"
                        >
                            Clear Queue
                        </button>
                    </div>

                    {@render DNDTrackList(userQueue, "user")}
                </section>
            {/if}

            {#if player.playNext.length > 0}
                <section>
                    <h4
                        class="py-2 text-[13px] font-bold uppercase tracking-wider text-stone-300 truncate"
                    >
                        {player.nextSectionTitle}
                    </h4>
                    {@render DNDTrackList(player.playNext, "context")}
                </section>
            {/if}

            {#if !player.currentTrack && player.userQueue.length === 0 && player.playNext.length === 0}
                <p class="px-4 py-8 text-center text-sm text-zinc-500">
                    No tracks in queue
                </p>
            {/if}
        </div>
    </div>
{/if}
