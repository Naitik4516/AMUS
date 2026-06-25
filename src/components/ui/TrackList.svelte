<script lang="ts">
    import Icon from "./Icon.svelte";
    import DropdownMenu from "./DropdownMenu.svelte";
    import TrackMenu from "./TrackMenu.svelte";
    import type { Track } from "$lib/types";
    import { getImageUrl } from "$lib/utils";
    import { Heart } from "@lucide/svelte";
    import { player } from "$lib/player.svelte";
    import { invoke } from "@tauri-apps/api/core";

    type ColumnKey = (typeof COLUMN_ORDER)[number];

    const COLUMN_ORDER = [
        "index",
        "title",
        "album",
        "dateAdded",
        "duration",
    ] as const;

    type Context = "playlist" | "album" | "artist" | "liked";

    type MenuItem = {
        label: string;
        icon?: string;
        onClick?: () => void;
        href?: string;
        danger?: boolean;
        disabled?: boolean;
        type?: string;
        items?: MenuItem[];
    };

    interface TrackTableProps {
        tracks: Track[];
        context?: Context;
        visibleColumns?: ColumnKey[] | null;
        canEdit?: boolean;
        canSort?: boolean;
        canToggleColumns?: boolean;
        playlists?: { id: number; name: string }[];
        extraActions?: MenuItem[];
        playlistId?: number | null;
    }

    let {
        tracks = [],
        context = "playlist",
        visibleColumns = null,
        canEdit = false,
        canSort = true,
        canToggleColumns = true,
        playlists = [],
        extraActions = [],
        playlistId = null,
    }: TrackTableProps = $props();

    let menuPlaylists = $state<{ id: number; name: string }[]>([]);

    $effect(() => {
        invoke("list_playlists")
            .then((pl) => {
                menuPlaylists = pl as { id: number; name: string }[];
            })
            .catch(() => {
                menuPlaylists = [];
            });
    });

    function onPlaylistsChanged() {
        invoke("list_playlists")
            .then((pl) => {
                menuPlaylists = pl as { id: number; name: string }[];
            })
            .catch(() => {
                menuPlaylists = [];
            });
    }

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
            width: 380,
            minWidth: 220,
            maxWidth: 640,
            sortable: true,
            locked: true,
            resizable: true,
        },
        album: {
            label: "Album",
            settingsLabel: "Album",
            width: 220,
            minWidth: 120,
            maxWidth: 420,
            sortable: true,
            locked: false,
            resizable: true,
        },
        dateAdded: {
            label: "Date added",
            settingsLabel: "Date added",
            width: 160,
            minWidth: 110,
            maxWidth: 280,
            sortable: true,
            locked: false,
            resizable: true,
        },
        duration: {
            label: "",
            settingsLabel: "Duration",
            width: 90,
            minWidth: 64,
            maxWidth: 64,
            sortable: true,
            locked: false,
            resizable: false,
            icon: "clock",
        },
    };

    const CONTEXT_DEFAULT_COLUMNS: Record<string, ColumnKey[]> = {
        playlist: ["index", "title", "album", "dateAdded", "duration"],
        liked: ["index", "title", "album", "dateAdded", "duration"],
        album: ["index", "title", "duration"],
        artist: ["index", "title", "album", "duration"],
    };

    let columns = $state(
        Object.fromEntries(
            COLUMN_ORDER.map((key) => [
                key,
                {
                    visible: (
                        visibleColumns ??
                        CONTEXT_DEFAULT_COLUMNS[context] ??
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
    let actionMenuBtn = $state<HTMLButtonElement | null>(null);
    let openRowMenuId = $state<number | null>(null);
    const rowMenuButtons: Record<number, HTMLButtonElement> = {};

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
        if (!sortKey) return tracks;
        const key = sortKey;
        const sorted = [...tracks].sort((a, b) => compareTracks(a, b, key));
        return sortDir === "desc" ? sorted.reverse() : sorted;
    });

    function toggleSort(key: ColumnKey) {
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
                    ? `minmax(${columns[key].width}px, 1fr)`
                    : `${columns[key].width}px`,
            )
            .join(" ") + " 40px",
    );

    function startResize(key: ColumnKey, event: PointerEvent) {
        event.preventDefault();
        const startX = event.clientX;
        const startWidth = columns[key].width;
        const meta = COLUMN_META[key];

        function onMove(e: PointerEvent) {
            const next = startWidth + (e.clientX - startX);
            columns[key].width = Math.min(
                meta.maxWidth,
                Math.max(meta.minWidth, next),
            );
        }
        function onUp() {
            window.removeEventListener("pointermove", onMove);
            window.removeEventListener("pointerup", onUp);
        }
        window.addEventListener("pointermove", onMove);
        window.addEventListener("pointerup", onUp);
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

    function sourceType(): string {
        return context === "liked" ? "favorites" : context;
    }

    function handleMainPlay() {
        const first = orderedTracks[0];
        if (!first) return;
        if (
            player.isPlaying &&
            orderedTracks.some((x) => player.currentTrack?.id === x.id)
        )
            player.togglePlay();
        else player.play(first, orderedTracks, sourceType() as any);
    }

    function handleRowActivate(track: Track) {
        if (player.currentTrack?.id === track.id && player.isPlaying)
            player.togglePlay();
        else player.play(track, orderedTracks, sourceType() as any);
    }

    let actionMenuItems = $derived.by(() => {
        const items: any[] = [
            {
                label: "Add all to queue",
                icon: "list-plus",
                onClick: () => {
                    for (const t of orderedTracks) {
                        player.addToQueue(t);
                    }
                },
            },
        ];
        if (extraActions.length) {
            items.push({ type: "separator" }, ...extraActions);
        }
        return items;
    });

    // Click-outside / Escape for the column-settings popover
    // (row & action menus handle this internally inside DropdownMenu)
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

    const focusRing =
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-emerald-400/70 focus-visible:ring-offset-2 focus-visible:ring-offset-zinc-950";
</script>

<div class="w-full rounded-b-2xl px-2 backdrop-blur-lg bg-black/5">
    <!-- ============================== ACTION BAR ============================== -->
    <div class="relative -mx-1 mb-2 sm:-mx-2">
        <div class="pointer-events-none absolute inset-0"></div>
        <div class="relative flex items-center gap-5 px-3 py-4 sm:px-4">
            <button
                type="button"
                class="flex h-14 w-14 items-center justify-center rounded-full bg-emerald-400 text-black shadow-lg shadow-emerald-400/20 transition-all hover:scale-105 hover:bg-emerald-300 active:scale-95 {focusRing}"
                onclick={handleMainPlay}
                aria-label={player.isPlaying && player.currentTrack
                    ? "Pause"
                    : "Play"}
            >
                <Icon
                    name={player.isPlaying && player.currentTrack
                        ? "pause"
                        : "play"}
                    size={24}
                />
            </button>

            <button
                type="button"
                class="flex h-9 w-9 items-center justify-center rounded-full transition-colors hover:text-white {focusRing}"
                onclick={async () => {
                    if (!player.shuffle) await player.toggleShuffle();
                    await player.play(
                        orderedTracks[0],
                        orderedTracks,
                        sourceType() as any,
                    );
                }}
                aria-label="Shuffle play"
            >
                <Icon name="shuffle" size={22} />
            </button>

            <div>
                <button
                    bind:this={actionMenuBtn}
                    type="button"
                    class="flex h-9 w-9 items-center justify-center rounded-full transition-colors hover:text-white {focusRing}"
                    onclick={() => (actionMenuOpen = !actionMenuOpen)}
                    aria-label="More options"
                    aria-haspopup="menu"
                    aria-expanded={actionMenuOpen}
                >
                    <Icon name="more-horizontal" size={22} />
                </button>
                {#if actionMenuOpen}
                    <DropdownMenu
                        items={actionMenuItems}
                        anchorEl={actionMenuBtn}
                        placement="bottom-start"
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
                    <Icon name="sliders" size={15} />
                    <span class="hidden sm:inline">View</span>
                </button>

                {#if settingsOpen}
                    <div
                        bind:this={settingsPanel}
                        class="absolute right-0 top-full z-50 mt-2 w-60 rounded-xl border border-white/10 bg-zinc-900/95 p-3 shadow-2xl shadow-black/60 backdrop-blur-md"
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
                                            | "relaxed"
                                            | "compact")}
                                >
                                    {mode}
                                </button>
                            {/each}
                        </div>

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
                            <Icon name="clock" size={14} />
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
                                <Icon
                                    name={sortDir === "asc"
                                        ? "chevron-up"
                                        : "chevron-down"}
                                    size={13}
                                />
                            {/if}
                        </button>
                    {/if}

                    {#if meta.resizable}
                        <div
                            class="absolute right-0 top-1/2 h-4 w-3 -translate-y-1/2 cursor-col-resize opacity-0 transition-opacity group-hover:opacity-100"
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
                    ondblclick={() => handleRowActivate(track)}
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
                                    onclick={() => handleRowActivate(track)}
                                    aria-label={active && player.isPlaying
                                        ? "Pause"
                                        : "Play"}
                                >
                                    {#if active && player.isPlaying}
                                        <span
                                            class="flex items-end gap-0.5 group-hover:hidden"
                                        >
                                            <span
                                                class="eq-bar"
                                                style="animation-delay:0ms"
                                            ></span>
                                            <span
                                                class="eq-bar"
                                                style="animation-delay:160ms"
                                            ></span>
                                            <span
                                                class="eq-bar"
                                                style="animation-delay:300ms"
                                            ></span>
                                        </span>
                                        <Icon
                                            name="pause"
                                            size={14}
                                            class="hidden text-white group-hover:block"
                                        />
                                    {:else}
                                        <span class="group-hover:hidden"
                                            >{i + 1}</span
                                        >
                                        <Icon
                                            name="play"
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
                                                    class="h-11 w-11 shrink-0 rounded object-cover"
                                                    loading="lazy"
                                                />
                                            {/await}
                                        {:else}
                                            <div
                                                class="h-12 w-12 shrink-0 rounded bg-zinc-800"
                                            ></div>
                                        {/if}
                                    {/if}
                                    <div class="min-w-0">
                                        <button
                                            type="button"
                                            class="block max-w-full truncate rounded text-left text-base font-medium {active
                                                ? 'text-emerald-400'
                                                : 'text-zinc-50'} hover:underline {focusRing}"
                                            onclick={() =>
                                                handleRowActivate(track)}
                                        >
                                            {track.title}
                                        </button>
                                        <div class="truncate text-stone-400">
                                            {#each track.artists as artist, ai (artist.id)}
                                                {#if ai > 0}
                                                    <span>, </span>
                                                {/if}
                                                <a
                                                    href="/library/artist/{artist.id}"
                                                    class="rounded hover:text-white hover:underline {focusRing}"
                                                    onclick={(e) =>
                                                        e.stopPropagation()}
                                                    >{artist.name}</a
                                                >
                                            {/each}
                                        </div>
                                    </div>
                                </div>
                            {:else if key === "album"}
                                <a
                                    href="/library/album/{track.album.id}"
                                    class="truncate rounded hover:text-white hover:underline {focusRing}"
                                    onclick={(e) => e.stopPropagation()}
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
                                        class="hidden hover:text-white group-hover:flex {track.is_favorite
                                            ? 'flex!'
                                            : ''} {focusRing}"
                                        onclick={(e) => {
                                            e.stopPropagation();
                                            player.toggleFavorite(track);
                                        }}
                                        aria-label={track.is_favorite
                                            ? "Remove from Liked Songs"
                                            : "Save to Liked Songs"}
                                    >
                                        <Heart
                                            fill={track.is_favorite
                                                ? "red"
                                                : ""}
                                            size={16}
                                            class={track.is_favorite
                                                ? "text-emerald-400"
                                                : ""}
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
                            class="flex h-8 w-8 items-center justify-center rounded-full opacity-0 transition-all hover:bg-white/10 hover:text-white group-hover:opacity-100 {openRowMenuId ===
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
                            <Icon name="more-horizontal" size={18} />
                        </button>
                        {#if openRowMenuId === track.id}
                            <TrackMenu
                                {track}
                                {context}
                                playlists={menuPlaylists}
                                {playlistId}
                                exclude={context !== "playlist"
                                    ? ["removeFromPlaylist"]
                                    : []}
                                anchorEl={rowMenuButtons[track.id]}
                                onClose={() => (openRowMenuId = null)}
                                onPlaylistsChanged={onPlaylistsChanged}
                            />
                        {/if}
                    </div>
                </div>
            {/each}

            {#if orderedTracks.length === 0}
                <div
                    class="flex flex-col items-center gap-2 py-16 text-center text-zinc-500"
                >
                    <Icon name="disc" size={28} />
                    <p class="text-sm">No tracks here yet.</p>
                </div>
            {/if}
        </div>
    </div>
</div>

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
