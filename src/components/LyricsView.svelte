<script lang="ts">
    import {
        getTrackLyrics,
        fetchLyricsFromLrclib,
    } from "$lib/commands.svelte";
    import type { Lyrics } from "$lib/types";
    import { settings } from "$lib/settings.svelte";
    import { Music, MicVocal, FileText, Minus, Plus } from "@lucide/svelte";
    import type { Action } from "svelte/action";
    import Button from "./ui/button/button.svelte";

    let {
        trackId = $bindable(0),
        position = $bindable(0),
        isPlaying = $bindable(false),
        onSeek = (sec: number) => {},
    } = $props();

    let fontSize = $state(28);

    let lyrics = $state<Lyrics | null>(null);
    let loading = $state(false);
    let error = $state<string | null>(null);
    let viewMode = $state<"synced" | "plain">("synced");
    let prevLineIndex = $state(-1);
    let lineElements: (HTMLElement | undefined)[] = [];
    let lyricsContainer: HTMLElement | null = $state(null);
    let userScrolling = $state(false);
    let scrollTimeout: ReturnType<typeof setTimeout> | null = null;

    function parseLrc(lrc: string): { time: number; text: string }[] {
        const lines: { time: number; text: string }[] = [];
        const textLines = lrc.split("\n");

        for (const raw of textLines) {
            const line = raw.trimEnd();
            if (!line) continue;

            const regex = /\[(\d{2}):(\d{2})(?:[.:](\d{2,3}))?\]/g;
            let match;
            const times: number[] = [];
            let lastIndex = 0;

            while ((match = regex.exec(line)) !== null) {
                const mins = parseInt(match[1]);
                const secs = parseInt(match[2]);
                let ms = 0;
                if (match[3]) {
                    const msStr = match[3];
                    ms =
                        msStr.length === 2
                            ? parseInt(msStr) * 10
                            : parseInt(msStr);
                }
                times.push(mins * 60 + secs + ms / 1000);
                lastIndex = regex.lastIndex;
            }

            const text = line.slice(lastIndex).trim();

            if (times.length > 0) {
                for (const time of times) {
                    lines.push({ time, text });
                }
            } else if (text && !/^\[\w+:/i.test(line.trim())) {
                lines.push({ time: -1, text });
            }
        }

        return lines.sort((a, b) => {
            if (a.time < 0 && b.time < 0) return 0;
            if (a.time < 0) return 1;
            if (b.time < 0) return -1;
            return a.time - b.time;
        });
    }

    let parsedLines = $derived.by(() => {
        if (!lyrics?.synced_lyrics) return [];
        return parseLrc(lyrics.synced_lyrics);
    });

    let currentLineIndex = $derived.by(() => {
        if (parsedLines.length === 0 || parsedLines[0].time < 0) return -1;
        let idx = -1;
        for (let i = 0; i < parsedLines.length; i++) {
            const t = parsedLines[i].time;
            if (t >= 0 && position >= t) {
                idx = i;
            } else if (t > position) {
                break;
            }
        }
        return idx;
    });

    let plainParagraphs = $derived.by(() => {
        if (!lyrics?.plain_lyrics) return [];
        return lyrics.plain_lyrics
            .trim()
            .split("\n")
            .map((l) => l.trimEnd());
    });

    let hasBoth = $derived(!!(lyrics?.plain_lyrics && lyrics?.synced_lyrics));

    let displayMode = $derived<"synced" | "plain" | "none">(
        viewMode === "synced" && parsedLines.length > 0
            ? "synced"
            : viewMode === "plain" && plainParagraphs.length > 0
              ? "plain"
              : parsedLines.length > 0
                ? "synced"
                : plainParagraphs.length > 0
                  ? "plain"
                  : "none",
    );

    $effect(() => {
        const id = trackId;
        if (!id) {
            lyrics = null;
            loading = false;
            error = null;
            return;
        }

        let cancelled = false;
        loading = true;
        error = null;
        lineElements = [];

        getTrackLyrics(id)
            .then((result) => {
                if (cancelled) return;
                lyrics = result;
                loading = false;
                if (result?.synced_lyrics) {
                    viewMode = "synced";
                } else {
                    viewMode = "plain";
                }

                if (!result && settings.autoFetchLyrics) {
                    fetchLyricsFromLrclib(id).then((found: boolean) => {
                        if (cancelled) return;
                        if (found) {
                            getTrackLyrics(id).then((r) => {
                                if (cancelled) return;
                                lyrics = r;
                                if (r?.synced_lyrics) {
                                    viewMode = "synced";
                                }
                            });
                        }
                    });
                }
            })
            .catch((e) => {
                if (cancelled) return;
                error =
                    e instanceof Error ? e.message : "Failed to load lyrics";
                loading = false;
            });

        return () => {
            cancelled = true;
        };
    });

    $effect(() => {
        const idx = currentLineIndex;
        if (
            idx < 0 ||
            idx === prevLineIndex ||
            userScrolling ||
            !lyricsContainer
        )
            return;
        prevLineIndex = idx;

        const el = lineElements[idx];
        if (!el) return;

        const containerRect = lyricsContainer.getBoundingClientRect();
        const elRect = el.getBoundingClientRect();
        const isVisible =
            elRect.top >= containerRect.top + 60 &&
            elRect.bottom <= containerRect.bottom - 60;

        if (!isVisible) {
            el.scrollIntoView({ behavior: "smooth", block: "center" });
        }
    });

    function handleUserScroll() {
        userScrolling = true;
        if (scrollTimeout) clearTimeout(scrollTimeout);
        scrollTimeout = setTimeout(() => {
            userScrolling = false;
        }, 3000);
    }

    function handleLineClick(time: number) {
        if (time >= 0) {
            onSeek(time);
        }
    }

    const captureRef: Action<HTMLElement, number> = (node, i) => {
        lineElements[i] = node;
        return {
            destroy() {
                if (lineElements[i] === node) {
                    lineElements[i] = undefined;
                }
            },
        };
    };

    const fontSizeMin = 14;
    const fontSizeMax = 40;
</script>

<div class="flex flex-col h-full" role="region" aria-label="Lyrics">
    {#if displayMode !== "none"}
        <div class="flex justify-end gap-1.5 p-2 shrink-0">
            <Button
                onclick={() => (fontSize = Math.max(fontSizeMin, fontSize - 2))}
                variant="outline"
                size="icon-sm"
                disabled={fontSize <= fontSizeMin}
                aria-label="Decrease font size"
            >
                <Minus size={12} />
            </Button>
            <Button
                onclick={() => (fontSize = Math.min(fontSizeMax, fontSize + 2))}
                variant="outline"
                size="icon-sm"
                disabled={fontSize >= fontSizeMax}
                aria-label="Increase font size"
            >
                <Plus size={12} />
            </Button>
        </div>
    {/if}

    <div
        bind:this={lyricsContainer}
        class="flex-1 overflow-y-auto px-4 py-2 scroll-smooth font-semibold items-center mask-y-from-80%"
        class:flex={displayMode === "none" || loading || !!error}
        class:items-center={displayMode === "none" || loading || !!error}
        onscroll={handleUserScroll}
    >
        {#if loading}
            <div
                class="absolute top-1/2 w-full flex flex-col items-center justify-center gap-4 text-muted-foreground"
            >
                <div
                    class="size-9 border-2 border-muted-foreground/30 border-t-foreground rounded-full animate-spin"
                ></div>
                <p class="font-bold font-satoshi">Loading lyrics...</p>
            </div>
        {:else if error}
            <div
                class="absolute top-1/2 w-full text-center text-muted-foreground"
            >
                Failed to load lyrics
            </div>
        {:else if displayMode === "synced"}
            <div class="flex flex-col gap-3 py-4">
                {#each parsedLines as line, i}
                    {#if line.time >= 0}
                        <button
                            use:captureRef={i}
                            onclick={() => handleLineClick(line.time)}
                            class="w-full text-center transition-all duration-300 px-2 py-1 rounded-lg border-l-2 border-transparent font-satoshi font-extrabold"
                            class:text-gray-500={i !== currentLineIndex}
                            class:font-extrabold={i === currentLineIndex}
                            style="font-size: {i === currentLineIndex
                                ? fontSize + 3
                                : fontSize}px; line-height: 1.8"
                            tabindex="-1"
                        >
                            {line.text || "\u00A0"}
                        </button>
                    {:else if line.text}
                        <p
                            class="text-center text-gray-600 text-sm py-2 select-none"
                        >
                            {line.text}
                        </p>
                    {/if}
                {/each}
            </div>
        {:else if displayMode === "plain"}
            <div
                class="flex flex-col gap-3 py-4 text-center max-w-2xl mx-auto"
                style="font-size: {fontSize}px; line-height: 1.8"
            >
                {#each plainParagraphs as line}
                    <p class="text-gray-200 select-text">
                        {line || "\u00A0"}
                    </p>
                {/each}
            </div>
        {:else}
            <div
                class="absolute top-1/2 w-full mb-4 mr-4 text-center text-muted-foreground"
            >
                <MicVocal size={56} class="m-auto mb-4 " />
                <p class="text-lg font-statoshi font-bold">
                    No lyrics available
                </p>
            </div>
        {/if}
    </div>

    {#if hasBoth}
        <div class="flex justify-center p-3 shrink-0">
            <div
                class="inline-flex items-center rounded-full bg-white/5 border p-0.75 gap-0.5"
            >
                <button
                    onclick={() => (viewMode = "synced")}
                    class="flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-medium transition-all {viewMode ===
                    'synced'
                        ? 'bg-white/10 text-white'
                        : 'text-muted-foreground'}"
                >
                    <Music size={12} />
                    Synced
                </button>
                <button
                    onclick={() => (viewMode = "plain")}
                    class="flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-medium transition-all {viewMode ===
                    'plain'
                        ? 'bg-white/10 text-white'
                        : 'text-muted-foreground'}"
                >
                    <FileText size={12} />
                    Plain
                </button>
            </div>
        </div>
    {/if}
</div>
