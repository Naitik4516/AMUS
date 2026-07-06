<script lang="ts">
    import DropdownMenu from "./Menu/DropdownMenu.svelte";
    import TrackMenu from "./Menu/TrackMenu.svelte";
    import type { Track } from "$lib/types";
    import { getImageUrl } from "$lib/utils";
    import {
        Heart,
        Play,
        Pause,
        Ellipsis,
        Shuffle,
        Clock,
        ChevronUp,
        ChevronDown,
        SlidersHorizontal,
        Disc,
        Music2,
    } from "@lucide/svelte";
    import { player } from "$lib/player.svelte";
    import EditPlaylistDialog from "./Dialog/EditPlaylistDialog.svelte";
    import EditAlbumDialog from "./Dialog/EditAlbumDialog.svelte";
    import EditArtistDialog from "./Dialog/EditArtistDialog.svelte";
    import type { Context } from "$lib/types";
    import PlayingVisualizer from "./PlayingVisualizer.svelte";
    import { toast } from "svelte-sonner";
    import { invoke } from "@tauri-apps/api/core";
    import { invalidate } from "$app/navigation";

    type ColumnKey = (typeof COLUMN_ORDER)[number];

    const COLUMN_ORDER = [
        "index",
        "title",
        "album",
        "dateAdded",
        "duration",
    ] as const;

    interface TrackTableProps {
        tracks: Track[];
        context: Context;
        visibleColumns?: ColumnKey[] | null;
        canEdit?: boolean;
        canSort?: boolean;
        canToggleColumns?: boolean;
    }

    let {
        tracks = [],
        context,
        visibleColumns = null,
        canEdit = true,
        canSort = true,
        canToggleColumns = true,
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
            minWidth: 160,
            maxWidth: 640,
            sortable: true,
            locked: false,
            resizable: true,
        },
        album: {
            label: "Album",
            settingsLabel: "Album",
            width: 300,
            minWidth: 120,
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
    let actionMenuOpen = $state(false);
    let showEditDialog = $state(false);
    let openRowMenuId = $state<number | null>(null);
    let rowMenuButtons = $state<Record<number, HTMLButtonElement>>({});

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
        if (!sortKey) return tracks;
        const key = sortKey;
        const sorted = [...tracks].sort((a, b) => compareTracks(a, b, key));
        return sortDir === "desc" ? sorted.reverse() : sorted;
    });

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
        }
        resizer.addEventListener("pointermove", onMove);
        resizer.addEventListener("pointerup", onUp);
    }

    function formatDuration(totalSeconds: number) {
        const m = Math.floor(totalSeconds / 60);
        const s = Math.floor(totalSeconds % 60)
            .toString()
            .padStart(2, "0");
        return `${m}:${s}`;
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

    function handleMainPlay() {
        if (!orderedTracks) return;
        if (
            player.isPlaying &&
            orderedTracks.some((x) => player.currentTrack?.id === x.id)
        )
            player.playPause();
        else player.play(orderedTracks, context, 0, context.name);
    }

    function handleRowActivate(track: Track, index: number) {
        if (player.currentTrack?.id === track.id && player.isPlaying)
            player.playPause();
        else player.play(orderedTracks, context, index, context.name);
    }

    async function toggleFavorite(track: Track) {
        try {
            await invoke<boolean>("toggle_favorite", {
                id: track.id,
            });
            if (context.type === "Favorites") {
                invalidate(context.type);
            } else {
                invalidate(`${context.type}:${context.id ?? ""}`);
            }
        } catch (e) {
            console.error("Failed to toggle favorite", e);
        }
    }

    let actionMenuItems = $derived.by(() => {
        const items: any[] = [
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

    let isCurrentCollectionPlaying = $derived(
        player.isPlaying &&
            player.currentTrack &&
            orderedTracks.some((x) => player.currentTrack?.id === x.id),
    );

    const focusRing =
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/70 focus-visible:ring-offset-2 focus-visible:ring-offset-zinc-950";
</script>

<div class="w-full rounded-b-2xl px-2 pb-4 h-full">
    <!-- ============================== ACTION BAR ============================== -->
    <div>
        <div class="pointer-events-none absolute inset-0"></div>
        <div class="relative flex items-center gap-5 px-3 py-4 sm:px-4">
            <button
                type="button"
                class="flex h-16 w-16 items-center justify-center rounded-full bg-accent text-accent-foreground shadow-lg shadow-accent/20 transition-all hover:scale-105 hover:bg-accent/80 active:scale-95 {focusRing}"
                onclick={handleMainPlay}
                aria-label={isCurrentCollectionPlaying ? "Pause" : "Play"}
            >
                {#if isCurrentCollectionPlaying}
                    <Pause size={24} fill="var(--color-accent-foreground)" />
                {:else}
                    <Play size={24} fill="var(--color-accent-foreground)" />
                {/if}
            </button>

            <button
                type="button"
                class="flex h-14 w-14 items-center justify-center rounded-full transition-colors hover:text-white {focusRing} {player.shuffleEnabled
                    ? 'text-accent'
                    : ''}"
                onclick={player.toggleShuffle}
                aria-label="Shuffle play"
            >
                <Shuffle size={30} />
            </button>

            <div class="relative">
                <button
                    type="button"
                    class="dropdown-trigger flex h-9 w-9 items-center justify-center rounded-full transition-colors hover:text-white {focusRing}"
                    onclick={() => (actionMenuOpen = !actionMenuOpen)}
                    aria-label="More options"
                    aria-haspopup="menu"
                    aria-expanded={actionMenuOpen}
                >
                    <Ellipsis size={22} />
                </button>

                {#if actionMenuOpen}
                    <DropdownMenu
                        items={actionMenuItems}
                        onClose={() => (actionMenuOpen = false)}
                    />
                {/if}
            </div>

            <div class="flex-1"></div>

            <div class="relative">
                <button
                    bind:this={settingsBtn}
                    type="button"
                    class="flex h-9 items-center gap-1.5 rounded-full px-3 text-[13px] font-medium transition-colors hover:bg-white/5 hover:text-white {focusRing}"
                    onclick={() => (settingsOpen = !settingsOpen)}
                    aria-label="Table view settings"
                    aria-haspopup="true"
                    aria-expanded={settingsOpen}
                >
                    <SlidersHorizontal size={15} />
                    <span class="hidden sm:inline">View</span>
                </button>

                {#if settingsOpen}
                    <div
                        bind:this={settingsPanel}
                        class="absolute right-0 top-full z-50 mt-2 w-60 rounded-2xl border border-white/10 bg-card/50 p-3 shadow-lg backdrop-blur-md"
                        role="dialog"
                        aria-label="Table view settings"
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
                                        (density = mode as
                                            "relaxed" | "compact")}
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
    </div>

    <!-- ================================ TABLE ================================= -->
    <div role="table" aria-label="Track list">
        <!-- header -->
        <div
            role="row"
            class="grid border-b border-white/10 px-2 font-medium uppercase tracking-wide text-zinc-300 text-sm sm:px-3"
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
                            aria-label="Sort by duration"
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
            <div></div>
        </div>

        <!-- rows -->
        <div class="mt-1 flex flex-col gap-1">
            {#each orderedTracks as track, i (track.id)}
                {@const active = player.currentTrack?.id === track.id}
                <div
                    role="row"
                    tabindex="0"
                    class="group relative grid items-center rounded-lg px-2 transition-colors text-neutral-300 text-sm hover:bg-white/6 {density ===
                    'compact'
                        ? 'py-1.5'
                        : 'py-3'}"
                    style="grid-template-columns:{gridTemplate}"
                    ondblclick={() => handleRowActivate(track, i)}
                    onauxclick={(e) => {
                        if (e.button === 1) {
                            e.preventDefault();
                            player.enqueueEnd(track);
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
                                    class="relative flex h-8 w-8 items-center justify-center rounded {active
                                        ? 'text-emerald-400'
                                        : ''} {focusRing}"
                                    onclick={() => handleRowActivate(track, i)}
                                    aria-label={active && player.isPlaying
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
                                            size={13}
                                            class="hidden text-white group-hover:block"
                                        />
                                    {/if}
                                </button>
                            {:else if key === "title"}
                                <div class="flex min-w-0 items-center gap-3">
                                    {#if density !== "compact"}
                                        {#if track.cover_art}
                                            {#await getImageUrl(track.cover_art) then url}
                                                <img
                                                    src={url}
                                                    alt=""
                                                    class="h-12 w-12 shrink-0 rounded-lg object-cover"
                                                    loading="lazy"
                                                />
                                            {/await}
                                        {:else}
                                            <div
                                                class="h-12 w-12 shrink-0 rounded-lg bg-zinc-800 flex items-center justify-center"
                                            >
                                                <Music2 size={20} />
                                            </div>
                                        {/if}
                                    {/if}
                                    <div class="min-w-0">
                                        <button
                                            type="button"
                                            class="block max-w-full truncate rounded text-left text-base font-medium {active
                                                ? 'text-accent'
                                                : 'text-zinc-50'} hover:underline {focusRing}"
                                            onclick={() =>
                                                handleRowActivate(track, i)}
                                        >
                                            {track.title}
                                        </button>
                                        <div class="truncate text-stone-400">
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
                                    href="/library/albums/{track.album.id}"
                                    class="truncate rounded hover:text-white hover:underline {focusRing}"
                                >
                                    {track.album.name}
                                </a>
                            {:else if key === "dateAdded"}
                                <span class="truncate"
                                    >{formatDateAdded(track.added_at)}</span
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
                                        onclick={() => toggleFavorite(track)}
                                        aria-label={track.is_favorite
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
                                        >{formatDuration(
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
                            bind:this={rowMenuButtons[track.id]}
                            type="button"
                            class="dropdown-trigger flex h-8 w-8 items-center justify-center rounded-full opacity-0 transition-all hover:bg-white/10 hover:text-white group-hover:opacity-100 {openRowMenuId ===
                            track.id
                                ? 'opacity-100'
                                : ''} {focusRing}"
                            onclick={() =>
                                (openRowMenuId =
                                    openRowMenuId === track.id
                                        ? null
                                        : track.id)}
                            aria-label="More options for {track.title}"
                            aria-haspopup="menu"
                            aria-expanded={openRowMenuId === track.id}
                        >
                            <Ellipsis size={18} />
                        </button>
                        {#if openRowMenuId === track.id}
                            <TrackMenu
                                {track}
                                {context}
                                onClose={() => (openRowMenuId = null)}
                            />
                        {/if}
                    </div>
                </div>
            {/each}

            {#if orderedTracks.length === 0}
                <div
                    class="flex flex-col items-center gap-2 py-16 text-center text-zinc-500"
                >
                    <Disc size={28} />
                    <p class="text-sm">No tracks here yet.</p>
                </div>
            {/if}
        </div>
    </div>
</div>

{#if showEditDialog && context.type === "Playlist"}
    <EditPlaylistDialog
        bind:open={showEditDialog}
        playlistId={context.id ?? 0}
        name={context.name}
        coverArt={context.coverArt}
        onClose={() => (showEditDialog = false)}
    />
{/if}

{#if showEditDialog && context.type === "Album"}
    <EditAlbumDialog
        bind:open={showEditDialog}
        albumId={context.id ?? 0}
        name={context.name}
        coverArt={context.coverArt}
        onClose={() => (showEditDialog = false)}
    />
{/if}

{#if showEditDialog && context.type === "Artist"}
    <EditArtistDialog
        bind:open={showEditDialog}
        artistId={context.id ?? 0}
        name={context.name}
        profileImage={context.profileImage}
        bannerImage={context.bannerImage}
        onClose={() => (showEditDialog = false)}
    />
{/if}

<style>
    .eq-bar {
        width: 3px;
        height: 5px;
        background: currentColor;
        border-radius: 1px;
        animation: eq-bounce 0.9s ease-in-out infinite;
    }
    @keyframes eq-bounce {
        0%,
        100% {
            height: 4px;
        }
        50% {
            height: 13px;
        }
    }
    @media (prefers-reduced-motion: reduce) {
        .eq-bar {
            animation: none;
            height: 9px;
        }
    }
</style>
