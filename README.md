# AMUS

<div align="center">

<img src="static/icon.svg" alt="Amus" width="256" height="256" />

**A fast local modern music player**

AMUS is a fast, privacy-focused local music player built for users who own their music library. It works completely offline and is designed to feel modern while remaining lightweight.

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Tauri](https://img.shields.io/badge/Tauri-FFC131?style=for-the-badge&logo=Tauri&logoColor=white)
![SvekteKit](https://img.shields.io/badge/SvelteKit-FF3E00?style=for-the-badge&logo=Svelte&logoColor=white)
![License](https://img.shields.io/badge/MPL--2.0-blue?style=for-the-badge)

</div>

## Features

- **Audio playback** — MP3, FLAC, WAV, OGG, M4A, AAC, OPUS via rodio
- **Fast library scanning** — Incremental scanning with automatic metadata extraction.
- **Real-time file watcher** — automatically detects added, modified, and deleted files
- **Advanced Queue** — play next, drag & drop reorder, shuffle and repeat.
- **Auto-regeneration** — when queues run dry, similar tracks are suggested based on artist/album match, play count, and randomness
- **Playback history & stats** — play counts, listening time trends, streaks, library growth, format distribution, hourly/weekday heatmaps, favorite trends
- **Playlists** — create, rename, delete; add/remove tracks; auto-generated cover art
- **Favorites** — one-click toggle per track
- **Artist metadata** — automatic profile + banner fetching from Bing / DuckDuckGo
- **Mini player** — compact window with album art, track info, and controls
- **System tray**
- **Auto-updater**
- **Background playback**

## Installation

<p align="center">
  <a href="https://github.com/Naitik4516/AMUS/releases/latest">
  <img alt="GitHub Downloads (all assets, latest release)" src="https://img.shields.io/github/downloads/Naitik4516/AMUS/latest/total?style=social">
  </a>
</p>

**Supported Platforms**

- ✅ Windows (x64)
- ✅ Linux (x64)
- ✅ macOS (Intel & Apple Silicon)

## Screenshots

<p align="center">
  <img src=".github/images/home.webp" width="45%">
  <img src=".github/images/player+queue+search.webp" width="45%">
</p>

<p align="center">
  <img src=".github/images/aritsts.webp" width="45%">
  <img src=".github/images/artist_page.webp" width="45%">
</p>

<p align="center">
  <img src=".github/images/albums.webp" width="45%">
  <img src=".github/images/album_page.webp" width="45%">
</p>

<p align="center">
  <img src=".github/images/stats.webp" width="45%">
  <img src=".github/images/settings.webp" width="45%">
</p>

## Build & Run

### Prerequisites

1. [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites)
2. [Bun](https://bun.sh/)

```bash
git clone https://github.com/Naitik4516/AMUS.git
cd AMUS

bun install
bun tauri dev
```

## Architecture

<details>
<summary>Click to expand</summary>

```
amus/
├── src/                          # SvelteKit frontend (SSR off, adapter-static with fallback)
│   ├── lib/
│   │   ├── player.svelte.ts      # PlayerState singleton (Svelte 5 $state runes), Tauri event listener
│   │   ├── commands.svelte.ts    # invoke() wrappers for all ~50 Tauri commands
│   │   ├── data.svelte.ts        # Data-fetching helpers (playlists, cover art)
│   │   ├── settings.svelte.ts    # tauri-plugin-store backed settings (booleans, defaults in code)
│   │   ├── stats.svelte.ts       # Stats state management
│   │   ├── update.svelte.ts       # Auto-updater wrapper
│   │   ├── utils.ts              # getImageUrl(), formatDuration(), cn()
│   │   └── types.d.ts            # All shared TS types (Track, Artist, Album, Playlist, etc.)
│   ├── components/               # shadcn-svelte ui/, Menu/, Card/, Home/, stats/
│   ├── routes/                   # (main)/ library|artists|albums|playlists|favourites|stats|settings|track/[id]
│   │                             # miniplayer/ (separate Tauri window)
│   └── styles/
│       ├── theme.css             # Tailwind v4 @theme + 6 themes
│       └── fonts.css             # Custom font faces
├── src-tauri/                    # Rust backend
│   └── src/
│       ├── main.rs               # Entry point (calls amus_lib::run)
│       ├── lib.rs                # App setup: plugins, DB pool, tray, sync, player actor, ~50 commands
│       ├── commands.rs           # Tauri command handlers (async, use State<DbPool> / State<PlayerHandle>)
│       ├── db.rs                 # SQLite schema (3 migrations), queries, stats (rusqlite + r2d2 pool)
│       ├── engine/               # Audio engine (rodio), queue management, shuffle/repeat
│       │   ├── mod.rs
│       │   ├── engine.rs         # rodio-based audio playback
│       ├── player/
│       │   ├── actor.rs          # PlayerActor: runs on a dedicated thread, receives PlayerCommand via mpsc
│       │   ├── engine.rs         # rodio-based audio playback
│       │   ├── events.rs         # Tauri event emission (player://event channel)
│       │   ├── playback.rs       # Playback state machine
│       │   ├── queue.rs          # Queue data structures (user_queue + context_queue)
│       │   └── source.rs         # PlaybackSource / RepeatMode types
│       ├── scanner.rs            # Library scanning (rayon parallel, lofty metadata, cover art extraction)
│       ├── sync.rs               # SyncManager: startup scan + notify file watcher (Create/Modify/Remove)
│       ├── models.rs             # Shared Rust types (serde Serialize/Deserialize)
│       ├── error.rs              # thiserror Error enum -> serialized as strings
│       └── artist_pic_fetcher.rs # Bing/DuckDuckGo image scraping (primp + scraper)
├── migrations/                   # rusqlite_migration SQL files (001, 002, 003)
├── static/                        # Static assets (favicon, icons)
├── components.json              # shadcn-svelte config (alias: $components -> src/components)
├── package.json
├── svelte.config.js
└── tsconfig.json
```

</details>

## Tech Stack

| Layer         | Technology                                   |
| ------------- | -------------------------------------------- |
| Desktop shell | Tauri v2                                     |
| Frontend      | SvelteKit 5 (SPA, SSR off)                   |
| UI            | shadcn-svelte, Tailwind CSS v4, Lucide Icons |
| Rust backend  | rodio, rusqlite, r2d2, lofty, rayon, notify  |
| Artist images | primp (HTTP), scraper, Bing/DuckDuckGo       |
| Audio formats | MP3, FLAC, WAV, OGG, M4A, AAC, OPUS          |

### Recently completed

- [x] Mini player with always-on-top option
- [x] Enhanced search
- [x] Global and local keyboard shortcuts

### Library & Playback

- [ ] Smart playlists
- [ ] Music recommendations
- [ ] Gapless playback & silence skipping
- [ ] Crossfade
- [ ] Equalizer
- [ ] Audio normalization
- [ ] DSP effects
- [ ] Sleep timer

### Library Management

- [ ] Automatic metadata tagging
- [ ] Lyrics support
- [ ] Advanced search

### User Experience

- [ ] Dynamic theming
- [ ] Improved UI animations
- [ ] Better OS integration
- [ ] Auto-start and scheduled playback

### Media

- [ ] Video playback support

## FAQ

1. **Is this a vibe-coded project? Is it completely AI-generated?**  
    No. AMUS is a personal project that I designed and built myself.
   I do use AI tools, including the Opencode coding agent and AI code completion, to help with implementation and bug fixing — especially while I was learning Rust or working on boilerplate and low-level code. Some files, such as parts of the library scanner and synchronization logic, were largely generated with AI assistance.
   All AI-generated code has been reviewed, tested, and integrated by me. I make the architectural decisions, develop new features, fix bugs, and maintain the codebase.
