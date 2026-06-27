# AMUS
<div align="center">

<img src="static/icon.svg" alt="Amus" width="120" height="120" />

**A fast local modern music player** 
AMUS is a fast, privacy-focused local music player built for users who own their music library. It works completely offline and is designed to feel modern while remaining lightweight.

</div>

## Features

- **Audio playback** — MP3, FLAC, WAV, OGG, M4A, AAC, OPUS via rodio
- **Fast Library scanning** — Incremental scanning with automatic metadata extraction.
- **Real-time file watcher** — automatically picks up new/deleted files via notify
- **Advanced Queue** — play next, drag & drop reorder, shuffle and repeat.
- **Auto-regeneration** — when queues run dry, similar tracks are suggested based on artist/album match, play count, and randomness
- **Playback history & stats** — play counts, listening time trends, streaks, library growth, format distribution, hourly/weekday heatmaps, favorite trends
- **Playlists** — create, rename, delete; add/remove tracks; auto-generated cover art
- **Favorites** — one-click toggle per track
- **Artist images** — automatic profile + banner fetching from Bing / DuckDuckGo
- **System tray** — play/pause, previous/next, show/hide, quit
- **Pop-up mini-player** — separate always-on-top window showing current track
- **Auto-updater** — GitHub releases with passive install on Windows
- **Custom borderless window** — draggable header, custom titlebar
- **Smooth scrolling** — locomotive-scroll (toggleable in settings)
- **Keyboard shortcuts** — global media keys via Tauri global-shortcut plugin
- **Background running** — keeps alive in system tray on window close

## Prerequisites

- **Rust** 1.95+ (edition 2024)
- **Bun** (for frontend tooling)
- **System deps** (Linux): `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libasound2-dev`

## Getting Started

```bash
# Install frontend deps
bun install

# Run in development mode (frontend on :1420, Tauri window with hot-reload)
bun run tauri dev

# Type-check frontend
bun run check

# Production build
bun run tauri build
```

To work on the frontend independently (without Tauri window):

```bash
bun run dev          # Vite dev server on http://localhost:1420
```

To work on Rust code independently:

```bash
cargo check          # Type-check only (fast)
cargo build          # Full Rust build (in src-tauri/)
```

## Architecture

```
amus/
├── src/                          # SvelteKit frontend (SSR disabled)
│   ├── lib/
│   │   ├── player.svelte.ts      # PlayerState singleton (Svelte 5 runes)
│   │   ├── commands.svelte.ts    # Tauri invoke wrappers
│   │   ├── data.svelte.ts        # Data-fetching helpers
│   │   ├── settings.svelte.ts    # Settings store (plugin-store)
│   │   ├── stats.svelte.ts       # Stats state management
│   │   ├── update.svelte.ts      # Auto-updater wrapper
│   │   ├── stores/toast.svelte.ts
│   │   ├── utils.ts              # getImageUrl, formatDuration, cn()
│   │   └── types.d.ts
│   ├── components/               # Header, Sidebar, Player, ScanProgress, UI
│   ├── routes/                   # Pages: library, artists, albums, playlists,
│   │                             # favourites, stats, track/[id], settings
│   └── styles/
│       ├── theme.css             # Tailwind v4 @theme + 6 themes
│       └── fonts.css             # Custom font faces
├── src-tauri/
│   └── src/
│       ├── lib.rs                # App setup, plugins, tray, popup window
│       ├── main.rs               # Entry point
│       ├── commands.rs           # ~50 #[tauri::command] handlers
│       ├── db.rs                 # SQLite schema, queries, stats (rusqlite + r2d2)
│       ├── engine/
│       │   ├── engine.rs         # AudioEngine (rodio), queue, shuffle/repeat
│       │   └── mod.rs
│       ├── scanner.rs            # Library scanning (rayon, lofty, cover art)
│       ├── sync.rs               # SyncManager (startup scan + notify watcher)
│       ├── artist_pic_fetcher.rs # Bing/DuckDuckGo image scraping
│       ├── models.rs             # Shared types (Track, Album, Artist, etc.)
│       └── error.rs              # thiserror Error enum
├── package.json
├── svelte.config.js
└── tsconfig.json
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri v2 |
| Frontend | SvelteKit 5 (SPA, SSR off) |
| UI | shadcn-svelte, Tailwind CSS v4, Lucide Icons |
| Rust backend | rodio, rusqlite, r2d2, lofty, rayon, notify |
| Artist images | primp (HTTP), scraper, Bing/DuckDuckGo |
| Audio formats | MP3, FLAC, WAV, OGG, M4A, AAC, OPUS |
