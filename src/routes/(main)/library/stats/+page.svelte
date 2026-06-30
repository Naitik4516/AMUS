<script lang="ts">
    import { onMount } from "svelte";
    import { stats } from "$lib/stats.svelte";
    import { formatDuration, formatBytes } from "$lib/utils";
    import type { Timeframe } from "$lib/commands.svelte";
    import {
        ChartNoAxesColumn,
        Disc3,
        Users,
        Clock,
        Repeat,
        TrendingUp,
        Database,
        CircleOff,
    } from "@lucide/svelte";
    import TimeframeSelector from "$components/stats/TimeframeSelector.svelte";
    import StatCard from "$components/stats/StatCard.svelte";
    import TopTracksTable from "$components/stats/TopTracksTable.svelte";
    import TopArtistsGrid from "$components/stats/TopArtistsGrid.svelte";
    import TopAlbumsGrid from "$components/stats/TopAlbumsGrid.svelte";
    import ListeningTrendChart from "$components/stats/ListeningTrendChart.svelte";
    import LibraryGrowthChart from "$components/stats/LibraryGrowthChart.svelte";
    import FormatDistributionChart from "$components/stats/FormatDistributionChart.svelte";
    import HeatmapGrid from "$components/stats/HeatmapGrid.svelte";
    import StreakCalendar from "$components/stats/StreakCalendar.svelte";
    import FavoriteTrendChart from "$components/stats/FavoriteTrendChart.svelte";
    import PlaybackTimeline from "$components/stats/PlaybackTimeline.svelte";

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
        if (!stats.overview && !stats.loading) {
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
            <h1 class="text-4xl font-black text-white font-switzer">Statistics</h1>
        </div>
        <TimeframeSelector
            value={stats.timeframe}
            onchange={handleTimeframeChange}
            available={stats.availableTimeframes}
        />
    </header>

    {#if stats.loading && !stats.overview}
        <div class="flex items-center justify-center py-32">
            <div
                class="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-foreground"
            ></div>
        </div>
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
        <!-- Overview Cards -->
        <section
            class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-4 mb-8"
        >
            <StatCard label="Tracks" value={overview?.total_tracks}>
                {#snippet icon()}<Disc3 size={20} />{/snippet}
            </StatCard>
            <StatCard label="Artists" value={overview?.total_artists}>
                {#snippet icon()}<Users size={20} />{/snippet}
            </StatCard>
            <StatCard label="Albums" value={overview?.total_albums}>
                {#snippet icon()}<Database size={20} />{/snippet}
            </StatCard>
            <StatCard label="Total Plays" value={overview?.total_plays}>
                {#snippet icon()}<Repeat size={20} />{/snippet}
            </StatCard>
            <StatCard
                label="Listening Time"
                value={overview
                    ? formatDuration(overview.total_listening_time_sec)
                    : "—"}
            >
                {#snippet icon()}<Clock size={20} />{/snippet}
            </StatCard>
            <StatCard
                label="Avg Daily"
                value={overview
                    ? `${overview.avg_daily_listening_min.toFixed(0)} min`
                    : "—"}
            >
                {#snippet icon()}<TrendingUp size={20} />{/snippet}
            </StatCard>
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

        <div class="grid lg:grid-cols-3 gap-6">
            <!-- Left Column (2/3) -->
            <div class="lg:col-span-2 space-y-6">
                <ListeningTrendChart data={stats.listeningTrend} />

                {#if showStreak}
                    <StreakCalendar data={stats.streakData} />
                {/if}

                    <LibraryGrowthChart data={stats.libraryGrowth} />

                {#if showFavoriteTrends}
                    <FavoriteTrendChart data={stats.favoriteTrends} />
                {/if}

                {#if showTimeline}
                    <PlaybackTimeline events={stats.playbackHistory} />
                {/if}
            </div>

            <!-- Right Column (1/3) -->
            <div class="space-y-6">
                <TopTracksTable tracks={stats.topTracks} />
                <TopArtistsGrid artists={stats.topArtists} />
                <TopAlbumsGrid albums={stats.topAlbums} />
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
