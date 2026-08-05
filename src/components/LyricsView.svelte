<script lang="ts">
    import {
        getTrackLyrics,
        fetchLyricsFromLrclib,
    } from "$lib/commands.svelte";
    import type { Lyrics } from "$lib/types";
    import { settings } from "$lib/settings.svelte";
    import {
        Music,
        MicVocal,
        FileText,
        Minus,
        Plus,
        RefreshCw,
    } from "@lucide/svelte";
    import type { Action } from "svelte/action";
    import Button from "./ui/button/button.svelte";

    let {
        trackId = $bindable(0),
        position = $bindable(0),
        isPlaying = $bindable(false),
        onSeek = (sec: number) => {},
        fontSize = $bindable(28),
    } = $props();

    let lyrics = $state<Lyrics | null>(null);
    let loading = $state(false);
    let error = $state<string | null>(null);
    let autoFetchFailed = $state(false);
    let reloadKey = $state(0);
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
            .replace(/\r\n/g, "\n")
            .split(/\n+/)
            .map((l) => l.trim())
            .filter(Boolean);
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
        void reloadKey;
        if (!id) {
            lyrics = null;
            loading = false;
            error = null;
            autoFetchFailed = false;
            return;
        }

        let cancelled = false;
        loading = true;
        error = null;
        autoFetchFailed = false;
        lineElements = [];
        prevLineIndex = -1;
        userScrolling = false;
        if (scrollTimeout) {
            clearTimeout(scrollTimeout);
            scrollTimeout = null;
        }

        async function run() {
            try {
                let result = await getTrackLyrics(id);
                if (cancelled) return;
                lyrics = result;
                viewMode = result?.synced_lyrics ? "synced" : "plain";

                if (!result && settings.autoFetchLyrics) {
                    let found = false;
                    try {
                        found = await fetchLyricsFromLrclib(id);
                    } catch (e) {
                        if (cancelled) return;
                        autoFetchFailed = true;
                        loading = false;
                        return;
                    }
                    if (cancelled) return;
                    if (found) {
                        const r = await getTrackLyrics(id);
                        if (cancelled) return;
                        lyrics = r;
                        if (r?.synced_lyrics) {
                            viewMode = "synced";
                        }
                    }
                }

                if (cancelled) return;
                loading = false;
            } catch (e) {
                if (cancelled) return;
                error =
                    e instanceof Error ? e.message : "Failed to load lyrics";
                loading = false;
            }
        }

        const timer = setTimeout(run, LYRIC_LOAD_DELAY_MS);

        return () => {
            cancelled = true;
            clearTimeout(timer);
        };
    });

    $effect(() => {
        const idx = currentLineIndex;
        if (
            idx < 0 ||
            idx === prevLineIndex ||
            userScrolling ||
            loading ||
            !lyricsContainer
        )
            return;
        prevLineIndex = idx;

        const el = lineElements[idx];
        if (!el) return;

        const container = lyricsContainer;
        const targetTop =
            el.offsetTop - container.clientHeight / 2 + el.offsetHeight / 2;
        const maxTop = container.scrollHeight - container.clientHeight;
        container.scrollTo({
            top: Math.max(0, Math.min(targetTop, maxTop)),
            behavior: "smooth",
        });
    });

    function handleUserScrollStart() {
        userScrolling = true;
        if (scrollTimeout) clearTimeout(scrollTimeout);
        scrollTimeout = setTimeout(() => {
            userScrolling = false;
        }, 3000);
    }

    const trackUserScroll: Action<HTMLElement> = (node) => {
        node.addEventListener("wheel", handleUserScrollStart, {
            passive: true,
        });
        node.addEventListener("touchstart", handleUserScrollStart, {
            passive: true,
        });
        node.addEventListener("pointerdown", handleUserScrollStart);
        return {
            destroy() {
                node.removeEventListener("wheel", handleUserScrollStart);
                node.removeEventListener("touchstart", handleUserScrollStart);
                node.removeEventListener("pointerdown", handleUserScrollStart);
            },
        };
    };

    function handleLineClick(time: number) {
        if (time >= 0) {
            userScrolling = false;
            if (scrollTimeout) {
                clearTimeout(scrollTimeout);
                scrollTimeout = null;
            }
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
    const LYRIC_LOAD_DELAY_MS = 400;
</script>

<div
    class="flex flex-col h-full"
    role="region"
    aria-label="Lyrics"
    aria-busy={loading}
>
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
        use:trackUserScroll
        class="relative flex-1 overflow-y-auto px-4 pt-20 font-bold flex flex-col mask-y-from-70% font-satoshi"
    >
        {#if loading}
            <div class="my-auto w-full flex flex-col items-center gap-4 px-8">
                <div
                    class="w-full max-w-md flex flex-col justify-center gap-3.5"
                    aria-hidden="true"
                >
                    {#each [100, 92, 97, 85, 90, 72, 84, 60, 76, 48] as w, i}
                        <div
                            class="h-4 rounded-full bg-white/10 animate-pulse mx-auto"
                            style="width: {w}%; {i % 2 === 1
                                ? 'margin-left: 4%'
                                : ''}"
                        ></div>
                    {/each}
                </div>
            </div>
        {:else if error}
            <div
                class="my-auto w-full flex flex-col items-center gap-4 px-8 text-muted-foreground"
            >
                <MicVocal size={56} class="opacity-60" />
                <p class="text-lg font-satoshi font-bold">
                    Failed to load lyrics
                </p>
                <p class="text-sm text-center opacity-80 max-w-sm">{error}</p>
                <Button variant="outline" size="sm" onclick={() => reloadKey++}>
                    <RefreshCw size={14} class="mr-1.5" />
                    Retry
                </Button>
            </div>
        {:else if displayMode === "synced"}
            <div class="flex flex-col gap-3 py-4">
                {#each parsedLines as line, i}
                    {#if line.time >= 0}
                        <button
                            use:captureRef={i}
                            onclick={() => handleLineClick(line.time)}
                            class="w-full text-center transition-colors duration-300 px-2 py-1  leading-relaxed"
                            class:text-gray-500={i !== currentLineIndex}
                            class:text-white={i === currentLineIndex}
                            class:font-black={i === currentLineIndex}
                            style="font-size: {i === currentLineIndex
                                ? fontSize + 4
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
                class="whitespace-pre-wrap select-text py-4 text-center max-w-2xl  leading-loose"
                style="font-size: {fontSize}px"
            >
                {lyrics?.plain_lyrics?.replace(/\\n/g, "\n")}
            </div>
        {:else}
            <div
                class="my-auto w-full mb-4 mr-4 text-center text-muted-foreground"
            >
                <MicVocal size={56} class="m-auto mb-4" />
                <p class="text-lg font-satoshi font-bold">
                    No lyrics available
                </p>
                {#if autoFetchFailed}
                    <p class="text-sm mt-2">
                        Couldn't fetch lyrics from LRCLIB.
                    </p>
                    <Button
                        variant="ghost"
                        size="sm"
                        class="mt-3"
                        onclick={() => reloadKey++}
                    >
                        <RefreshCw size={14} class="mr-1.5" />
                        Try again
                    </Button>
                {/if}
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
