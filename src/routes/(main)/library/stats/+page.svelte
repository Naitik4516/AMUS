<script lang="ts">
    import FavoriteTrendChart from "$components/stats/FavoriteTrendChart.svelte";
    import FormatDistributionChart from "$components/stats/FormatDistributionChart.svelte";
    import HeatmapGrid from "$components/stats/HeatmapGrid.svelte";
    import LibraryGrowthChart from "$components/stats/LibraryGrowthChart.svelte";
    import ListeningTrendChart from "$components/stats/ListeningTrendChart.svelte";
    import PlaybackTimeline from "$components/stats/PlaybackTimeline.svelte";
    import StatCard from "$components/stats/StatCard.svelte";
    import StatsSkeleton from "$components/stats/StatsSkeleton.svelte";
    import StreakCalendar from "$components/stats/StreakCalendar.svelte";
    import TimeframeSelector from "$components/stats/TimeframeSelector.svelte";
    import TopList from "$components/stats/TopList.svelte";
    import ArtistAvatar from "$components/ui/ArtistAvatar.svelte";
    import TrackCoverArt from "$components/ui/TrackCoverArt.svelte";
    import type { Timeframe } from "$lib/commands.svelte";
    import { stats } from "$lib/stats.svelte";
    import { store } from "$lib/stores.svelte";
    import {
        formatBytes,
        formatDuration
    } from "$lib/utils";
    import {
        ChartNoAxesColumn,
        CircleOff,
        Clock,
        Database,
        Disc3,
        Repeat,
        TrendingUp,
        Users,
    } from "@lucide/svelte";
    import { onMount } from "svelte";

    const showStreak = $derived(stats.timeframe !== "today");
    const showFavoriteTrends = $derived(
        !["today", "this_week", "this_month"].includes(stats.timeframe),
    );
    const showTimeline = $derived(
        !["last_5_years", "all_time"].includes(stats.timeframe),
    );
    const overview = $derived(stats.overview);

    function handleTimeframeChange(tf: Timeframe) {
        stats.setTimeframe(tf);
    }

    onMount(() => {
        if (!stats.loading) {
            stats.loadAll();
        }
        if (!stats.dataAge) {
            stats.loadDataAge();
        }
    });
</script>

<div class="p-6 max-w-7xl mx-auto h-full overflow-y-scroll">
    <header class="flex items-center justify-between mb-8">
        <div class="flex items-center gap-3">
            <ChartNoAxesColumn size={28} class="text-foreground" />
            <h1 class="text-4xl font-black text-white font-switzer">
                Statistics
            </h1>
        </div>
        <TimeframeSelector
            value={stats.timeframe}
            onchange={handleTimeframeChange}
            available={stats.availableTimeframes}
        />
    </header>

    {#if stats.loading && !stats.overview}
        <!-- <div class="flex items-center justify-center py-32">
            <div
                class="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-foreground"
            ></div>
        </div> -->
        <StatsSkeleton />
    {:else if stats.error}
        <div
            class="flex flex-col items-center justify-center py-20 text-gray-500"
        >
            <CircleOff size={48} class="mb-4 opacity-30" />
            <p class="text-lg font-medium">Failed to load statistics</p>
            <button
                onclick={() => stats.loadAll()}
                class="mt-4 px-6 py-2 bg-accent text-black font-bold rounded-full hover:scale-105 transition-transform"
            >
                Retry
            </button>
        </div>
    {:else if overview && overview.total_tracks === 0}
        <div
            class="flex flex-col items-center justify-center py-20 text-gray-500"
        >
            <Disc3 size={64} class="mb-4 opacity-20" />
            <p class="text-xl font-medium">No tracks in your library</p>
            <p class="text-sm mt-2">
                Add music sources in Settings to start listening.
            </p>
        </div>
    {:else}
        <div
            class:opacity-60={stats.loading}
            class="transition-opacity duration-200"
        >
            <!-- Overview Cards -->
            <section
                class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-4 mb-8"
            >
                <StatCard
                    label="Tracks"
                    value={overview?.total_tracks}
                    Icon={Disc3}
                />
                <StatCard
                    label="Artists"
                    value={overview?.total_artists}
                    Icon={Users}
                />
                <StatCard
                    label="Albums"
                    value={overview?.total_albums}
                    Icon={Database}
                />
                <StatCard
                    label="Total Plays"
                    value={overview?.total_plays}
                    Icon={Repeat}
                />
                <StatCard
                    label="Listening Time"
                    value={overview
                        ? formatDuration(overview.total_listening_time_sec)
                        : "—"}
                    Icon={Clock}
                />
                <StatCard
                    label="Avg Daily"
                    value={overview
                        ? `${overview.avg_daily_listening_min.toFixed(0)} min`
                        : "—"}
                    Icon={TrendingUp}
                />
            </section>

            <!-- Second row of smaller cards -->
            <section class="grid grid-cols-2 sm:grid-cols-4 gap-4 mb-8">
                <StatCard
                    label="Library Size"
                    value={overview
                        ? formatBytes(overview.total_file_size_bytes)
                        : "—"}
                    subtitle={overview
                        ? `${overview.avg_file_size_mb.toFixed(1)} MB avg`
                        : ""}
                />
                <StatCard
                    label="Largest File"
                    value={overview
                        ? `${overview.largest_file_mb.toFixed(1)} MB`
                        : "—"}
                />
                <StatCard
                    label="Library Played"
                    value={overview
                        ? `${overview.pct_library_played.toFixed(1)}%`
                        : "—"}
                />
                <StatCard label="Unplayed" value={overview?.unplayed_tracks} />
            </section>

            <!-- Quality + listening behaviour -->
            <section
                class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-4 mb-8"
            >
                <StatCard
                    label="Avg Bitrate"
                    value={overview?.avg_bitrate_kbps != null
                        ? `${overview.avg_bitrate_kbps.toFixed(0)} kbps`
                        : "—"}
                />
                <StatCard
                    label="Avg Sample Rate"
                    value={overview?.avg_sample_rate != null
                        ? `${(overview.avg_sample_rate / 1000).toFixed(1)} kHz`
                        : "—"}
                />
                <StatCard
                    label="Avg Bit Depth"
                    value={overview?.avg_bit_depth != null
                        ? `${overview.avg_bit_depth.toFixed(0)} bit`
                        : "—"}
                />
                <StatCard
                    label="Avg Completion"
                    value={overview?.avg_completion_pct != null
                        ? `${overview.avg_completion_pct.toFixed(0)}%`
                        : "—"}
                    subtitle="of track listened"
                />
                <StatCard
                    label="Skip Rate"
                    value={overview?.skip_rate != null
                        ? `${(overview.skip_rate * 100).toFixed(0)}%`
                        : "—"}
                    subtitle="of plays skipped"
                />
            </section>

            <div class="grid lg:grid-cols-3 gap-6">
                <!-- Left Column (2/3) -->
                <div class="lg:col-span-2 space-y-6">
                    <ListeningTrendChart data={stats.listeningTrend} />

                    {#if showStreak}
                        <StreakCalendar
                            data={stats.streakData}
                            timeframe={stats.timeframe}
                        />
                    {/if}

                    <LibraryGrowthChart data={stats.libraryGrowth} />

                    {#if showFavoriteTrends}
                        <FavoriteTrendChart data={stats.favoriteTrends} />
                    {/if}

                    {#if showTimeline}
                        <PlaybackTimeline
                            events={stats.playbackHistory}
                            canLoadMore={stats.playbackHistory.length >=
                                stats.historyLimit}
                            onLoadMore={() => stats.loadMoreHistory()}
                        />
                    {/if}
                    <FormatDistributionChart data={stats.formatDist} />
                    <HeatmapGrid
                        title="By Hour"
                        data={stats.heatmapHourly}
                        type="hourly"
                    />
                    <HeatmapGrid
                        title="By Weekday"
                        data={stats.heatmapWeekday}
                        type="weekday"
                    />
                </div>

                <!-- Right Column (1/3) -->
                <div class="space-y-6">
                    <TopList
                        title="Top Tracks"
                        items={stats.topTracks}
                        sortBy={stats.tracksSortBy}
                        onSortChange={(sort) => stats.setSortBy("tracks", sort)}
                    >
                        {#snippet leading(item)}
                            <div
                                class="size-10 rounded-lg bg-zinc-800 shrink-0 overflow-hidden"
                            >
                                <TrackCoverArt
                                    cover_art={item.track.cover_art}
                                />
                            </div>
                        {/snippet}
                        {#snippet subtitle(item)}
                            <p class="text-sm text-gray-300 truncate">
                                {item.track.artists
                                    .map((a) => a.name)
                                    .join(", ")}
                            </p>
                        {/snippet}
                    </TopList>

                    <TopList
                        title="Top Artists"
                        items={stats.topArtists}
                        sortBy={stats.artistsSortBy}
                        onSortChange={(sort) =>
                            stats.setSortBy("artists", sort)}
                    >
                        {#snippet leading(item)}
                            <ArtistAvatar
                                size={40}
                                profileImage={item.artist.profile_image}
                                name={item.artist.name}
                            />
                        {/snippet}
                        {#snippet subtitle(item)}
                            <p class="text-xs text-gray-400">
                                {item.tracks_played} tracks
                            </p>
                        {/snippet}
                    </TopList>

                    <TopList
                        title="Top Albums"
                        items={stats.topAlbums}
                        sortBy={stats.albumsSortBy}
                        onSortChange={(sort) => stats.setSortBy("albums", sort)}
                    >
                        {#snippet leading(item)}
                            <div
                                class="size-10 rounded-lg bg-zinc-800 shrink-0 overflow-hidden"
                            >
                                {#if store.getImageSrc(item.album.cover_art)}
                                    <img
                                        src={store.getImageSrc(
                                            item.album.cover_art,
                                        )}
                                        alt=""
                                        class="size-full object-cover"
                                    />
                                {/if}
                            </div>
                        {/snippet}
                    </TopList>

                    <TopList
                        title="Top Genres"
                        items={stats.topGenres}
                        sortBy={stats.genresSortBy}
                        onSortChange={(sort) => stats.setSortBy("genres", sort)}
                    >
                        {#snippet leading(item)}
                            <div
                                class="size-10 rounded-full bg-zinc-800 shrink-0 overflow-hidden"
                            >
                                {#if store.getImageSrc(item.genre.thumbnail)}
                                    <img
                                        src={store.getImageSrc(
                                            item.genre.thumbnail,
                                        )}
                                        alt=""
                                        class="size-full object-cover"
                                    />
                                {:else}
                                    <div
                                        class="size-full flex items-center justify-center text-xs text-gray-500"
                                    >
                                        {item.genre.name.charAt(0)}
                                    </div>
                                {/if}
                            </div>
                        {/snippet}
                        {#snippet subtitle(item)}
                            <p class="text-xs text-gray-400">
                                {item.tracks_played} tracks
                            </p>
                        {/snippet}
                    </TopList>
                </div>
            </div>
        </div>

        {#if stats.loading}
            <div class="flex justify-center py-4">
                <div
                    class="animate-spin rounded-full h-6 w-6 border-t-2 border-foreground"
                ></div>
            </div>
        {/if}
    {/if}
</div>
