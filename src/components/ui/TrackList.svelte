<script lang="ts">
    import { invalidate } from "$app/navigation";
    import { openConfirmDialog } from "$lib/context-menu.svelte";
    import { player } from "$lib/player.svelte";
    import { store } from "$lib/stores.svelte";
    import type { Context, MenuPosition, PlaybackSource, Track } from "$lib/types";
    import { formatDurationColon } from "$lib/utils";
    import {
        ChevronDown,
        ChevronUp,
        CircleMinus,
        Clock,
        Disc,
        Ellipsis,
        Heart,
        ListPlus,
        Music2,
        Pause,
        Play,
        Shuffle,
        SlidersHorizontal,
        Trash2,
        X,
    } from "@lucide/svelte";
    import { toast } from "svelte-sonner";
    import { VList } from "virtua/svelte";
    import Button from "./button/button.svelte";
    import EditAlbumDialog from "./Dialog/EditAlbumDialog.svelte";
    import EditArtistDialog from "./Dialog/EditArtistDialog.svelte";
    import EditPlaylistDialog from "./Dialog/EditPlaylistDialog.svelte";
    import DropdownMenu, { type MenuItem } from "./Menu/DropdownMenu.svelte";
    import PlaylistMenu from "./Menu/PlaylistMenu.svelte";
    import TrackMenu from "./Menu/TrackMenu.svelte";
    import PlayingVisualizer from "./PlayingVisualizer.svelte";

    type ColumnKey = (typeof COLUMN_ORDER)[number];

    type NonNullContext = Exclude<Context, null>;

    const COLUMN_ORDER = [
        "index",
        "title",
        "album",
        "dateAdded",
        "duration",
    ] as const;

    interface TrackTableProps {
        tracks: Track[];
        context: NonNullContext;
        visibleColumns?: ColumnKey[] | null;
        canEdit?: boolean;
        canSort?: boolean;
        canToggleColumns?: boolean;
        accentColor?: string;
    }

    let {
        tracks = [],
        context,
        visibleColumns = null,
        canEdit = true,
        canSort = true,
        canToggleColumns = true,
        accentColor = "#fff",
    }: TrackTableProps = $props();

    const COLUMN_META: Record<
        ColumnKey,
        {
            label: string;
            settingsLabel: string;
            width: number;
            minWidth: number;
            maxWidth: number;
            sortable: boolean;
            locked: boolean;
            resizable: boolean;
            icon?: string;
        }
    > = {
        index: {
            label: "#",
            settingsLabel: "Index",
            width: 56,
            minWidth: 44,
            maxWidth: 44,
            sortable: false,
            locked: true,
            resizable: false,
        },
        title: {
            label: "Title",
            settingsLabel: "Title",
            width: 300,
            minWidth: 100,
            maxWidth: 640,
            sortable: true,
            locked: false,
            resizable: true,
        },
        album: {
            label: "Album",
            settingsLabel: "Album",
            width: 300,
            minWidth: 80,
            maxWidth: 420,
            sortable: true,
            locked: false,
            resizable: true,
        },
        dateAdded: {
            label: "Date added",
            settingsLabel: "Date added",
            width: 100,
            minWidth: 80,
            maxWidth: 150,
            sortable: true,
            locked: false,
            resizable: true,
        },
        duration: {
            label: "",
            settingsLabel: "Duration",
            width: 64,
            minWidth: 64,
            maxWidth: 64,
            sortable: true,
            locked: false,
            resizable: false,
            icon: "clock",
        },
    };

    const CONTEXT_DEFAULT_COLUMNS: Record<string, ColumnKey[]> = {
        Playlist: ["index", "title", "album", "dateAdded", "duration"],
        Favorites: ["index", "title", "album", "dateAdded", "duration"],
        Album: ["index", "title", "duration"],
        Artist: ["index", "title", "album", "duration"],
    };

    let columns = $state(
        Object.fromEntries(
            COLUMN_ORDER.map((key) => [
                key,
                {
                    visible: (
                        visibleColumns ??
                        CONTEXT_DEFAULT_COLUMNS[context.type] ??
                        COLUMN_ORDER
                    ).includes(key),
                    width: COLUMN_META[key].width,
                },
            ]),
        ) as Record<ColumnKey, { visible: boolean; width: number }>,
    );
    let density = $state<"relaxed" | "compact">("relaxed");

    let sortKey = $state<ColumnKey | null>(null);
    let sortDir = $state<"asc" | "desc">("asc");

    let settingsOpen = $state(false);
    let settingsBtn = $state<HTMLButtonElement | null>(null);
    let settingsPanel = $state<HTMLDivElement | null>(null);
    let showEditDialog = $state(false);

    let actionMenuOpen = $state<MenuPosition | null>(null);
    let rowMenuOpen = $state<[MenuPosition, Track] | null>(null);
    let groupMenuOpen = $state<MenuPosition | null>(null);

    let selectedIds = $state<Set<number>>(new Set());
    let lastSelectedIndex = $state<number | null>(null);

    function compareTracks(a: Track, b: Track, key: ColumnKey) {
        if (key === "title") return a.title.localeCompare(b.title);
        if (key === "album")
            return (a.album?.name ?? "").localeCompare(b.album?.name ?? "");
        if (key === "duration") return a.duration_seconds - b.duration_seconds;
        if (key === "dateAdded")
            return (
                new Date(a.added_at).getTime() - new Date(b.added_at).getTime()
            );
        return 0;
    }

    let orderedTracks = $derived.by(() => {
        if (context.type === "Album") {
            const sorted = [...tracks].sort((a, b) => {
                const aNum = a.track_number ?? Number.MAX_SAFE_INTEGER;
                const bNum = b.track_number ?? Number.MAX_SAFE_INTEGER;
                return aNum - bNum;
            });
            return sorted;
        }
        if (!sortKey) {
            if (context.type === "Playlist") {
                return [...tracks];
            }
            return [...tracks].sort((a, b) => a.id - b.id);
        }
        const key = sortKey;
        const sorted = [...tracks].sort((a, b) => compareTracks(a, b, key));
        return sortDir === "desc" ? sorted.reverse() : sorted;
    });

    let selectedTracks = $derived(
        orderedTracks.filter((t) => selectedIds.has(t.id)),
    );
    let allSelectedFavorite = $derived(
        selectedTracks.length > 0 && selectedTracks.every((t) => t.is_favorite),
    );
    let isSelectionPlaying = $derived(
        player.isPlaying &&
            player.currentTrack &&
            selectedTracks.some((x) => player.currentTrack?.id === x.id),
    );

    let rootEl = $state<HTMLDivElement | null>(null);

    let rowHeight = $derived(density === "compact" ? 68 : 80);
    let viewportHeight = $state(window.innerHeight);

    $effect(() => {
        const onResize = () => {
            viewportHeight = window.innerHeight;
        };
        window.addEventListener("resize", onResize);
        return () => window.removeEventListener("resize", onResize);
    });

    let VListHeight = $derived(
        Math.min(orderedTracks.length * rowHeight, viewportHeight),
    );

    function toggleSort(key: ColumnKey) {
        if (!canSort || context.type === "Album") return;
        if (!COLUMN_META[key].sortable) return;
        if (sortKey !== key) {
            sortKey = key;
            sortDir = "asc";
        } else if (sortDir === "asc") {
            sortDir = "desc";
        } else {
            sortKey = null;
            sortDir = "asc";
        }
    }

    let visibleColumnKeys = $derived(
        COLUMN_ORDER.filter((key) => columns[key].visible),
    );

    let gridTemplate = $derived(
        visibleColumnKeys
            .map((key) =>
                key === "title"
                    ? `minmax(${COLUMN_META.title.minWidth}px, 1fr)`
                    : `${columns[key].width}px`,
            )
            .join(" ") + " 40px",
    );

    function isColumnResizable(key: ColumnKey) {
        const meta = COLUMN_META[key];
        if (!meta.resizable) return false;
        if (key === "title") {
            const titleIdx = visibleColumnKeys.indexOf("title");
            if (titleIdx === -1) return false;
            return visibleColumnKeys
                .slice(titleIdx + 1)
                .some((k) => COLUMN_META[k].resizable);
        }
        return true;
    }

    function startResize(key: ColumnKey, event: PointerEvent) {
        event.preventDefault();
        const resizer = event.currentTarget as HTMLElement;
        resizer.setPointerCapture(event.pointerId);

        let targetKey = key;
        let direction = 1;

        if (key === "title") {
            const titleIdx = visibleColumnKeys.indexOf("title");
            const rightKey = visibleColumnKeys
                .slice(titleIdx + 1)
                .find((k) => COLUMN_META[k].resizable);
            if (!rightKey) return;
            targetKey = rightKey;
            direction = -1;
        }

        const startX = event.clientX;
        const startWidth = columns[targetKey].width;
        const meta = COLUMN_META[targetKey];

        function onMove(e: PointerEvent) {
            const deltaX = e.clientX - startX;
            const next = startWidth + deltaX * direction;
            columns[targetKey].width = Math.min(
                meta.maxWidth,
                Math.max(meta.minWidth, next),
            );
        }
        function onUp(e: PointerEvent) {
            try {
                resizer.releasePointerCapture(e.pointerId);
            } catch (err) {
                // Ignore if pointer capture was already released or invalid
            }
            resizer.removeEventListener("pointermove", onMove);
            resizer.removeEventListener("pointerup", onUp);
            resizer.removeEventListener("pointercancel", onCancel);
        }
        function onCancel(e: PointerEvent) {
            onUp(e);
        }
        resizer.addEventListener("pointermove", onMove);
        resizer.addEventListener("pointerup", onUp);
        resizer.addEventListener("pointercancel", onCancel);
    }

    function formatDateAdded(value: string) {
        const date = new Date(value);
        const diffDays = Math.floor((Date.now() - date.getTime()) / 86_400_000);
        if (diffDays <= 0) return "Today";
        if (diffDays === 1) return "Yesterday";
        if (diffDays < 7) return `${diffDays} days ago`;
        return date.toLocaleDateString("en-US", {
            month: "short",
            day: "numeric",
            year: "numeric",
        });
    }

    function contextToSource(ctx: NonNullContext): PlaybackSource {
        switch (ctx.type) {
            case "Album":
            case "Playlist":
            case "Artist":
            case "Genre":
                return { type: ctx.type, id: ctx.id };
            case "Favorites":
                return { type: "Favorites" };
        }
    }

    function handleMainPlay() {
        if (!orderedTracks) return;
        if (
            player.isPlaying &&
            orderedTracks.some((x) => player.currentTrack?.id === x.id)
        )
            player.playPause();
        else
            player.play(
                orderedTracks,
                contextToSource(context),
                0,
                context.name,
            );
    }

    function handleRowActivate(track: Track, index: number) {
        if (player.currentTrack?.id === track.id && player.isPlaying)
            player.playPause();
        else
            player.play(
                orderedTracks,
                contextToSource(context),
                index,
                context.name,
            );
    }

    async function toggleFavorite(track: Track) {
        try {
            await store.toggleFavorite(track.id);
            invalidate("app:track-details");
        } catch (e) {
            console.error("Failed to toggle favorite", e);
        }
    }

    function clearSelection() {
        selectedIds = new Set();
        lastSelectedIndex = null;
    }

    function handleRowClick(e: MouseEvent, track: Track, i: number) {
        const target = e.target as HTMLElement;
        if (target.closest("a, button, input, label")) return;
        if (e.shiftKey && lastSelectedIndex !== null) {
            const start = Math.min(lastSelectedIndex, i);
            const end = Math.max(lastSelectedIndex, i);
            const ids = new Set(selectedIds);
            for (let j = start; j <= end; j++) ids.add(orderedTracks[j].id);
            selectedIds = ids;
            return;
        }
        if (e.ctrlKey || e.metaKey) {
            const ids = new Set(selectedIds);
            if (ids.has(track.id)) {
                ids.delete(track.id);
            } else {
                ids.add(track.id);
                lastSelectedIndex = i;
            }
            selectedIds = ids;
            return;
        }
        selectedIds = new Set([track.id]);
        lastSelectedIndex = i;
    }

    function handlePlaySelection() {
        if (selectedTracks.length === 0) return;
        if (isSelectionPlaying) player.playPause();
        else player.play(selectedTracks, { type: "Direct" }, 0, context.name);
    }

    async function handleToggleSelectionFavorites() {
        const ids = selectedTracks.map((t) => t.id);
        const adding = !allSelectedFavorite;
        try {
            await store.setTracksFavorite(ids, adding);
            toast.success(
                adding ? "Added to favourites" : "Removed from favourites",
            );
            clearSelection();
        } catch (e) {
            console.error("Failed to update favourites", e);
            toast.error("Failed to update favourites");
        }
    }

    async function handleRemoveFromPlaylist() {
        if (context.type !== "Playlist") return;
        const ids = selectedTracks.map((t) => t.id);
        try {
            await store.removeTracksFromPlaylist(ids, context.id);
            toast.success("Removed from playlist");
            clearSelection();
        } catch (e) {
            console.error("Failed to remove from playlist", e);
            toast.error("Failed to remove from playlist");
        }
    }

    async function handleAddToQueue() {
        player.enqueueEndMany(selectedTracks);
        toast.success("Added to queue");
        clearSelection();
    }

    async function handlePlayNext() {
        const tracks = [...selectedTracks];
        for (const track of tracks.reverse()) {
            await player.enqueueNext(track);
        }
        toast.success("Will play next");
        clearSelection();
    }

    function handleDeleteSelected() {
        const count = selectedTracks.length;
        const ids = selectedTracks.map((t) => t.id);
        const preview = selectedTracks
            .slice(0, 3)
            .map((t) => `"${t.title}"`)
            .join(", ");
        openConfirmDialog({
            title: count > 1 ? `Delete ${count} tracks` : "Delete track",
            message:
                count > 1
                    ? `Are you sure you want to delete these ${count} tracks (${preview}${count > 3 ? ", …" : ""}) from your library? This will also remove them from all playlists.`
                    : `Are you sure you want to delete "${selectedTracks[0].title}" from your library? This will also remove it from all playlists.`,
            confirmLabel: "Delete",
            onConfirm: async () => {
                await store.deleteTracks(ids);
                toast.success(
                    count > 1
                        ? "Tracks deleted from library"
                        : "Track deleted from library",
                );
                clearSelection();
            },
        });
    }

    let groupMenuItems = $derived.by(() => {
        const tracks = selectedTracks;
        if (tracks.length === 0) return [];
        const items: MenuItem[] = [
            {
                label: allSelectedFavorite
                    ? "Remove from favourites"
                    : "Add to favourites",
                icon: allSelectedFavorite ? "heart-filled" : "heart",
                onClick: handleToggleSelectionFavorites,
            },
            {
                label: "Add to queue",
                icon: "list-plus",
                onClick: handleAddToQueue,
            },
            {
                label: "Play next",
                icon: "skip-forward",
                onClick: handlePlayNext,
            },
            {
                label: "Add to playlist",
                icon: "plus",
                submenu: PlaylistMenu,
                tracks,
                context,
            },
        ];
        if (context.type === "Playlist") {
            items.push({
                label: "Remove from this playlist",
                icon: "circle-minus",
                danger: true,
                onClick: handleRemoveFromPlaylist,
            });
        }
        items.push({ type: "separator" });
        items.push({
            label: "Delete from library",
            icon: "trash",
            danger: true,
            onClick: handleDeleteSelected,
        });
        return items;
    });

    let actionMenuItems = $derived.by(() => {
        const items: MenuItem[] = [
            {
                label: "Add to queue",
                icon: "list-plus",
                onClick: () => {
                    player.enqueueEndMany(orderedTracks);
                    toast.success("Added to queue");
                },
            },
        ];
        if (canEdit) {
            items.push({
                label: "Edit details",
                icon: "edit",
                onClick: () => {
                    showEditDialog = true;
                },
            });
        }

        return items;
    });

    $effect(() => {
        if (!settingsOpen) return;
        function handlePointerDown(e: PointerEvent) {
            if (
                settingsPanel?.contains(e.target as Node) ||
                settingsBtn?.contains(e.target as Node)
            )
                return;
            settingsOpen = false;
        }
        function handleKey(e: KeyboardEvent) {
            if (e.key === "Escape") settingsOpen = false;
        }
        window.addEventListener("pointerdown", handlePointerDown, true);
        window.addEventListener("keydown", handleKey);
        return () => {
            window.removeEventListener("pointerdown", handlePointerDown, true);
            window.removeEventListener("keydown", handleKey);
        };
    });

    $effect(() => {
        function onKey(e: KeyboardEvent) {
            const target = e.target as HTMLElement | null;
            if (
                target?.closest(
                    "input, textarea, select, [contenteditable='true']",
                )
            )
                return;
            if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "a") {
                e.preventDefault();
                e.stopImmediatePropagation();
                selectedIds = new Set(orderedTracks.map((t) => t.id));
                return;
            }
            if (selectedIds.size === 0) return;
            if (e.key === "Escape") {
                e.stopImmediatePropagation();
                clearSelection();
                return;
            }
            const key = e.key.toLowerCase();
            if (key === "q") {
                e.preventDefault();
                e.stopImmediatePropagation();
                handleAddToQueue();
                return;
            }
            if (key === "n") {
                e.preventDefault();
                e.stopImmediatePropagation();
                handlePlayNext();
                return;
            }
            if (key === "f") {
                e.preventDefault();
                e.stopImmediatePropagation();
                handleToggleSelectionFavorites();
                return;
            }
            if (e.key === "Delete" || e.key === "Backspace") {
                e.preventDefault();
                e.stopImmediatePropagation();
                handleDeleteSelected();
            }
        }
        window.addEventListener("keydown", onKey, true);
        return () => window.removeEventListener("keydown", onKey, true);
    });

    $effect(() => {
        function onPointerDown(e: PointerEvent) {
            const target = e.target as Node;
            if (rootEl?.contains(target)) return;
            if (
                target instanceof Element &&
                target.closest(".dropdown-menu, .dropdown-trigger")
            )
                return;
            if (selectedIds.size > 0) clearSelection();
        }
        window.addEventListener("pointerdown", onPointerDown, true);
        return () =>
            window.removeEventListener("pointerdown", onPointerDown, true);
    });

    let isCurrentCollectionPlaying = $derived(
        player.isPlaying &&
            player.currentTrack &&
            orderedTracks.some((x) => player.currentTrack?.id === x.id),
    );

    const focusRing =
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/70 focus-visible:ring-offset-2 focus-visible:ring-offset-zinc-950";
</script>

<div class="w-full px-2" bind:this={rootEl}>
    <!-- ============================== ACTION BAR ============================== -->
    <div class="relative flex items-center gap-5 px-3 h-24 sm:px-4">
        {#if selectedTracks.length > 0}
            <div class="flex items-center gap-2">
                <span class="text-lg font-semibold text-white"
                    >{selectedTracks.length} selected</span
                >
                <Button
                    variant="ghost"
                    size="icon"
                    onclick={clearSelection}
                    title="Clear selection"
                >
                    <X size={18} />
                </Button>
            </div>

            <div class="flex-1"></div>

            <div class="flex items-center gap-2">
                <Button
                    variant="outline"
                    size="icon-lg"
                    onclick={handlePlaySelection}
                    title={isSelectionPlaying
                        ? "Pause selected"
                        : "Play selected"}
                >
                    {#if isSelectionPlaying}
                        <Pause size={18} fill="currentColor" />
                    {:else}
                        <Play size={18} fill="currentColor" />
                    {/if}
                </Button>

                <Button
                    variant="outline"
                    size="icon-lg"
                    class={allSelectedFavorite
                        ? "text-rose-600"
                        : "text-zinc-300"}
                    onclick={handleToggleSelectionFavorites}
                    title={allSelectedFavorite
                        ? "Remove from favourites"
                        : "Add to favourites"}
                >
                    <Heart
                        size={18}
                        fill={allSelectedFavorite ? "currentColor" : "none"}
                    />
                </Button>

                <Button
                    variant="outline"
                    size="icon-lg"
                    onclick={handleAddToQueue}
                    title="Add to queue"
                >
                    <ListPlus size={18} />
                </Button>

                {#if context.type === "Playlist"}
                    <Button
                        variant="outline"
                        size="icon-lg"
                        onclick={handleRemoveFromPlaylist}
                        title="Remove from this playlist"
                    >
                        <CircleMinus size={18} />
                    </Button>
                {/if}

                <Button
                    variant="outline"
                    size="icon-lg"
                    class="text-red-500  hover:bg-red-700/10 hover:text-red-600 "
                    onclick={handleDeleteSelected}
                    title="Delete from library"
                >
                    <Trash2 size={20} />
                </Button>

                <Button
                    variant="outline"
                    size="icon-lg"
                    class="dropdown-trigger"
                    onclick={(e) =>
                        (groupMenuOpen = {
                            type: "anchor",
                            anchor: e.currentTarget as HTMLElement,
                        })}
                    title="Selection options"
                    aria-haspopup="menu"
                    aria-expanded={groupMenuOpen != null}
                >
                    <Ellipsis size={20} />
                </Button>
            </div>
        {:else}
            <button
                type="button"
                class=" flex h-16 w-16 items-center justify-center rounded-full bg-[{accentColor}] text-accent-foreground shadow-lg shadow-accent/20 transition-all hover:scale-105 hover:bg-{accentColor}/80 active:scale-95 {focusRing}"
                style:background-color={accentColor}
                onclick={handleMainPlay}
                title={isCurrentCollectionPlaying ? "Pause" : "Play"}
            >
                {#if isCurrentCollectionPlaying}
                    <Pause size={24} fill="var(--color-accent-foreground)" />
                {:else}
                    <Play size={24} fill="var(--color-accent-foreground)" />
                {/if}
            </button>

            <Button
                variant="ghost"
                size="icon-2xl"
                class="flex h-14 w-14 items-center justify-center rounded-full transition-colors hover:text-white {focusRing} {player.shuffleEnabled
                    ? 'text-accent'
                    : ''}"
                onclick={player.toggleShuffle}
                title="Shuffle play"
            >
                <Shuffle size={30} />
            </Button>

            <Button
                variant="ghost"
                size="icon-2xl"
                class="dropdown-trigger flex h-9 w-9 items-center justify-center rounded-full transition-colors hover:text-white {focusRing}"
                onclick={(e) =>
                    (actionMenuOpen = {
                        type: "anchor",
                        anchor: e.currentTarget as HTMLElement,
                    })}
                title="More options"
                aria-haspopup="menu"
            >
                <Ellipsis size={22} />
            </Button>
            {#if actionMenuOpen}
                <DropdownMenu
                    position={actionMenuOpen}
                    items={actionMenuItems}
                    onClose={() => (actionMenuOpen = null)}
                />
            {/if}

            <div class="flex-1"></div>
        {/if}

        <div class="relative">
            <button
                bind:this={settingsBtn}
                type="button"
                class="flex h-9 items-center gap-1.5 rounded-full px-3 text-[13px] font-medium transition-colors hover:bg-white/5 hover:text-white {focusRing}"
                onclick={() => (settingsOpen = !settingsOpen)}
                title="Table view settings"
                aria-haspopup="true"
                aria-expanded={settingsOpen}
            >
                <SlidersHorizontal size={15} />
                <span class="hidden sm:inline">View</span>
            </button>

            {#if settingsOpen}
                <div
                    bind:this={settingsPanel}
                    class="absolute right-0 top-full z-20 mt-2 w-60 rounded-2xl border border-white/10 bg-card/50 p-3 shadow-lg backdrop-blur-md"
                    role="dialog"
                    title="Table view settings"
                >
                    <p
                        class="mb-2 px-1 text-[11px] font-semibold uppercase tracking-wider text-zinc-500"
                    >
                        Density
                    </p>
                    <div class="mb-4 flex gap-1 rounded-lg bg-white/5 p-1">
                        {#each ["compact", "relaxed"] as mode}
                            <button
                                type="button"
                                class="flex-1 rounded-md py-1.5 text-[13px] capitalize transition-colors {density ===
                                mode
                                    ? 'bg-white text-zinc-900'
                                    : 'text-zinc-300 hover:text-white'} {focusRing}"
                                onclick={() =>
                                    (density = mode as "relaxed" | "compact")}
                            >
                                {mode}
                            </button>
                        {/each}
                    </div>

                    {#if canToggleColumns}
                        <p
                            class="mb-1 px-1 text-[11px] font-semibold uppercase tracking-wider text-zinc-500"
                        >
                            Columns
                        </p>
                        <div class="flex flex-col">
                            {#each COLUMN_ORDER as key (key)}
                                {#if !COLUMN_META[key].locked}
                                    <label
                                        class="flex cursor-pointer items-center justify-between rounded-md px-1.5 py-1.5 text-[13px] text-zinc-200 hover:bg-white/5"
                                    >
                                        <span
                                            >{COLUMN_META[key]
                                                .settingsLabel}</span
                                        >
                                        <input
                                            type="checkbox"
                                            class="h-4 w-4 rounded accent-emerald-400 {focusRing}"
                                            checked={columns[key].visible}
                                            onchange={() =>
                                                (columns[key].visible =
                                                    !columns[key].visible)}
                                        />
                                    </label>
                                {/if}
                            {/each}
                        </div>
                    {/if}
                </div>
            {/if}
        </div>
    </div>

    <!-- ================================ TABLE ================================= -->
    <div
        class="mask-container overflow-x-scroll w-full"
        role="table"
        title="Track list"
    >
        <!-- header -->
        <div
            role="row"
            class="grid border-b-2 border-white/10 mx-2 mb-2 font-medium uppercase tracking-wide text-zinc-300 text-sm sm:px-3"
            style="grid-template-columns:{gridTemplate}"
        >
            {#each visibleColumnKeys as key (key)}
                {@const meta = COLUMN_META[key]}
                <div
                    role="columnheader"
                    aria-sort={sortKey === key
                        ? sortDir === "asc"
                            ? "ascending"
                            : "descending"
                        : "none"}
                    class="group relative flex items-center py-2.5 pr-3 {key ===
                    'duration'
                        ? 'justify-end'
                        : key === 'index'
                          ? 'justify-center'
                          : 'justify-start'}"
                >
                    {#if key === "index"}
                        <span>#</span>
                    {:else if key === "duration"}
                        <button
                            type="button"
                            class="flex items-center {focusRing}"
                            onclick={() => toggleSort(key)}
                            title="Sort by duration"
                        >
                            <Clock size={14} />
                        </button>
                    {:else}
                        <button
                            type="button"
                            class="flex items-center gap-1 rounded {meta.sortable
                                ? 'hover:text-zinc-200'
                                : 'cursor-default'} {focusRing}"
                            onclick={() => toggleSort(key)}
                            disabled={!meta.sortable}
                        >
                            <span>{meta.label}</span>
                            {#if sortKey === key}
                                {#if sortDir === "asc"}
                                    <ChevronUp size={13} />
                                {:else}
                                    <ChevronDown size={13} />
                                {/if}
                            {/if}
                        </button>
                    {/if}

                    {#if isColumnResizable(key)}
                        <div
                            class="absolute right-0 top-0 h-full w-3 cursor-col-resize opacity-0 transition-opacity group-hover:opacity-100"
                            onpointerdown={(e) => startResize(key, e)}
                            role="presentation"
                        >
                            <div class="mx-auto h-full w-px bg-white/25"></div>
                        </div>
                    {/if}
                </div>
            {/each}
        </div>

        <!-- rows -->
        <div class="mt-1 w-full overflow-x-scroll px-2">
            {#if orderedTracks.length > 0}
                <VList
                    class="vlist scroll-smooth"
                    data={orderedTracks}
                    style="height: {VListHeight}px;"
                    getKey={(t) => t.id}
                >
                    {#snippet children(track, i)}
                        {@const active = player.currentTrack?.id === track.id}
                        {@const selected = selectedIds.has(track.id)}
                        <div
                            role="row"
                            tabindex="0"
                            aria-selected={selected}
                            class="group relative grid items-center rounded-2xl px-2 transition-colors text-neutral-300 font-satoshi font-medium text-sm hover:bg-neutral-600/20 hover:shadow-lg {density ===
                            'compact'
                                ? 'py-1.5 my-0.5'
                                : 'py-2 my-1'} {selected
                                ? 'bg-neutral-600/15'
                                : ''}"
                            style="grid-template-columns:{gridTemplate}"
                            onclick={(e) => handleRowClick(e, track, i)}
                            ondblclick={() => handleRowActivate(track, i)}
                            onkeydown={(e) => {
                                if (e.key === "Enter" || e.key === " ") {
                                    e.preventDefault();
                                    handleRowActivate(track, i);
                                }
                            }}
                            onauxclick={(e) => {
                                if (e.button === 1) {
                                    e.preventDefault();
                                    player.enqueueEnd(track);
                                }
                            }}
                            oncontextmenu={(e) => {
                                e.preventDefault();
                                const pos: MenuPosition = {
                                    type: "coordinates",
                                    x: e.clientX,
                                    y: e.clientY,
                                };
                                if (
                                    selectedIds.has(track.id) &&
                                    selectedIds.size > 1
                                ) {
                                    groupMenuOpen = pos;
                                } else {
                                    selectedIds = new Set([track.id]);
                                    lastSelectedIndex = i;
                                    rowMenuOpen = [pos, track];
                                }
                            }}
                        >
                            {#each visibleColumnKeys as key (key)}
                                <div
                                    role="gridcell"
                                    class="flex min-w-0 items-center pr-3 {key ===
                                    'duration'
                                        ? 'justify-end'
                                        : key === 'index'
                                          ? 'justify-center'
                                          : 'justify-start'}"
                                >
                                    {#if key === "index"}
                                        <button
                                            type="button"
                                            class="relative flex h-9 w-9 items-center justify-center rounded {active
                                                ? 'text-emerald-400'
                                                : ''} {focusRing}"
                                            onclick={() =>
                                                handleRowActivate(track, i)}
                                            title={active && player.isPlaying
                                                ? "Pause"
                                                : "Play"}
                                        >
                                            {#if active && player.isPlaying}
                                                <div
                                                    class="absolute inset-0 flex items-end justify-between px-1"
                                                >
                                                    <PlayingVisualizer />
                                                </div>
                                            {:else}
                                                <span class="group-hover:hidden"
                                                    >{i + 1}</span
                                                >
                                                <Play
                                                    size={20}
                                                    class="hidden text-gray-300 fill-gray-300 group-hover:block"
                                                />
                                            {/if}
                                        </button>
                                    {:else if key === "title"}
                                        <div
                                            class="flex min-w-0 items-center gap-3"
                                        >
                                            {#if density !== "compact"}
                                                {#if track.cover_art}
                                                    <img
                                                        src={store.getImageSrc(
                                                            track.cover_art,
                                                        )}
                                                        alt=""
                                                        class="h-14 w-14 shrink-0 rounded-lg object-cover"
                                                        loading="lazy"
                                                    />
                                                {:else}
                                                    <div
                                                        class="h-12 w-12 shrink-0 rounded-lg bg-zinc-800 flex items-center justify-center"
                                                    >
                                                        <Music2 size={20} />
                                                    </div>
                                                {/if}
                                            {/if}
                                            <div class="min-w-0">
                                                <a
                                                    class="block max-w-full truncate text-lg {active
                                                        ? 'font-extrabold text-white'
                                                        : 'text-zinc-100  font-medium'} hover:underline {focusRing}"
                                                    href="/library/track/{track.id}"
                                                >
                                                    {track.title}
                                                </a>
                                                <div
                                                    class="truncate text-stone-400"
                                                >
                                                    {#each track.artists as artist, ai (artist.id)}
                                                        {#if ai > 0}
                                                            <span>, </span>
                                                        {/if}
                                                        <a
                                                            href="/library/artists/{artist.id}"
                                                            class="rounded hover:text-white hover:underline {focusRing}"
                                                            >{artist.name}</a
                                                        >
                                                    {/each}
                                                </div>
                                            </div>
                                        </div>
                                    {:else if key === "album"}
                                        <a
                                            href="/library/albums/{track.album
                                                .id}"
                                            class="truncate rounded hover:text-white hover:underline {focusRing}"
                                        >
                                            {track.album.name}
                                        </a>
                                    {:else if key === "dateAdded"}
                                        <span class="truncate"
                                            >{formatDateAdded(
                                                track.added_at,
                                            )}</span
                                        >
                                    {:else if key === "duration"}
                                        <div class="flex items-center gap-3">
                                            <button
                                                type="button"
                                                class="hidden group-hover:flex {track.is_favorite
                                                    ? 'flex!'
                                                    : ''} {track.is_favorite
                                                    ? 'text-rose-600 fill-rose-600'
                                                    : 'text-gray-300'}  hover:text-secondary transition-colors {focusRing}"
                                                onclick={() =>
                                                    toggleFavorite(track)}
                                                title={track.is_favorite
                                                    ? "Remove from Liked Songs"
                                                    : "Save to Liked Songs"}
                                            >
                                                <Heart
                                                    size={16}
                                                    class={track.is_favorite
                                                        ? "text-rose-600 fill-rose-600"
                                                        : "text-gray-300"}
                                                />
                                            </button>
                                            <span class="text-sm"
                                                >{formatDurationColon(
                                                    track.duration_seconds,
                                                )}</span
                                            >
                                        </div>
                                    {/if}
                                </div>
                            {/each}

                            <div
                                role="gridcell"
                                class="relative flex items-center justify-center"
                            >
                                <button
                                    type="button"
                                    class="flex h-8 w-8 items-center justify-center rounded-full opacity-0 transition-all hover:bg-white/10 hover:text-white group-hover:opacity-100 {focusRing}"
                                    onclick={(e) => {
                                        rowMenuOpen = [
                                            {
                                                type: "anchor",
                                                anchor: e.currentTarget as HTMLElement,
                                            },
                                            track,
                                        ];
                                    }}
                                    title="More options for {track.title}"
                                    aria-haspopup="menu"
                                    aria-expanded={rowMenuOpen != null}
                                >
                                    <Ellipsis size={18} />
                                </button>
                            </div>
                        </div>
                    {/snippet}
                </VList>
            {:else}
                <div
                    class="flex flex-col items-center gap-4 py-16 text-center text-zinc-400"
                >
                    <Disc size={50} />
                    <p class="">No tracks here yet.</p>
                </div>
            {/if}
        </div>
    </div>
</div>

{#if rowMenuOpen}
    <TrackMenu
        position={rowMenuOpen[0]}
        track={rowMenuOpen[1]}
        {context}
        onClose={() => {
            rowMenuOpen = null;
        }}
    />
{/if}

{#if groupMenuOpen}
    <DropdownMenu
        position={groupMenuOpen}
        items={groupMenuItems}
        onClose={() => (groupMenuOpen = null)}
    />
{/if}

{#if showEditDialog && context.type === "Playlist"}
    <EditPlaylistDialog
        bind:open={showEditDialog}
        playlistId={context.id ?? 0}
        name={context.name}
        coverArt={context.coverArt}
    />
{/if}

{#if showEditDialog && context.type === "Album"}
    <EditAlbumDialog
        bind:open={showEditDialog}
        albumId={context.id ?? 0}
        name={context.name}
        coverArt={context.coverArt}
    />
{/if}

{#if showEditDialog && context.type === "Artist"}
    <EditArtistDialog
        bind:open={showEditDialog}
        artistId={context.id ?? 0}
        name={context.name}
        profileImage={context.profileImage}
        bannerImage={context.bannerImage}
    />
{/if}
