<script lang="ts">
    import { X } from "@lucide/svelte";
    import TrackListSmall from "./ui/TrackListSmall.svelte";
    import { player } from "$lib/player.svelte";
    import { slide } from "svelte/transition";
    import Button from "./ui/button/button.svelte";
    import type { Track } from "$lib/types";
    import { VList } from "virtua/svelte";
    import { gsap } from "gsap";
    import { Flip } from "gsap/Flip";

    gsap.registerPlugin(Flip);

    let { showQueue = $bindable(false) }: { showQueue?: boolean } = $props();

    let userQueue = $derived(player.userQueue);

    let isDragging = $state(false);
    let dragSection = $state<"user" | "context" | null>(null);
    let dragItem = $state<Track | null>(null);
    let dragFromIndex = $state(0);
    let dragStartY = $state(0);
    let dropIndex = $state<number | null>(null);
    let itemHeight = $state(68);
    let containerEl = $state<HTMLDivElement | null>(null);
    let userVListWrapper = $state<HTMLElement | null>(null);
    let contextVListWrapper = $state<HTMLElement | null>(null);

    let activeVListScrollEl: HTMLElement | null = null;
    let lastPointerEvent: PointerEvent | null = null;
    let scrollSpeed = 0;
    let autoScrollRafId: number | null = null;

    let lastUserKeys = "";
    let lastContextKeys = "";
    let flipUserSnapshot: Flip.FlipState | null = null;
    let flipContextSnapshot: Flip.FlipState | null = null;

    $effect.pre(() => {
        const userKeys = userQueue.map((t) => t.queue_id ?? t.id).join(",");
        const contextKeys = player.playNext.map((t) => t.id).join(",");

        if (userKeys !== lastUserKeys) {
            if (userVListWrapper && lastUserKeys !== "" && !isDragging) {
                const els = userVListWrapper.querySelectorAll("[data-flip-id]");
                if (els.length > 0) {
                    flipUserSnapshot = Flip.getState(els);
                }
            }
            lastUserKeys = userKeys;
        }

        if (contextKeys !== lastContextKeys) {
            if (contextVListWrapper && lastContextKeys !== "" && !isDragging) {
                const els =
                    contextVListWrapper.querySelectorAll("[data-flip-id]");
                if (els.length > 0) {
                    flipContextSnapshot = Flip.getState(els);
                }
            }
            lastContextKeys = contextKeys;
        }
    });

    $effect(() => {
        // Read reactive dependencies
        userQueue;
        player.playNext;

        if (flipUserSnapshot && userVListWrapper) {
            const els = userVListWrapper.querySelectorAll("[data-flip-id]");
            if (els.length > 0) {
                Flip.from(flipUserSnapshot, {
                    duration: 0.3,
                    ease: "power2.out",
                    targets: els,
                    scale: false,
                    clearProps: "transform",
                });
            }
            flipUserSnapshot = null;
        }

        if (flipContextSnapshot && contextVListWrapper) {
            const els = contextVListWrapper.querySelectorAll("[data-flip-id]");
            if (els.length > 0) {
                Flip.from(flipContextSnapshot, {
                    duration: 0.3,
                    ease: "power2.out",
                    targets: els,
                    scale: false,
                    clearProps: "transform",
                });
            }
            flipContextSnapshot = null;
        }
    });

    const DRAG_THRESHOLD = 5;
    const SCROLL_ZONE = 40;

    function getTotalItems(section: "user" | "context"): number {
        return section === "user" ? userQueue.length : player.playNext.length;
    }

    function getVListScrollEl(section: "user" | "context"): HTMLElement | null {
        const wrapper =
            section === "user" ? userVListWrapper : contextVListWrapper;
        if (!wrapper) return null;
        return (wrapper.querySelector(".vlist") as HTMLElement) || wrapper;
    }

    let previewEl: HTMLDivElement | null = null;

    function createPreview(clone: HTMLElement) {
        previewEl = clone.cloneNode(true) as HTMLDivElement;
        previewEl.style.position = "fixed";
        previewEl.style.pointerEvents = "none";
        previewEl.style.zIndex = "999";
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
        const wrapper =
            (target.closest(".drag-item-wrapper") as HTMLElement) || target;
        itemHeight = wrapper.offsetHeight || 72;

        dragSection = section;
        dragItem = track;
        dragFromIndex = index;
        dragStartY = e.clientY;
        dropIndex = index;
        isDragging = false;
        activeVListScrollEl = getVListScrollEl(section);

        createPreview(target);

        document.addEventListener("pointermove", onDocumentMove);
        document.addEventListener("pointerup", onDocumentUp);
        document.addEventListener("pointercancel", onDocumentCancel);
        document.addEventListener("keydown", onDocumentKeyDown);
    }

    function updateDropIndexAndAutoScroll(e: PointerEvent) {
        if (dragSection === null || !activeVListScrollEl) return;

        const rect = activeVListScrollEl.getBoundingClientRect();
        const relativeY = e.clientY - rect.top;
        const scrollTop = activeVListScrollEl.scrollTop;
        const contentY = relativeY + scrollTop;

        const total = getTotalItems(dragSection);
        const calculatedIndex = Math.floor(
            (contentY + itemHeight / 2) / itemHeight,
        );
        dropIndex = Math.max(0, Math.min(total - 1, calculatedIndex));

        if (relativeY < SCROLL_ZONE) {
            const distance = SCROLL_ZONE - relativeY;
            scrollSpeed = -Math.min(25, Math.max(5, distance * 0.5));
            startAutoScrollLoop();
        } else if (relativeY > rect.height - SCROLL_ZONE) {
            const distance = relativeY - (rect.height - SCROLL_ZONE);
            scrollSpeed = Math.min(25, Math.max(5, distance * 0.5));
            startAutoScrollLoop();
        } else {
            stopAutoScrollLoop();
        }
    }

    function startAutoScrollLoop() {
        if (autoScrollRafId !== null) return;
        const tick = () => {
            if (!isDragging || !activeVListScrollEl || scrollSpeed === 0) {
                autoScrollRafId = null;
                return;
            }
            activeVListScrollEl.scrollTop += scrollSpeed;
            if (lastPointerEvent && activeVListScrollEl) {
                const rect = activeVListScrollEl.getBoundingClientRect();
                const relativeY = lastPointerEvent.clientY - rect.top;
                const scrollTop = activeVListScrollEl.scrollTop;
                const contentY = relativeY + scrollTop;
                const total = getTotalItems(dragSection!);
                const calculatedIndex = Math.floor(
                    (contentY + itemHeight / 2) / itemHeight,
                );
                dropIndex = Math.max(0, Math.min(total - 1, calculatedIndex));
            }
            autoScrollRafId = requestAnimationFrame(tick);
        };
        autoScrollRafId = requestAnimationFrame(tick);
    }

    function stopAutoScrollLoop() {
        if (autoScrollRafId !== null) {
            cancelAnimationFrame(autoScrollRafId);
            autoScrollRafId = null;
        }
        scrollSpeed = 0;
    }

    function onDocumentMove(e: PointerEvent) {
        if (dragSection === null) return;
        lastPointerEvent = e;
        const deltaY = e.clientY - dragStartY;

        if (!isDragging && Math.abs(deltaY) > DRAG_THRESHOLD) {
            isDragging = true;
        }
        if (!isDragging) return;

        if (previewEl) {
            previewEl.style.left = `${e.clientX - 168}px`;
            previewEl.style.top = `${e.clientY - 20}px`;
        }

        updateDropIndexAndAutoScroll(e);
    }

    function onDocumentKeyDown(e: KeyboardEvent) {
        if (e.key === "Escape") {
            onDocumentCancel();
        }
    }

    function onDocumentUp(_e: PointerEvent) {
        document.removeEventListener("pointermove", onDocumentMove);
        document.removeEventListener("pointerup", onDocumentUp);
        document.removeEventListener("pointercancel", onDocumentCancel);
        document.removeEventListener("keydown", onDocumentKeyDown);
        stopAutoScrollLoop();

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
    }

    function onDocumentCancel() {
        document.removeEventListener("pointermove", onDocumentMove);
        document.removeEventListener("pointerup", onDocumentUp);
        document.removeEventListener("pointercancel", onDocumentCancel);
        document.removeEventListener("keydown", onDocumentKeyDown);
        stopAutoScrollLoop();
        resetDragState();
    }

    function resetDragState() {
        isDragging = false;
        dragSection = null;
        dragItem = null;
        dropIndex = null;
        activeVListScrollEl = null;
        lastPointerEvent = null;
        destroyPreview();
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

    const calcHeight = (tracks: number) => {
        const maxHeight = 360;
        const totalHeight = tracks * itemHeight + 16;
        return Math.min(totalHeight, maxHeight);
    };

</script>

{#snippet DNDTrackList(tracks: Track[], section: "user" | "context")}
    <VList
        class="vlist pr-1 pt-1 mask-y-from-95% scroll-smooth"
        data={tracks}
        style="height: {calcHeight(tracks.length)}px"
        getKey={(track, _i) => track.queue_id ?? track.id}
    >
        {#snippet children(track, i)}
            <div
                data-flip-id={track.queue_id ?? track.id}
                class="relative drag-item-wrapper py-1"
            >
                {#if shouldShowIndicator(section, i)}
                    <div
                        class="absolute -top-1 left-2 right-2 h-0.75 bg-zinc-500 rounded z-20 pointer-events-none "
                    ></div>
                {/if}
                <div
                    class="flex justify-between items-center rounded-xl h-16 px-1 gap-1 hover:bg-white/5 hover:shadow-lg transition-colors duration-300 drag-item group {isDragging &&
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
                        class="text-gray-400 hover:text-red-600 group-hover:opacity-100 opacity-0"
                        onclick={() => {
                            const id =
                                section === "user" ? track.queue_id! : track.id;
                            player.removeFromQueue(id, section);
                        }}
                        title={section === "user"
                            ? "Remove from Queue"
                            : "Remove from Play Next"}
                    >
                        <X size={28} />
                    </Button>
                </div>
            </div>
        {/snippet}
    </VList>
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
        <div class="flex flex-col gap-1 px-3 pb-4 overflow-y-scroll">
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
                <section bind:this={userVListWrapper}>
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
                <section bind:this={contextVListWrapper}>
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
