<script lang="ts">
    import {
        Music,
        Trash2,
        Pen,
        Ellipsis,
        Clock,
        Play,
        Plus,
        User,
    } from "@lucide/svelte";
    import type { Track, Playlist } from "$lib/types.d.ts";
    import Menu from "$components/ui/Menu.svelte";
    import MenuItem from "$components/ui/MenuItem.svelte";
    import CircleIconButton from "$components/ui/Button/CircleIconButton.svelte";
    import { formatDuration, getCoverUrl } from "$lib/utils";
    import { getPlaylists } from "$lib/data.svelte";
    import { player } from "$lib/player.svelte";
    import type { Snippet } from "svelte";
    import { goto } from "$app/navigation";

    let {
        tracks,
        name,
        coverArt,
        otherMenuItems,
        coverSnippet,
        subtitle,
    }: {
        tracks: Track[];
        name: string;
        coverArt: string | string[] | null;
        otherMenuItems?: Snippet;
        coverSnippet?: Snippet;
        subtitle?: string;
    } = $props();

    // ── State ────────────────────────────────────────────────────────────────

    let playlists = $state<Playlist[]>([]);
    let showTrackOptions = $state(-1);
    let hovering = $state(-1);
    let heroHidden = $state(false);
    let scrollContainer = $state<HTMLDivElement | null>(null);

    // ── Derived ──────────────────────────────────────────────────────────────

    let totalDuration = $derived(
        tracks.reduce((sum, track) => sum + track.duration_seconds, 0),
    );

    // ── Effects ──────────────────────────────────────────────────────────────

    $effect(() => {
        getPlaylists().then((data) => {
            playlists = data.playlists;
        });
    });

    /**
     * Attach scroll listener to the inner scrollable container.
     * The container must have a fixed/max height for scrollTop to work —
     * apply `h-full` or `max-h-[...]` via the parent layout.
     */
    $effect(() => {
        const el = scrollContainer;
        if (!el) return;

        const onScroll = () => {
            const y = el.scrollTop;
            if (!heroHidden && y > 50) heroHidden = true;
            else if (heroHidden && y < 10) heroHidden = false;
        };

        el.addEventListener("scroll", onScroll, { passive: true });
        return () => el.removeEventListener("scroll", onScroll);
    });

    // ── Helpers ──────────────────────────────────────────────────────────────

    function playTrack(track: Track) {
        player.play(track, tracks);
    }

    function playFirst() {
        if (tracks.length > 0) player.play(tracks[0], tracks);
    }

    function closeTrackOptions() {
        showTrackOptions = -1;
    }
</script>

<div class="flex flex-col gap-10 p-8">
    <!-- ── Hero ──────────────────────────────────────────────────────────── -->
    <div
        class="grid transition-all duration-300 ease-in-out"
        style:grid-template-rows={heroHidden ? "0fr" : "1fr"}
    >
        <div class="overflow-hidden">
            <div class="p-5 flex gap-15 rounded-2xl items-end">
                <!-- Cover art -->
                <div
                    class="aspect-square w-50 rounded-xl overflow-hidden bg-neutral-800 relative shadow-2xl shrink-0"
                >
                    {#if coverSnippet}
                        {@render coverSnippet()}
                    {:else if !coverArt}
                        <div
                            class="absolute inset-0 flex items-center justify-center"
                        >
                            <Music size={48} class="text-neutral-700" />
                        </div>
                    {:else if typeof coverArt === "string"}
                        {#await getCoverUrl(coverArt)}
                            <div
                                class="absolute inset-0 bg-neutral-800 animate-pulse"
                            ></div>
                        {:then url}
                            <img
                                src={url}
                                alt={name}
                                class="w-full h-full object-cover"
                            />
                        {/await}
                    {:else}
                        <!-- Mosaic for multiple covers (max 4) -->
                        <div class="grid grid-cols-2 grid-rows-2 w-full h-full">
                            {#each coverArt.slice(0, 4) as art}
                                {#await getCoverUrl(art)}
                                    <div
                                        class="w-full h-full bg-neutral-800 animate-pulse"
                                    ></div>
                                {:then url}
                                    <img
                                        src={url}
                                        alt=""
                                        class="w-full h-full object-cover"
                                    />
                                {/await}
                            {/each}
                        </div>
                    {/if}
                </div>

                <!-- Title + meta -->
                <div class="flex flex-col gap-4 min-w-0">
                    <h1
                        class="text-3xl md:text-5xl lg:text-[6rem] font-black drop-shadow-lg truncate"
                    >
                        {name}
                    </h1>
                    <span class="text-gray-300">
                        {#if subtitle}
                            {subtitle}
                        {:else}
                            {tracks.length} songs, {formatDuration(
                                totalDuration,
                                true,
                            )}
                        {/if}
                    </span>
                </div>
            </div>
        </div>
    </div>

    <!-- ── Track list ────────────────────────────────────────────────────── -->
    <!--
        NOTE: This container needs a constrained height from the parent layout
        (e.g. h-full or max-h-[calc(100vh-...)]) for inner scrolling to work.
        Without it, the page scrolls instead and heroHidden never updates.
    -->
    <div
        bind:this={scrollContainer}
        role="list"
        class="overflow-y-auto overflow-x-hidden p-5 pb-0 rounded-2xl shadow-2xl bg-gray-900/40"
    >
        <!-- Sticky mini-header (visible when hero is collapsed) -->
        <div
            class="sticky -top-5 z-10 -mx-5 px-5 py-5 bg-gray-900/80 backdrop-blur-md rounded-t-2xl flex items-center gap-5 transition-opacity"
            class:opacity-0={!heroHidden}
            class:pointer-events-none={!heroHidden}
        >
            <CircleIconButton type="primary" size={4} onclick={playFirst}>
                <Play size={23} fill="black" />
            </CircleIconButton>
            <span class="text-3xl font-extrabold truncate">{name}</span>
        </div>

        <!-- Action bar (visible when hero is expanded) -->
        <div class="flex items-center gap-4 p-4 mb-2" class:hidden={heroHidden}>
            <CircleIconButton type="primary" size={5} onclick={playFirst}>
                <Play size={26} fill="black" />
            </CircleIconButton>
            <!--
                TODO: wire up edit and overflow actions via props or events
                once the parent pages implement them.
            -->
            <CircleIconButton type="secondary" onclick={() => {}}>
                <Pen size={16} fill="white" />
            </CircleIconButton>
            <CircleIconButton type="secondary" onclick={() => {}}>
                <Ellipsis size={16} fill="white" />
            </CircleIconButton>
        </div>

        <!-- Column headers -->
        <div
            class="grid grid-cols-24 items-center p-4 pb-0 text-gray-300 border-b-2 border-gray-800/60 font-semibold mb-2"
        >
            <span class="col-span-1 font-mono">#</span>
            <span class="col-span-13">Title</span>
            <span class="col-span-6">Album</span>
            <span class="col-span-4 font-mono text-right">Duration</span>
        </div>

        <!-- Empty state -->
        {#if tracks.length === 0}
            <div
                class="flex flex-col items-center justify-center py-16 text-gray-500"
            >
                <Music size={48} class="mb-4 opacity-20" />
                <p class="text-lg font-medium">No tracks yet</p>
                <p class="text-sm">Add songs to see them here.</p>
            </div>

            <!-- Track rows -->
        {:else}
            {#each tracks as track, index}
                <div
                    class="grid grid-cols-24 items-center p-4 hover:bg-gray-500/15 text-gray-300 group relative rounded-lg"
                    onmouseenter={() => (hovering = index)}
                    onmouseleave={() => (hovering = -1)}
                    role="listitem"
                >
                    <!-- Index / play button -->
                    <div class="col-span-1 flex items-center">
                        {#if hovering === index}
                            <button
                                class="w-5 transition-colors"
                                aria-label="Play {track.title}"
                                onclick={() => playTrack(track)}
                            >
                                <Play
                                    size={24}
                                    class="fill-secondary hover:fill-secondary/75"
                                />
                            </button>
                        {:else}
                            <span class="font-mono">{index + 1}</span>
                        {/if}
                    </div>

                    <!-- Title + artists -->
                    <div class="flex gap-4 items-center col-span-13 min-w-0">
                        {#if track.cover_art}
                            {#await getCoverUrl(track.cover_art)}
                                <div
                                    class="w-14 h-14 bg-neutral-800 animate-pulse rounded-lg shrink-0"
                                ></div>
                            {:then url}
                                <img
                                    src={url}
                                    alt=""
                                    class="w-14 h-14 object-cover rounded-lg shrink-0"
                                />
                            {/await}
                        {:else}
                            <div
                                class="w-14 h-14 bg-neutral-800 flex items-center justify-center rounded-lg shrink-0"
                            >
                                <Music size={24} />
                            </div>
                        {/if}

                        <div class="flex flex-col min-w-0">
                            <button
                                class="text-white font-bold hover:underline text-left truncate"
                                onclick={(e) => {
                                    e.stopPropagation();
                                    goto(`/library/trackdetails/${track.id}`);
                                }}
                            >
                                {track.title}
                            </button>
                            <span class="text-sm text-gray-400 truncate">
                                {#each track.artists as artist, i}
                                    {#if i > 0}{", "}{/if}<button
                                        class="hover:underline hover:text-gray-300"
                                        onclick={(e) => {
                                            e.stopPropagation();
                                            goto(
                                                `/library/artists/${artist.id}`,
                                            );
                                        }}>{artist.name}</button
                                    >
                                {/each}
                            </span>
                        </div>
                    </div>

                    <!-- Album -->
                    <div class="col-span-6 truncate text-sm">
                        {track.album?.name ?? "Unknown Album"}
                    </div>

                    <!-- Duration -->
                    <span class="col-span-3 font-mono text-right">
                        {formatDuration(track.duration_seconds)}
                    </span>

                    <!-- Options button -->
                    <div class="col-span-1 flex justify-end">
                        <button
                            class="text-gray-400 hover:text-white transition-colors invisible group-hover:visible"
                            aria-label="More options for {track.title}"
                            onclick={() => (showTrackOptions = index)}
                        >
                            <Ellipsis size={20} />
                        </button>
                    </div>

                    <!-- Context menu -->
                    {#if showTrackOptions === index}
                        <!-- Backdrop to close menu -->
                        <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
                        <div
                            class="fixed inset-0 z-10"
                            onclick={closeTrackOptions}
                        ></div>

                        <Menu class="absolute right-12 top-12 z-20">
                            <MenuItem
                                Icon={Pen}
                                label="Edit"
                                onclick={() => {
                                    closeTrackOptions();
                                    // TODO: wire up edit action
                                }}
                            />
                            <MenuItem
                                Icon={Clock}
                                label="Add to Queue"
                                onclick={() => {
                                    player.addToQueue(track);
                                    closeTrackOptions();
                                }}
                            />
                            <MenuItem Icon={Plus} label="Add to Playlist">
                                {#snippet children()}
                                    <Menu>
                                        {#each playlists as playlist}
                                            <MenuItem
                                                label={playlist.name}
                                                onclick={() => {
                                                    // TODO: wire up add-to-playlist action
                                                    closeTrackOptions();
                                                }}
                                            />
                                        {/each}
                                    </Menu>
                                {/snippet}
                            </MenuItem>
                            <MenuItem Icon={User} label="Go to Artist">
                                {#snippet children()}
                                    <Menu>
                                        {#each track.artists as artist}
                                            <MenuItem
                                                label={artist.name}
                                                onclick={() => {
                                                    goto(
                                                        `/library/artists/${artist.id}`,
                                                    );
                                                    closeTrackOptions();
                                                }}
                                            />
                                        {/each}
                                    </Menu>
                                {/snippet}
                            </MenuItem>
                            <MenuItem
                                Icon={Trash2}
                                label="Remove from Playlist"
                                onclick={() => {
                                    // TODO: wire up remove action
                                    closeTrackOptions();
                                }}
                            />
                            {@render otherMenuItems?.()}
                        </Menu>
                    {/if}
                </div>
            {/each}
        {/if}

        <!-- Fade-out footer gradient -->
        <div
            class="w-[110%] -ml-5 h-15 bg-linera-to-t from-black/50 to-transparent sticky -bottom-2 z-2 pointer-events-none"
        ></div>
    </div>
</div>
