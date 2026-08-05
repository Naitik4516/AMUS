<script lang="ts">
    import { player } from "$lib/player.svelte";
    import {
        Play,
        Pause,
        Heart,
        Music,
        Calendar,
        Clock,
        ChartNoAxesColumn,
        Folder,
        Info,
        Trash2,
        Plus,
        X,
        Search,
        MicVocal,
        FileHeadphone,
        SlidersHorizontal,
    } from "@lucide/svelte";
    import Artist from "$components/icons/Artist.svelte";
    import type { PageProps } from "./$types";
    import { formatDuration } from "$lib/utils";
    import { invoke } from "@tauri-apps/api/core";
    import { store } from "$lib/stores.svelte";
    import { goto, invalidate } from "$app/navigation";
    import { openConfirmDialog } from "$lib/context-menu.svelte";
    import { toast } from "svelte-sonner";
    import * as commands from "$lib/commands.svelte";
    import { selectAndUploadImage } from "$lib/edit-helpers";
    import { revealItemInDir } from "@tauri-apps/plugin-opener";
    import EditImage from "$components/ui/EditImage.svelte";
    import Button from "$components/ui/button/button.svelte";
    import Dialog from "$components/Dialog.svelte";
    import Fuse from "fuse.js";
    import { Input } from "$components/ui/input/index.js";
    import * as Popover from "$components/ui/popover/index.js";

    let { data }: PageProps = $props();
    let track = $derived(data.trackDetails);

    function initialEditState() {
        const t = data.trackDetails;
        return {
            title: t.title,
            year: t.year?.toString() ?? "",
            coverArt: t.cover_art ?? null,
            plainLyrics: t.lyrics?.plain_lyrics ?? "",
            syncedLyrics: t.lyrics?.synced_lyrics ?? "",
            source: t.lyrics?.source ?? "",
        };
    }
    const initialEdit = initialEditState();

    let titleEdit = $state(initialEdit.title);
    let yearEdit = $state(initialEdit.year);

    let coverArtFile = $state<string | null>(initialEdit.coverArt);

    let showLyrics = $state(false);
    let plainLyricsEdit = $state(initialEdit.plainLyrics);
    let syncedLyricsEdit = $state(initialEdit.syncedLyrics);
    let lyricsSource = $state(initialEdit.source);

    let editTrackId = $state<number | null>(null);

    $effect(() => {
        const t = data.trackDetails;
        if (t.id === editTrackId) return;
        editTrackId = t.id;
        titleEdit = t.title;
        yearEdit = t.year?.toString() ?? "";
        coverArtFile = t.cover_art ?? null;
        plainLyricsEdit = t.lyrics?.plain_lyrics ?? "";
        syncedLyricsEdit = t.lyrics?.synced_lyrics ?? "";
        lyricsSource = t.lyrics?.source ?? "";
    });

    let showArtistSearch = $state(false);
    let artistSearchQuery = $state("");
    let artistFuse: Fuse<(typeof store.artists)[number]>;

    let albumSearchQuery = $state("");
    let albumFuse: Fuse<(typeof store.albums)[number]>;

    let showGenreSearch = $state(false);
    let genreSearchQuery = $state("");
    let genreFuse: Fuse<(typeof store.genres)[number]>;

    let artistSearchResults = $state<typeof store.artists>([]);
    let albumSearchResults = $state<typeof store.albums>([]);
    let genreSearchResults = $state<typeof store.genres>([]);

    function initArtistFuse() {
        if (store.artists.length > 0) {
            artistFuse = new Fuse(store.artists, {
                keys: ["name"],
                threshold: 0.3,
            });
        }
    }
    function initAlbumFuse() {
        if (store.albums.length > 0) {
            albumFuse = new Fuse(store.albums, {
                keys: ["name"],
                threshold: 0.3,
            });
        }
    }
    function initGenreFuse() {
        if (store.genres.length > 0) {
            genreFuse = new Fuse(store.genres, {
                keys: ["name"],
                threshold: 0.3,
            });
        }
    }

    $effect(() => {
        if (artistSearchQuery.length >= 1) {
            if (!artistFuse) initArtistFuse();
            if (!artistFuse) {
                artistSearchResults = [];
                return;
            }
            const existing = new Set(track.artists.map((a) => a.id));
            artistSearchResults = artistFuse
                .search(artistSearchQuery)
                .filter((r) => !existing.has(r.item.id))
                .slice(0, 8)
                .map((r) => r.item);
        } else {
            artistSearchResults = [];
        }
    });
    $effect(() => {
        if (albumSearchQuery.length >= 1) {
            if (!albumFuse) initAlbumFuse();
            if (!albumFuse) {
                albumSearchResults = [];
                return;
            }
            albumSearchResults = albumFuse
                .search(albumSearchQuery)
                .slice(0, 8)
                .map((r) => r.item);
        } else {
            albumSearchResults = [];
        }
    });
    $effect(() => {
        if (genreSearchQuery.length >= 1) {
            if (!genreFuse) initGenreFuse();
            if (!genreFuse) {
                genreSearchResults = [];
                return;
            }
            const existing = new Set(track.genres.map((g) => g.id));
            genreSearchResults = genreFuse
                .search(genreSearchQuery)
                .filter((r) => !existing.has(r.item.id))
                .slice(0, 8)
                .map((r) => r.item);
        } else {
            genreSearchResults = [];
        }
    });

    async function toggleFavorite() {
        try {
            await invoke<boolean>("toggle_favorite", { id: track.id });
            invalidate("app:track-details");
        } catch (e) {
            console.error("Failed to toggle favorite", e);
        }
    }

    function handleDelete() {
        const id = track.id;
        const title = track.title;
        openConfirmDialog({
            title: "Delete track",
            message: `Are you sure you want to delete "${title}" from your library? This will also remove it from all playlists.`,
            confirmLabel: "Delete",
            onConfirm: async () => {
                try {
                    await store.deleteTrack(id);
                    toast.success("Track deleted from library");
                    goto("/library");
                } catch (e) {
                    console.error("Failed to delete track:", e);
                    toast.error("Failed to delete track");
                }
            },
        });
    }

    async function pickCover() {
        try {
            const filename = await selectAndUploadImage("cover");
            if (filename) {
                coverArtFile = filename;
                await commands.setTrackCoverArt(track.id, filename);
                invalidate("app:track-details");
                toast.success("Cover updated");
            }
        } catch (e) {
            console.error("Failed to update cover:", e);
            toast.error("Failed to update cover");
        }
    }

    async function removeCover() {
        try {
            coverArtFile = null;
            await commands.setTrackCoverArt(track.id, null);
            invalidate("app:track-details");
            toast.success("Cover removed");
        } catch (e) {
            console.error("Failed to remove cover:", e);
            toast.error("Failed to remove cover");
        }
    }

    async function saveTitle() {
        if (titleEdit.trim() && titleEdit !== track.title) {
            try {
                await store.updateTrackMetadata(
                    track.id,
                    titleEdit.trim(),
                    track.year,
                );
                invalidate("app:track-details");
            } catch (e) {
                console.error("Failed to save title:", e);
                toast.error("Failed to save title");
            }
        }
    }

    async function saveYear() {
        const y = yearEdit ? parseInt(yearEdit) : null;
        if (y !== track.year) {
            try {
                await store.updateTrackMetadata(track.id, track.title, y);
                invalidate("app:track-details");
            } catch (e) {
                console.error("Failed to save year:", e);
                toast.error("Failed to save year");
            }
        }
    }

    async function saveLyrics() {
        const source = track.lyrics?.source ?? "manual";
        await commands.updateTrackLyrics(
            track.id,
            plainLyricsEdit || null,
            syncedLyricsEdit || null,
            source,
        );
        toast.success("Lyrics saved");
        invalidate("app:track-details");
    }

    async function addArtist(artistId: number) {
        const ids = [...track.artists.map((a) => a.id), artistId];
        await commands.setTrackArtists(track.id, ids);
        artistSearchQuery = "";
        artistSearchResults = [];
        showArtistSearch = false;
        invalidate("app:track-details");
    }

    async function removeArtist(artistId: number) {
        const ids = track.artists
            .filter((a) => a.id !== artistId)
            .map((a) => a.id);
        await commands.setTrackArtists(track.id, ids);
        invalidate("app:track-details");
    }

    async function changeAlbum(albumId: number) {
        await commands.setTrackAlbum(track.id, albumId);
        albumSearchQuery = "";
        albumSearchResults = [];
        invalidate("app:track-details");
    }

    async function addGenreByName(name: string) {
        if (!name.trim()) return;
        await commands.setTrackGenre(track.id, name.trim());
        genreSearchQuery = "";
        genreSearchResults = [];
        showGenreSearch = false;
        invalidate("app:track-details");
    }

    async function addGenre(genreId: number) {
        const genre = store.genresById.get(genreId);
        if (genre) {
            await commands.setTrackGenre(track.id, genre.name);
            genreSearchQuery = "";
            genreSearchResults = [];
            showGenreSearch = false;
            invalidate("app:track-details");
        }
    }

    async function handleShowInFileManager() {
        try {
            await revealItemInDir(track.path);
        } catch (e) {
            console.error("Failed to reveal in file manager:", e);
        }
    }

    let audioFormatLabel = $derived(track.audio_format?.toUpperCase() ?? "—");
</script>

<div class="p-8 pb-32 max-w-5xl mx-auto">
    <!-- top section -->
    <div
        class="flex flex-col md:flex-row gap-12 items-center md:items-end mb-16"
    >
        <EditImage
            onclick={pickCover}
            removeCover={coverArtFile ? removeCover : undefined}
            class="w-82 h-82 rounded-3xl shadow-2xl overflow-hidden shrink-0"
        >
            {#if coverArtFile}
                <img
                    src={store.getImageSrc(coverArtFile)}
                    alt={track.title}
                    class="w-full h-full object-cover"
                />
            {:else}
                <div
                    class="w-full h-full bg-muted flex items-center justify-center"
                >
                    <Music size={80} class="text-muted-foreground" />
                </div>
            {/if}
        </EditImage>

        <div
            class="flex flex-col flex-1 text-center md:text-left md:pb-2 min-w-0 font-satoshi font-medium"
        >
            <p
                class="text-sm font-bold uppercase tracking-wider text-muted-foreground"
            >
                Song track
            </p>

            <input
                type="text"
                bind:value={titleEdit}
                onblur={saveTitle}
                onkeydown={(e) => e.key === "Enter" && e.currentTarget.blur()}
                class="text-4xl md:text-5xl font-black text-white font-switzer bg-transparent border-b border-transparent hover:border-gray-600 focus:border-accent focus:outline-none pb-1"
            />

            <div class="flex items-center gap-5">
                <span class="inline-flex items-center gap-1">
                    •
                    <Popover.Root>
                        <Popover.Trigger
                            class="text-gray-200 hover:text-white font-semibold hover:underline decoration-dotted underline-offset-4"
                            title="Change Album"
                        >
                            {track.album?.name ?? "Unknown Album"}
                        </Popover.Trigger>
                        <Popover.Content class="w-72">
                            <div
                                class="flex items-center gap-2 bg-black/30 rounded-full px-3 py-2"
                            >
                                <Search
                                    size={14}
                                    class="text-gray-400 shrink-0"
                                />
                                <Input
                                    type="text"
                                    bind:value={albumSearchQuery}
                                    placeholder="Search albums..."
                                    class="bg-transparent border-none outline-none text-sm text-white w-full h-8"
                                />
                            </div>
                            <div
                                class="max-h-48 overflow-y-auto flex flex-col gap-0.5 pr-1"
                            >
                                {#each albumSearchResults as album}
                                    <button
                                        onclick={() => changeAlbum(album.id)}
                                        class="flex items-center gap-3 p-2 rounded-xl hover:bg-white/5 hover:shadow-md text-left w-full"
                                    >
                                        {#if album.cover_art}
                                            <img
                                                src={store.getImageSrc(
                                                    album.cover_art,
                                                )}
                                                alt=""
                                                class="w-9 h-9 rounded-lg object-cover"
                                            />
                                        {:else}
                                            <div
                                                class="w-9 h-9 rounded-lg bg-muted flex items-center justify-center"
                                            >
                                                <Music
                                                    size={16}
                                                    class="text-muted-foreground"
                                                />
                                            </div>
                                        {/if}
                                        <span class="text-white truncate"
                                            >{album.name}</span
                                        >
                                    </button>
                                {/each}
                                {#if albumSearchResults.length === 0 && albumSearchQuery.length >= 1}
                                    <p
                                        class="text-xs text-gray-500 text-center py-2"
                                    >
                                        No albums found
                                    </p>
                                {/if}
                            </div>
                        </Popover.Content>
                    </Popover.Root>
                </span>
                <span class="inline-flex items-center gap-1">
                    •
                    <input
                        type="text"
                        bind:value={yearEdit}
                        onblur={saveYear}
                        onkeydown={(e) =>
                            e.key === "Enter" && e.currentTarget.blur()}
                        class="w-20 bg-transparent border-b border-transparent hover:border-gray-600 focus:border-accent focus:outline-none text-gray-200 font-semibold proportional-nums"
                        placeholder="Year"
                    />
                </span>
            </div>

            <!-- Artists -->
            <div class="flex flex-wrap items-center gap-0.5 mt-2">
                {#each track.artists as artist (artist.id)}
                    <div
                        class="group/artist flex items-center gap-1 hover:bg-gray-200/5 hover:ring-border transition-colors rounded-xl p-1"
                    >
                        {#if artist.profile_image}
                            <img
                                src={store.getImageSrc(
                                    artist.profile_image,
                                    "artist",
                                )}
                                alt={artist.name}
                                class="w-8 h-8 rounded-full object-cover mr-0.5"
                            />
                        {:else}
                            <Artist size={18} class="text-gray-400" />
                        {/if}
                        <a
                            href={`/library/artists/${artist.id}`}
                            class="hover:text-white transition-colors font-semibold"
                        >
                            {artist.name}
                        </a>
                        <button
                            onclick={() => removeArtist(artist.id)}
                            class="opacity-0 group-hover/artist:opacity-100 transition-opacity hover:text-red-400"
                        >
                            <X size={14} />
                        </button>
                    </div>
                {/each}

                <!-- Add artist button -->
                <Popover.Root>
                    <Popover.Trigger
                        class="size-8 rounded-full border border-dashed border-gray-600 flex items-center justify-center hover:border-accent hover:text-accent transition-colors"
                    >
                        <Plus size={14} />
                    </Popover.Trigger>
                    <Popover.Content class="w-72">
                        <div
                            class="flex items-center gap-2 bg-black/30 rounded-full px-3 py-2 mb-2"
                        >
                            <Search size={14} class="text-gray-400 shrink-0" />
                            <input
                                type="text"
                                bind:value={artistSearchQuery}
                                placeholder="Search artists..."
                                class="bg-transparent border-none outline-none text-sm text-white w-full"
                            />
                        </div>
                        <div
                            class="max-h-48 overflow-y-auto flex flex-col gap-1"
                        >
                            {#each artistSearchResults as artist}
                                <button
                                    onclick={() => addArtist(artist.id)}
                                    class="flex items-center gap-2 p-2 rounded-xl hover:bg-white/5 text-left w-full"
                                >
                                    {#if artist.profile_image}
                                        <img
                                            src={store.getImageSrc(
                                                artist.profile_image,
                                                "artist",
                                            )}
                                            alt=""
                                            class="w-7 h-7 rounded-full object-cover"
                                        />
                                    {:else}
                                        <Artist
                                            size={18}
                                            class="text-gray-400"
                                        />
                                    {/if}
                                    <span class="text-sm text-white"
                                        >{artist.name}</span
                                    >
                                </button>
                            {/each}
                            {#if artistSearchResults.length === 0 && artistSearchQuery.length >= 1}
                                <p
                                    class="text-xs text-gray-500 text-center py-2"
                                >
                                    No artists found
                                </p>
                            {/if}
                        </div>
                    </Popover.Content>
                </Popover.Root>
            </div>

            <!-- Actions -->
            <div
                class="flex items-center gap-4 mt-4 justify-center md:justify-start"
            >
                <Button
                    onclick={() =>
                        player.currentTrack?.id === track.id
                            ? player.playPause()
                            : player.play([track])}
                    size="icon-2xl"
                >
                    {#if player.currentTrack?.id === track.id && player.isPlaying}
                        <Pause size={26} fill="currentColor" />
                    {:else}
                        <Play size={26} fill="currentColor" />
                    {/if}
                </Button>
                <Button
                    onclick={toggleFavorite}
                    class=""
                    size="icon-2xl"
                    variant="outline"
                >
                    <Heart
                        size={24}
                        class={track.is_favorite
                            ? "text-rose-600 fill-rose-600"
                            : "text-gray-300"}
                    />
                </Button>
                <Button
                    onclick={handleDelete}
                    size="icon-2xl"
                    variant="outline"
                    class="text-red-300 hover:text-red-500 hover:border-red-500/30"
                >
                    <Trash2 size={24} />
                </Button>
            </div>
        </div>
    </div>

    <!-- Info Cards Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
        <!-- General Information -->
        <div
            class="bg-card/30 backdrop-blur-md rounded-3xl p-6 border border-border shadow-xl"
        >
            <h3
                class="text-lg font-bold text-white mb-6 flex items-center gap-2"
            >
                <Info size={18} /> General Information
            </h3>
            <div class="flex flex-col gap-4">
                <div class="flex justify-between items-center">
                    <span class="text-gray-400 flex items-center gap-2"
                        ><Clock size={16} /> Duration</span
                    >
                    <span class="text-white font-medium"
                        >{formatDuration(track.duration_seconds)}</span
                    >
                </div>
                <div class="flex justify-between items-center">
                    <span class="text-gray-400 flex items-center gap-2"
                        ><Calendar size={16} /> Added on</span
                    >
                    <span class="text-white font-medium"
                        >{new Date(
                            track.mtime * 1000,
                        ).toLocaleDateString()}</span
                    >
                </div>

                <!-- Genres -->
                <div class="flex justify-between items-start">
                    <span class="text-gray-400 flex items-center gap-2 pt-1"
                        ><Music size={16} /> Genres</span
                    >
                    <div class="flex flex-wrap gap-1.5 justify-end">
                        {#each track.genres as genre (genre.id)}
                            <span
                                class="text-xs bg-white/10 px-2.5 py-0.5 rounded-full text-gray-200"
                            >
                                {genre.name}
                            </span>
                        {/each}
                        <Popover.Root>
                            <Popover.Trigger
                                class="size-5 rounded-full border border-dashed border-gray-600 flex items-center justify-center hover:border-accent"
                            >
                                <Plus size={10} />
                            </Popover.Trigger>
                            <Popover.Content class="w-72">
                                <div
                                    class="flex items-center gap-2 bg-black/30 rounded-full px-3 py-2 mb-2"
                                >
                                    <Search
                                        size={14}
                                        class="text-gray-400 shrink-0"
                                    />
                                    <input
                                        type="text"
                                        bind:value={genreSearchQuery}
                                        placeholder="Search genres..."
                                        class="bg-transparent border-none outline-none text-sm text-white w-full"
                                    />
                                </div>
                                <div
                                    class="max-h-40 overflow-y-auto flex flex-col gap-1"
                                >
                                    {#each genreSearchResults as genre}
                                        <button
                                            onclick={() => addGenre(genre.id)}
                                            class="text-sm text-left p-2 rounded-xl hover:bg-white/5 text-white w-full"
                                        >
                                            {genre.name}
                                        </button>
                                    {/each}
                                    {#if genreSearchQuery.length >= 1 && !genreSearchResults.some((g) => g.name.toLowerCase() === genreSearchQuery.toLowerCase())}
                                        <button
                                            onclick={() =>
                                                addGenreByName(
                                                    genreSearchQuery,
                                                )}
                                            class="text-sm text-left p-2 rounded-xl hover:bg-accent/20 text-accent w-full border border-dashed border-accent/40 mt-1"
                                        >
                                            + Create "{genreSearchQuery}"
                                        </button>
                                    {/if}
                                </div>
                            </Popover.Content>
                        </Popover.Root>
                    </div>
                </div>
            </div>
        </div>

        <!-- Playback Stats -->
        <div
            class="bg-card/20 backdrop-blur-md rounded-3xl p-6 border border-border shadow-xl"
        >
            <h3
                class="text-lg font-bold text-white mb-6 flex items-center gap-2"
            >
                <ChartNoAxesColumn size={18} /> Playback Stats
            </h3>
            <div class="flex flex-col gap-4">
                <div class="flex justify-between items-center">
                    <span class="text-gray-400">Total Plays</span>
                    <span class="text-white font-medium"
                        >{track.play_count}</span
                    >
                </div>
                <div class="flex justify-between items-center">
                    <span class="text-gray-400">Total Skips</span>
                    <span class="text-white font-medium"
                        >{track.skipped_count}</span
                    >
                </div>
                <div class="flex justify-between items-center">
                    <span class="text-gray-400">Last Played</span>
                    <span class="text-white font-medium">
                        {track.last_played_at
                            ? new Date(track.last_played_at).toLocaleString()
                            : "Never"}
                    </span>
                </div>
            </div>
        </div>

        <!-- Technical Info -->
        <div
            class="bg-card/20 backdrop-blur-md rounded-3xl p-6 border border-border shadow-xl"
        >
            <h3
                class="text-lg font-bold text-white mb-6 flex items-center gap-2"
            >
                <FileHeadphone size={18} /> Technical Info
            </h3>
            <div class="flex flex-col gap-3">
                <div class="flex justify-between items-center">
                    <span class="text-gray-400">Audio Format</span>
                    <span class="text-white font-medium"
                        >{audioFormatLabel}</span
                    >
                </div>
                {#if track.codec}
                    <div class="flex justify-between items-center">
                        <span class="text-gray-400">Codec</span>
                        <span class="text-white font-medium">{track.codec}</span
                        >
                    </div>
                {/if}
                {#if track.bitrate}
                    <div class="flex justify-between items-center">
                        <span class="text-gray-400">Bitrate</span>
                        <span class="text-white font-medium"
                            >{track.bitrate} kbps</span
                        >
                    </div>
                {/if}
                {#if track.sample_rate > 0}
                    <div class="flex justify-between items-center">
                        <span class="text-gray-400">Sample Rate</span>
                        <span class="text-white font-medium"
                            >{track.sample_rate} Hz</span
                        >
                    </div>
                {/if}
                {#if track.bit_depth}
                    <div class="flex justify-between items-center">
                        <span class="text-gray-400">Bit Depth</span>
                        <span class="text-white font-medium"
                            >{track.bit_depth}-bit</span
                        >
                    </div>
                {/if}
                {#if track.channels > 0}
                    <div class="flex justify-between items-center">
                        <span class="text-gray-400">Channels</span>
                        <span class="text-white font-medium"
                            >{track.channels === 1
                                ? "Mono"
                                : track.channels === 2
                                  ? "Stereo"
                                  : `${track.channels} channels`}</span
                        >
                    </div>
                {/if}
            </div>
        </div>

        <!-- Analysis -->
        <div
            class="bg-card/20 backdrop-blur-md rounded-3xl p-6 border border-border shadow-xl"
        >
            <h3
                class="text-lg font-bold text-white mb-6 flex items-center gap-2"
            >
                <SlidersHorizontal size={18} /> Analysis
            </h3>
            <div class="flex flex-col gap-3">
                {#if track.bpm}
                    <div class="flex justify-between items-center">
                        <span class="text-gray-400">BPM</span>
                        <span class="text-white font-medium"
                            >{Math.round(track.bpm)}</span
                        >
                    </div>
                {/if}
                {#if track.replaygain_track_gain}
                    <div class="flex justify-between items-center">
                        <span class="text-gray-400">RG Track Gain</span>
                        <span class="text-white font-medium"
                            >{track.replaygain_track_gain.toFixed(2)} dB</span
                        >
                    </div>
                {/if}
                {#if track.replaygain_track_peak}
                    <div class="flex justify-between items-center">
                        <span class="text-gray-400">RG Track Peak</span>
                        <span class="text-white font-medium"
                            >{track.replaygain_track_peak.toFixed(6)}</span
                        >
                    </div>
                {/if}
                {#if track.replaygain_album_gain}
                    <div class="flex justify-between items-center">
                        <span class="text-gray-400">RG Album Gain</span>
                        <span class="text-white font-medium"
                            >{track.replaygain_album_gain.toFixed(2)} dB</span
                        >
                    </div>
                {/if}
                {#if track.replaygain_album_peak}
                    <div class="flex justify-between items-center">
                        <span class="text-gray-400">RG Album Peak</span>
                        <span class="text-white font-medium"
                            >{track.replaygain_album_peak.toFixed(6)}</span
                        >
                    </div>
                {/if}
                {#if track.encoder}
                    <div class="flex justify-between items-center">
                        <span class="text-gray-400">Encoder</span>
                        <span class="text-white font-medium"
                            >{track.encoder}</span
                        >
                    </div>
                {/if}
            </div>
        </div>

        <!-- File Information + Lyrics -->
        <div
            class="md:col-span-2 bg-card/20 backdrop-blur-md rounded-3xl p-6 border border-border shadow-xl"
        >
            <h3
                class="text-lg font-bold text-white mb-4 flex items-center gap-2"
            >
                <Folder size={18} /> File Information
            </h3>
            <p
                class="text-gray-400 text-sm break-all bg-black/30 p-4 rounded-lg font-mono mb-4 select-text"
            >
                {track.path}
            </p>
            <div class="flex gap-3">
                <Button
                    variant="secondary"
                    onclick={handleShowInFileManager}
                    size="sm"
                >
                    <Folder size={14} /> Show in File Manager
                </Button>
                <Button
                    variant="secondary"
                    onclick={() => {
                        showLyrics = true;
                        plainLyricsEdit = track.lyrics?.plain_lyrics ?? "";
                        syncedLyricsEdit = track.lyrics?.synced_lyrics ?? "";
                        lyricsSource = track.lyrics?.source ?? "";
                    }}
                    size="sm"
                >
                    <MicVocal size={14} /> Show Lyrics
                </Button>
            </div>
        </div>
    </div>
</div>

<!-- Lyrics Dialog -->
<Dialog bind:open={showLyrics} title="Edit Lyrics">
    <div class="flex flex-col gap-4 mb-4">
        <div class="text-xs text-gray-500">
            Source: {lyricsSource || "None"}
        </div>

        <label class="text-sm text-gray-400 font-medium" for="plain-lyrics">
            Plain Lyrics
        </label>
        <textarea
            bind:value={plainLyricsEdit}
            id="plain-lyrics"
            class="w-full h-40 bg-black/30 border border-border rounded-xl p-3 text-sm text-white font-mono resize-y focus:outline-none focus:border-accent"
            placeholder="Enter plain lyrics..."></textarea>

        <label class="text-sm text-gray-400 font-medium" for="synced-lyrics">
            Synced Lyrics (LRC format)
        </label>
        <textarea
            bind:value={syncedLyricsEdit}
            id="synced-lyrics"
            class="w-full h-32 bg-black/30 border border-border rounded-xl p-3 text-sm text-white font-mono resize-y focus:outline-none focus:border-accent"
            placeholder="[00:12.34] Line one
[00:25.67] Line two"></textarea>
    </div>

    {#snippet Footer()}
        <Button
            variant="secondary"
            onclick={() => {
                showLyrics = false;
            }}>Cancel</Button
        >
        <Button onclick={saveLyrics}>Save Lyrics</Button>
    {/snippet}
</Dialog>
