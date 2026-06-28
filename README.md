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
- **Artist images** — automatic profile + banner fetching from Bing / DuckDuckGo
- **System tray** — play/pause, previous/next, show/hide, quit\
- **Auto-updater** — GitHub releases with passive install on Windows
- **Smooth scrolling** — locomotive-scroll (toggleable in settings)
- **Background playback** — keeps alive in system tray on window close

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
</details>

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri v2 |
| Frontend | SvelteKit 5 (SPA, SSR off) |
| UI | shadcn-svelte, Tailwind CSS v4, Lucide Icons |
| Rust backend | rodio, rusqlite, r2d2, lofty, rayon, notify |
| Artist images | primp (HTTP), scraper, Bing/DuckDuckGo |
| Audio formats | MP3, FLAC, WAV, OGG, M4A, AAC, OPUS |


## Roadmap
- [ ] Support lyrics
- [ ] Support podcasts
- [ ] Support video playback
- [ ] Better os integration
- [ ] Dynamic theming 
- [ ] Global and local keyboard shortcuts
- [ ] Mini player with always-on-top option
- [ ] Recommendation System
- [ ] Better Search
- [ ] Smart Playlists
- [ ] Auto tagging
- [ ] Sleep timer
- [ ] auto start and schedule auto playback
- [ ] Gapless Playback & skip silence
- [ ] Equalizer
- [ ] Audio Normalization
- [ ] Crossfade
- [ ] Advanced Search
- [ ] DSP Effects

## License
AMUS is licensed under the [Mozilla Public License 2.0](https://www.mozilla.org/en-US/MPL/2.0/). See [LICENSE](LICENSE) for details.
