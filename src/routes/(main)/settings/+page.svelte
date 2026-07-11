<script lang="ts">
    import ShortcutSettingsModal from "$components/ShortcutSettingsModal.svelte";
    import Button from "$components/ui/button/button.svelte";
    import Label from "$components/ui/label/label.svelte";
    import ToggleCard from "$components/ui/ToggleCard.svelte";
    import {
        getSourceDirs,
        importAudioLibrary,
        refreshWatcher,
        removeSource,
        scanLibrary,
    } from "$lib/commands.svelte";
    import { initSettings, setSetting, settings } from "$lib/settings.svelte";
    import { updater } from "$lib/update.svelte";
    import {
        Download,
        FolderOpen,
        Image,
        Keyboard,
        Minimize2,
        Plus,
        Power,
        RefreshCw,
        RotateCcw,
        Settings2,
        Trash2,
        User,
        Zap,
    } from "@lucide/svelte";
    import { onMount } from "svelte";

    let sources = $state<string[]>([]);
    let loading = $state(true);
    let syncing = $state(false);
    let showShortcutModal = $state(false);

    async function loadSources() {
        loading = true;
        try {
            sources = await getSourceDirs();
        } catch (e) {
            console.error("Failed to load sources", e);
        }
        loading = false;
    }

    async function handleRemoveSource(path: string) {
        try {
            await removeSource(path);
            sources = sources.filter((s) => s !== path);
        } catch (e) {
            console.error("Failed to remove source", e);
        }
    }

    async function handleAddSource() {
        await importAudioLibrary();
        await loadSources();
    }

    async function handleManualSync() {
        syncing = true;
        try {
            await scanLibrary();
        } catch (e) {
            console.error("Manual sync failed", e);
        } finally {
            syncing = false;
        }
    }

    onMount(() => {
        loadSources();
        initSettings();
        updater.loadCurrentVersion();
    });
</script>

<ShortcutSettingsModal bind:open={showShortcutModal} />

<div class="p-8 max-w-4xl mx-auto">
    <div class="mb-12">
        <div class="flex items-center justify-between mb-2">
            <h1 class="text-3xl font-black text-white">Library Sources</h1>
            <Button onclick={handleAddSource} size="lg">
                <Plus size={20} /> Add Folder
            </Button>
        </div>
        <div class="px-2">
            <p class="text-gray-400 mb-6">
                Manage the folders that Amus scans for audio files.
            </p>

            {#if loading}
                <div class="flex items-center justify-center py-20">
                    <div
                        class="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-foreground"
                    ></div>
                </div>
            {:else if sources.length === 0}
                <div
                    class="flex flex-col items-center justify-center py-20 text-gray-500"
                >
                    <FolderOpen size={64} class="mb-4 opacity-20" />
                    <p class="text-xl font-medium">No sources configured</p>
                    <p class="text-sm mb-6">
                        Add a folder to start scanning your music.
                    </p>
                    <button
                        onclick={handleAddSource}
                        class="flex items-center gap-2 px-6 py-3 bg-accent text-black font-bold rounded-full hover:scale-105 transition-transform"
                    >
                        <FolderOpen size={20} /> Select Folder
                    </button>
                </div>
            {:else}
                <div class="space-y-3">
                    {#each sources as source}
                        <div
                            class="flex items-center justify-between bg-card/50 backdrop-blur-lg border border-border rounded-3xl shadow-lg px-5 py-4 group hover:border-neutral-700 transition-colors"
                        >
                            <div class="flex items-center gap-4 min-w-0">
                                <FolderOpen
                                    size={20}
                                    class="text-foreground shrink-0"
                                />
                                <span
                                    class="text-white truncate"
                                    title={source}
                                >
                                    {source}
                                </span>
                            </div>
                            <button
                                onclick={() => handleRemoveSource(source)}
                                class="shrink-0 p-2 text-gray-500 hover:text-red-400 hover:bg-red-400/10 rounded-lg transition-colors"
                                title="Remove source"
                            >
                                <Trash2 size={18} />
                            </button>
                        </div>
                    {/each}
                </div>
            {/if}
        </div>
    </div>

    <div class="mb-12">
        <div class="flex items-center gap-2 mb-8">
            <Settings2 size={24} class="text-foreground" />
            <h2 class="text-2xl font-black text-white">Preferences</h2>
        </div>

        <div class="px-2 flex flex-col gap-4">
            <h3 class="text-lg font-bold text-gray-300">Sync</h3>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <ToggleCard
                    title="Real-time Sync"
                    description="Automatically update your library when files are added, modified, or removed from your source folders."
                    bind:checked={settings.realtimeSync}
                    onchange={(v) =>
                        setSetting("realtimeSync", v).then(refreshWatcher)}
                    icon={Zap}
                    iconActiveClass="text-yellow-400"
                />
                <ToggleCard
                    title="Sync on Startup"
                    description="Perform a full library scan every time you open Amus to ensure everything is up to date."
                    bind:checked={settings.syncOnStartup}
                    onchange={(v) => setSetting("syncOnStartup", v)}
                    icon={Power}
                    iconActiveClass="text-green-400"
                />
            </div>
            <div
                class="flex items-center justify-between px-8 py-4 bg-card/50 backdrop-blur-lg rounded-3xl shadow-lg border border-border"
            >
                <Label class="text-xl font-bold text-white">Manual Sync</Label>
                <Button
                    onclick={handleManualSync}
                    disabled={syncing}
                    variant="outline"
                    size="lg"
                >
                    <RefreshCw
                        size={20}
                        class={syncing ? "animate-spin" : ""}
                    />
                    {syncing ? "Syncing..." : "Sync Now"}
                </Button>
            </div>

            <h3 class="text-lg font-bold text-gray-300 mt-6">Artist Images</h3>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <ToggleCard
                    title="Auto-fetch Profile Pictures"
                    description="Automatically fetch artist profile pictures from the web when scanning your library."
                    bind:checked={settings.autoFetchArtistPic}
                    onchange={(v) => setSetting("autoFetchArtistPic", v)}
                    icon={User}
                    iconActiveClass="text-purple-400"
                />
                <ToggleCard
                    title="Auto-fetch Banner Images"
                    description="Automatically fetch artist banner images from the web when scanning your library."
                    bind:checked={settings.autoFetchArtistBanner}
                    onchange={(v) => setSetting("autoFetchArtistBanner", v)}
                    icon={Image}
                    iconActiveClass="text-sky-400"
                />
            </div>

            <h3 class="text-lg font-bold text-gray-300 mt-6">System</h3>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <ToggleCard
                    title="Keep Running in Background"
                    description="Keep Amus running in the system tray when you close the window instead of quitting."
                    bind:checked={settings.keepRunningInBg}
                    onchange={(v) => setSetting("keepRunningInBg", v)}
                    icon={Minimize2}
                    iconActiveClass="text-orange-400"
                />
            </div>

            <h3 class="text-lg font-bold text-gray-300 mt-6">Shortcuts</h3>
            <button
                class="flex items-center justify-between w-full px-8 py-4 bg-card/50 backdrop-blur-lg rounded-3xl shadow-lg border border-border cursor-pointer hover:bg-card/70 transition-colors text-left"
                command="show-modal"
                commandfor="shortcut-settings-modal"
                onclick={() => (showShortcutModal = true)}
            >
                <div class="flex items-center gap-3">
                    <Keyboard size={20} class="text-foreground" />
                    <Label class="text-xl font-bold text-white"
                        >Keyboard Shortcuts</Label
                    >
                </div>
                <span class="text-gray-400 text-sm">Customize</span>
            </button>

            <h3 class="text-lg font-bold text-gray-300 mt-6">Updates</h3>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <ToggleCard
                    title="Auto-check for Updates"
                    description="Automatically check for updates when Amus starts. If an update is found, you'll be notified."
                    bind:checked={settings.autoCheckUpdates}
                    onchange={(v) => setSetting("autoCheckUpdates", v)}
                    icon={Download}
                    iconActiveClass="text-emerald-400"
                />
                <div
                    class="flex items-center justify-between p-5 bg-card/50 backdrop-blur-lg rounded-3xl shadow-lg border border-border"
                >
                    <div class="flex flex-col">
                        <Label class="text-lg font-semibold text-white"
                            >Current Version</Label
                        >
                        <span class="text-sm text-gray-400 mt-1"
                            >{updater.currentVersion || "Loading..."}</span
                        >
                    </div>
                    <div class="flex items-center gap-3">
                        {#if updater.updateAvailable}
                            <span class="text-sm text-emerald-400 font-medium">
                                v{updater.updateAvailable.version} available
                            </span>
                            <Button
                                onclick={() => updater.downloadAndInstall()}
                                disabled={updater.downloading}
                                variant="default"
                                size="lg"
                            >
                                {updater.downloading
                                    ? "Installing..."
                                    : "Install"}
                            </Button>
                        {:else if updater.checking}
                            <Button disabled variant="outline">
                                <RotateCcw size={20} class="animate-spin" />
                                Checking...
                            </Button>
                        {:else}
                            <Button
                                onclick={() => updater.checkForUpdates()}
                                variant="outline"
                            >
                                <RotateCcw size={20} />
                                Check for Updates
                            </Button>
                        {/if}
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>
