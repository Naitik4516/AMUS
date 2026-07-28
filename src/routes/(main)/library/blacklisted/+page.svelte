<script lang="ts">
    import { onMount } from "svelte";
    import { getScanBlacklist, unblacklistPath, scanLibrary } from "$lib/commands.svelte";
    import { toast } from "svelte-sonner";
    import { Button } from "$components/ui/button/index.js";
    import type { BlacklistedEntry } from "$lib/types";
    import { ShieldBan, RotateCcw, RefreshCw, Ban } from "@lucide/svelte";

    let entries = $state<BlacklistedEntry[]>([]);
    let loading = $state(true);
    let restoring = $state<Set<string>>(new Set());

    onMount(async () => {
        await loadEntries();
    });

    async function loadEntries() {
        loading = true;
        try {
            entries = await getScanBlacklist();
        } catch (e) {
            console.error("Failed to load blacklist:", e);
            toast.error("Failed to load blacklisted files");
        } finally {
            loading = false;
        }
    }

    async function handleRestore(path: string) {
        restoring.add(path);
        try {
            await unblacklistPath(path);
            entries = entries.filter((e) => e.path !== path);
            toast.success("File restored — it will be re-added on next scan");
        } catch (e) {
            console.error("Failed to unblacklist:", e);
            toast.error("Failed to restore file");
        } finally {
            restoring.delete(path);
        }
    }

    async function handleRestoreAndScan(path: string) {
        restoring.add(path);
        try {
            await unblacklistPath(path);
            await scanLibrary();
            entries = entries.filter((e) => e.path !== path);
            toast.success("File restored and library scanned");
        } catch (e) {
            console.error("Failed to restore and scan:", e);
            toast.error("Failed to restore file");
        } finally {
            restoring.delete(path);
        }
    }

    function reasonLabel(reason: string): string {
        if (reason === "user_deleted") return "Deleted by user";
        if (reason.startsWith("corrupted:")) return "Corrupted file";
        return reason;
    }

    function formatPath(path: string): string {
        const parts = path.split("/");
        const filename = parts.pop() || path;
        const dir = parts.slice(-2).join("/");
        return dir ? `${dir}/${filename}` : filename;
    }

    function formatDate(dateStr: string): string {
        try {
            const d = new Date(dateStr);
            return d.toLocaleDateString(undefined, {
                year: "numeric",
                month: "short",
                day: "numeric",
                hour: "2-digit",
                minute: "2-digit",
            });
        } catch {
            return dateStr;
        }
    }
</script>

<div class="p-8 max-w-4xl mx-auto">
    <div class="flex items-center justify-between mb-8">
        <div>
            <h1 class="text-2xl font-bold text-white flex items-center gap-3">
                <ShieldBan size={28} class="text-zinc-400" />
                Blacklisted Files
            </h1>
            <p class="text-zinc-400 text-sm mt-1">
                Files that are skipped during library scans
            </p>
        </div>
        <Button variant="ghost" onclick={loadEntries} disabled={loading}>
            <RefreshCw size={16} class={loading ? "animate-spin" : ""} />
            Refresh
        </Button>
    </div>

    {#if loading}
        <div class="flex items-center justify-center py-20">
            <RefreshCw size={32} class="animate-spin text-zinc-500" />
        </div>
    {:else if entries.length === 0}
        <div class="flex flex-col items-center justify-center py-20 text-zinc-500">
            <Ban size={48} class="mb-4" />
            <p class="text-lg font-medium">No blacklisted files</p>
            <p class="text-sm mt-1">
                Deleted tracks and corrupted files will appear here
            </p>
        </div>
    {:else}
        <div class="space-y-2">
            {#each entries as entry (entry.path)}
                <div
                    class="flex items-center justify-between gap-4 p-4 rounded-xl bg-white/5 border border-zinc-700/30"
                >
                    <div class="flex-1 min-w-0">
                        <p class="text-white text-sm font-medium truncate" title={entry.path}>
                            {formatPath(entry.path)}
                        </p>
                        <p class="text-zinc-400 text-xs mt-1 flex items-center gap-2">
                            <span
                                class={`inline-block px-2 py-0.5 rounded text-xs font-medium ${
                                    entry.reason === "user_deleted"
                                        ? "bg-red-500/10 text-red-400"
                                        : "bg-amber-500/10 text-amber-400"
                                }`}
                            >
                                {reasonLabel(entry.reason)}
                            </span>
                            <span>{formatDate(entry.created_at)}</span>
                        </p>
                    </div>
                    <div class="flex gap-2 shrink-0">
                        <Button
                            variant="ghost"
                            size="sm"
                            disabled={restoring.has(entry.path)}
                            onclick={() => handleRestore(entry.path)}
                        >
                            {#if restoring.has(entry.path)}
                                <RefreshCw size={14} class="animate-spin mr-1" />
                            {:else}
                                <RotateCcw size={14} class="mr-1" />
                            {/if}
                            Restore
                        </Button>
                        <Button
                            variant="outline"
                            size="sm"
                            disabled={restoring.has(entry.path)}
                            onclick={() => handleRestoreAndScan(entry.path)}
                        >
                            Restore & Rescan
                        </Button>
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</div>
