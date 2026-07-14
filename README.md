# AMUS

<div align="center">

<img src="static/icon.svg" alt="AMUS" width="200" height="200" />

**A fast, modern, privacy-focused local music player**

AMUS is built for people who own their music library. It runs completely offline, stays lightweight, and feels like a modern desktop app — not a web wrapper.

[![License: MPL-2.0](https://img.shields.io/badge/License-MPL--2.0-blue?style=for-the-badge)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-FFC131?style=for-the-badge&logo=tauri&logoColor=white)](https://tauri.app/)
[![SvelteKit](https://img.shields.io/badge/SvelteKit-FF3E00?style=for-the-badge&logo=svelte&logoColor=white)](https://kit.svelte.dev/)
[![GitHub release](https://img.shields.io/github/v/release/Naitik4516/AMUS?style=for-the-badge&label=v0.5.0)](https://github.com/Naitik4516/AMUS/releases/latest)

</div>

## Features

### Playback

- **Wide format support** — MP3, FLAC, WAV, OGG, M4A, AAC, OPUS (via rodio)
- **Advanced queue** — play next, drag-and-drop reorder, shuffle, and repeat
- **Auto-regeneration** — when the queue runs dry, similar tracks are suggested from artist/album match, play count, and randomness
- **Background playback** — keeps playing from the system tray when the window is closed
- **Mini player** — compact always-available window with art, track info, and controls
- **Stop / pause / seek** — full transport controls plus stop

### Library

- **Fast incremental scanning** — metadata extraction and cover art on scan
- **Real-time file watcher** — picks up added, modified, and deleted files automatically
- **Playlists** — create, rename, delete; add/remove tracks; custom or auto-generated cover art; quick “Add more” search on the playlist page
- **Favorites** — one-click toggle per track
- **Artist metadata** — automatic profile and banner images (Bing / DuckDuckGo)
- **In-memory library cache** — library loads once at startup for snappy browsing and fewer IPC round-trips

### Search & navigation

- **Fuzzy global search** — client-side [Fuse.js](https://www.fusejs.io/) with [extended search](https://www.fusejs.io/examples.html#extended-search) patterns
- **Type filters** — `/tracks`, `/artists`, `/albums`, `/playlists` slash commands
- **Context menus** — right-click tracks for play, queue, playlist, favorite, and more
- **Keyboard shortcuts** — app-wide and optional global media shortcuts (customizable)

### Insights & polish

- **Playback history & stats** — play counts, listening time, streaks, library growth, format distribution, hourly/weekday heatmaps, favorite trends
- **System tray** — play/pause, previous/next, show/hide, quit
- **Auto-updater** — updates from GitHub Releases (passive install on Windows)
- **OS media controls** — integrate with system media keys (MPRIS/SMTC/Now Playing)
- **File associations** — open audio files directly with AMUS
- **Modern UI** — custom title bar, themes, and a responsive library layout

## Advanced search

Open global search and type normally for fuzzy matching, or use these **extended patterns** to refine results (powered by Fuse.js extended search). You can combine them with type filters like `/tracks belver` or `/artists ^Tu`.

| Token      | Match type                 | Description                                       |
| ---------- | -------------------------- | ------------------------------------------------- |
| `belver`   | fuzzy-match                | Items that fuzzy match _belver_ (e.g. “Believer”) |
| `="Rebel"` | exact-match                | Items that are exactly _Rebel_                    |
| `'lofi`    | include-match              | Items that include _lofi_                         |
| `!lofi`    | inverse-exact-match        | Items that do not include _lofi_                  |
| `^Tu`      | prefix-exact-match         | Items that start with _Tu_                        |
| `!^Tu`     | inverse-prefix-exact-match | Items that do not start with _Tu_                 |
| `na$`      | suffix-exact-match         | Items that end with _na_                          |
| `!na$`     | inverse-suffix-exact-match | Items that do not end with _na_                   |

**Tips**

- Whitespace-separated terms are AND’d together (all must match).
- Use `|` for OR (e.g. `'jazz | 'blues`).
- Prefix a query with a slash command to limit type: `/albums ^The`, `/tracks !live$`.
- Tab accepts the ghost suggestion when one is shown.

### CLI Interface

The `amus` binary doubles as a remote control for the running app. If AMUS isn't running, it's auto-started in the background. Use `amus help` to see everything, or `amus <command> --help` for details on a specific command.

| Command | Description |
|---------|-------------|
| `play [paths...]` | Resume playback, or play files/folders/globs |
| `play -s <query>` | Play the top search result for a query |
| `pause` | Pause playback |
| `stop` | Stop playback |
| `toggle` | Toggle play/pause |
| `next` | Skip to the next track |
| `prev` | Go to the previous track |
| `seek <value>` | Seek to a position (e.g. `90`) or by offset (`+10`, `-5`) |
| `volume <value>` | Set volume as a percent (e.g. `80`) or adjust (`+5`, `-10`) |
| `mute` | Toggle mute |
| `status` | Show current track, playback state, and position |
| `queue add [paths...]` | Add files/folders/globs to the end of the queue |
| `queue add -s <query>` | Add search results to the queue |
| `queue clear` | Clear the user queue |
| `queue shuffle` | Toggle shuffle on/off |
| `queue show` | Print the current queue contents |
| `library rescan` | Rescan all library sources |
| `search [scope] <query>` | Search the library (`artist:`, `album:`, or bare `track:`) |
| `playlist [name]` | Show a playlist's contents (omit to list all) |
| `playlist create <name>` | Create a new playlist |
| `playlist play <name>` | Play a playlist |
| `playlist add <name> <path>` | Add a track to a playlist |
| `playlist remove <name> <path>` | Remove a track from a playlist |
| `playlist delete <name>` | Delete a playlist |
| `albums` | List all albums |
| `artists` | List all artists |
| `album <id_or_name>` | Show an album's track listing |
| `artist <id_or_name>` | Show an artist's track listing |
| `import <path>` | Import a folder as a library source and scan it |
| `info <path>` | Print local metadata for an audio file (no server needed) |
| `open` | Show and focus the main window |
| `hide` | Hide the main window |
| `close` | Close or hide the main window |
| `update` | Check for and install updates |
| `version` | Print the AMUS version |

**Examples**

```
amus                                                # start the GUI
amus ~/Music/album.flac ~/Downloads/track.mp3       # play specific files
amus ~/Music                                        # play an entire folder
amus play -s "artist:Radiohead"                     # play the top artist match
amus next                                           # skip to next track
amus seek +30                                       # jump forward 30 seconds
amus volume 60                                      # set volume to 60%
amus queue add ~/Music/New\ Albums/*.flac           # glob and queue
amus queue show                                     # see what's coming next
amus playlist create "Late Night"                   # new empty playlist
amus playlist add "Late Night" ~/Music/jazz.mp3     # add a track to it
amus playlist play "Late Night"                     # start playing it
amus albums                                         # browse the library
amus search album "In Rainbows"                     # find an album
amus info ~/Downloads/unknown.flac                  # peek at metadata
```

**Notes**

- Paths accept glob patterns (`**/*.flac`) and can be files or directories — everything is scanned recursively for audio.
- The `-s` / `--search` flag is only available on `play` and `queue add`. For general lookups use `search`.
- Seek and volume values prefixed with `+` or `-` are treated as relative adjustments; bare numbers are absolute.
- On first invocation of any command, AMUS launches in the background if it isn't already running.

## Installation

<p align="center">
  <a href="https://github.com/Naitik4516/AMUS/releases/latest">
    <img alt="Download latest release" src="https://img.shields.io/github/v/release/Naitik4516/AMUS?style=for-the-badge&label=Download" />
  </a>
  &nbsp;
  <a href="https://github.com/Naitik4516/AMUS/releases/latest">
    <img alt="GitHub Downloads (all assets, latest release)" src="https://img.shields.io/github/downloads/Naitik4516/AMUS/latest/total?style=for-the-badge" />
  </a>
</p>

Grab the latest build from **[Releases](https://github.com/Naitik4516/AMUS/releases/latest)**.

| Platform | Arch                  |
| -------- | --------------------- |
| Windows  | x64                   |
| Linux    | x64                   |
| macOS    | Intel & Apple Silicon |

## Screenshots

<p align="center">
  <img src=".github/images/home.webp" width="45%" alt="Home">
  <img src=".github/images/player+queue+search.webp" width="45%" alt="Player, queue, and search">
</p>

<p align="center">
  <img src=".github/images/aritsts.webp" width="45%" alt="Artists">
  <img src=".github/images/artist_page.webp" width="45%" alt="Artist page">
</p>

<p align="center">
  <img src=".github/images/albums.webp" width="45%" alt="Albums">
  <img src=".github/images/album_page.webp" width="45%" alt="Album page">
</p>

<p align="center">
  <img src=".github/images/stats.webp" width="45%" alt="Stats">
  <img src=".github/images/settings.webp" width="45%" alt="Settings">
</p>

## Build & run

### Prerequisites

1. [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/) (Rust, platform deps)
2. [Bun](https://bun.sh/)

```bash
git clone https://github.com/Naitik4516/AMUS.git
cd AMUS

bun install
bun tauri dev
```



## Architecture

<details>
<summary>Project layout (click to expand)</summary>

```
amus/
├── src/                              # SvelteKit frontend (SPA, SSR off)
│   ├── lib/
│   │   ├── player.svelte.ts          # PlayerState ($state runes) + event listener
│   │   ├── stores.svelte.ts          # Library store (tracks/albums/artists/playlists)
│   │   ├── commands.svelte.ts        # invoke() wrappers for Tauri commands
│   │   ├── settings.svelte.ts        # tauri-plugin-store settings
│   │   ├── shortcuts.svelte.ts       # App + global shortcut definitions
│   │   ├── stats.svelte.ts           # Stats state
│   │   ├── update.svelte.ts          # Auto-updater
│   │   ├── utils.ts                  # Image URLs, duration formatting, cn()
│   │   └── types.d.ts                # Shared TS types
│   ├── components/                   # UI (shadcn-svelte, menus, cards, stats)
│   ├── routes/
│   │   ├── (main)/                   # Library, artists, albums, playlists, …
│   │   └── miniplayer/               # Separate mini-player window
│   └── styles/                       # Tailwind v4 theme + fonts
├── src-tauri/                        # Rust / Tauri backend
│   ├── migrations/                   # SQLite migrations (rusqlite_migration)
│   └── src/
│       ├── lib.rs                    # App setup: plugins, DB, tray, player, commands
│       ├── commands.rs               # Tauri command handlers
│       ├── db.rs                     # Schema, queries, stats
│       ├── player/                   # Actor-based playback (rodio)
│       ├── scanner.rs                # Parallel library scan (rayon + lofty)
│       ├── sync.rs                   # Startup scan + notify file watcher
│       └── artist_pic_fetcher.rs     # Artist image scraping
├── static/                           # Icons and static assets
└── package.json
```

**How it fits together**

- **Player actor** — `PlayerActor` runs on its own thread; the UI talks to it via commands and listens on `player://event`.
- **SQLite library** — pooled DB at app data (`music.db`); WAL mode for concurrent reads.
- **Frontend store** — library data is loaded once at startup into Svelte 5 state for fast UI and client-side search.

</details>

## Tech stack

| Layer         | Technology                                         |
| ------------- | -------------------------------------------------- |
| Desktop shell | Tauri v2                                           |
| Frontend      | SvelteKit 5 (SPA, SSR off), Svelte 5 runes         |
| UI            | shadcn-svelte, Tailwind CSS v4, Lucide             |
| Search        | Fuse.js (client-side fuzzy + extended search)      |
| Backend       | Rust — rodio, rusqlite, r2d2, lofty, rayon, notify |
| Artist images | primp + scraper (Bing / DuckDuckGo)                |
| Audio formats | MP3, FLAC, WAV, OGG, M4A, AAC, OPUS                |

## Roadmap

### Recently completed

- [x] OS media controls (MPRIS/SMTC/Now Playing)
- [x] Command-line interface for remote control
- [x] File associations (open audio files with AMUS)
- [x] Mini player with always-on-top option
- [x] Fuzzy global search (Fuse.js + type filters)
- [x] Global and local keyboard shortcuts
- [x] Track context menus (right-click)
- [x] Playlist cover art + quick "Add more" flow
- [x] Startup library cache for snappier UI
- [x] Comprehensive frontend test coverage (Vitest)
- [x] Player subsystem refactor for better architecture

### Library & playback

- [ ] Smart playlists
- [ ] Music recommendations
- [ ] Gapless playback & silence skipping
- [ ] Crossfade
- [ ] Equalizer
- [ ] Audio normalization
- [ ] DSP effects
- [ ] Sleep timer

### Library management

- [ ] Automatic metadata tagging
- [ ] Lyrics support

### User experience

- [ ] Dynamic theming
- [ ] Improved UI animations
- [ ] Better OS integration
- [ ] Auto-start and scheduled playback

### Media

- [ ] Video playback support

## FAQ

**Is this a vibe-coded / fully AI-generated project?**  
No. AMUS is a personal project I designed and built. I use AI tools (including coding agents and autocomplete) for implementation help and boilerplate — especially while learning Rust or working through low-level pieces. Some areas (for example parts of the scanner and sync logic) had substantial AI assistance. Everything is reviewed, tested, and integrated by me; architecture, features, and maintenance are intentional human decisions.

**Does it need the cloud or an account?**  
No. Your library stays on your machine. Artist image fetch is the only optional network use for metadata art.

**Where is my data stored?**  
In the app data directory (SQLite library DB, cover art, settings). Nothing is uploaded for playback.

## License

[MPL-2.0](LICENSE)
