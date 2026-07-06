# AMUS — Agent Guide

## Stack

- **Desktop shell:** Tauri v2 (Rust backend, SvelteKit 5 frontend)
- **Frontend:** SvelteKit 5 (SPA mode, SSR off — `src/routes/+layout.ts` sets `ssr = false`)
- **UI:** shadcn-svelte (maia style, mauve base), Tailwind CSS v4, Lucide icons
- **Rust:** edition 2024, `rust-version = "1.95"`
- **Package manager:** Bun (not npm/pnpm)

## Commands

| Command | Purpose |
|---|---|
| `bun tauri dev` | Run full dev (Vite + Tauri backend with hot-reload) |
| `bun run dev` | Vite frontend only (no Tauri backend) |
| `bun run build` | Build frontend only (output: `build/`) |
| `bun run check` | Typecheck frontend (`svelte-kit sync && svelte-check`) |
| `bun run preview` | Vite preview of built frontend |

No test, lint, or format scripts are configured. `bun run check` is the only available verification command.

## Project Map

```
amus/
├── src/                         # SvelteKit frontend (SSR off, adapter-static with fallback)
│   ├── lib/
│   │   ├── player.svelte.ts     # PlayerState singleton (Svelte 5 $state runes), Tauri event listener
│   │   ├── commands.svelte.ts   # invoke() wrappers for all ~50 Tauri commands
│   │   ├── data.svelte.ts       # Data-fetching helpers (playlists, cover art)
│   │   ├── settings.svelte.ts   # tauri-plugin-store backed settings (booleans, defaults in code)
│   │   ├── stats.svelte.ts      # Stats state management
│   │   ├── update.svelte.ts     # Auto-updater wrapper
│   │   ├── utils.ts             # getImageUrl(), formatDuration(), cn()
│   │   └── types.d.ts           # All shared TS types (Track, Artist, Album, Playlist, etc.)
│   ├── components/              # Svelte components (shadcn-svelte ui/, Menu/, Card/, Home/, stats/)
│   ├── routes/                  # (main)/ library|artists|albums|playlists|favourites|stats|settings|track/[id]
│   │                            # miniplayer/ (separate Tauri window)
│   └── styles/
│       ├── theme.css            # Tailwind v4 @theme + 6 themes
│       └── fonts.css            # Custom font faces
├── src-tauri/                   # Rust backend
│   └── src/
│       ├── main.rs              # Entry point (calls amus_lib::run)
│       ├── lib.rs               # App setup: plugins, DB pool, tray, sync, player actor, ~50 commands
│       ├── commands.rs          # Tauri command handlers (async, use State<DbPool> / State<PlayerHandle>)
│       ├── db.rs                # SQLite schema (3 migrations), queries, stats (rusqlite + r2d2 pool)
│       ├── engine/              # Audio engine (rodio), queue management, shuffle/repeat
│       ├── player/
│       │   ├── actor.rs         # PlayerActor: runs on a dedicated thread, receives PlayerCommand via mpsc
│       │   ├── engine.rs        # rodio-based audio playback
│       │   ├── events.rs        # Tauri event emission (player://event channel)
│       │   ├── playback.rs      # Playback state machine
│       │   ├── queue.rs         # Queue data structures (user_queue + context_queue)
│       │   └── source.rs        # PlaybackSource / RepeatMode types
│       ├── scanner.rs           # Library scanning (rayon parallel, lofty metadata, cover art extraction)
│       ├── sync.rs              # SyncManager: startup scan + notify file watcher (Create/Modify/Remove)
│       ├── models.rs            # Shared Rust types (serde Serialize/Deserialize)
│       ├── error.rs             # thiserror Error enum -> serialized as strings
│       └── artist_pic_fetcher.rs # Bing/DuckDuckGo image scraping (primp + scraper)
├── migrations/                  # rusqlite_migration SQL files (001, 002, 003)
├── static/                      # Static assets (favicon, icons)
└── components.json              # shadcn-svelte config (alias: $components -> src/components)
```

## Architecture Notes

- **Player is actor-based:** `PlayerActor` runs on its own thread, receives `PlayerCommand` via `mpsc::Sender`, returns responses via `oneshot` channels. The `PlayerHandle` struct wraps the sender and is managed as Tauri state.
- **Events flow one-way:** Rust backend emits events on the `"player://event"` channel. The frontend `player.svelte.ts` listens with `listen("player://event", ...)` and updates `$state` runes.
- **DB pool** (`r2d2::Pool<SqliteConnectionManager>` at `$APPDATA/music.db`) is managed as Tauri state. All commands access it via `State<'_, DbPool>`.
- **SQLite pragmas:** `foreign_keys=ON`, `journal_mode=WAL`, `synchronous=NORMAL`, `temp_store=MEMORY`, `busy_timeout=5000`
- **3 migrations** in `src-tauri/migrations/` using `rusqlite_migration`.
- **Settings** stored via `tauri-plugin-store` (a JSON file). Defaults are hardcoded in `settings.svelte.ts`.
- **Frontend alias:** `$components` -> `src/components` (configured in `svelte.config.js` and `tsconfig.json`).
- **Vite dev server** always on port 1420 (strict), HMR on port 1421 if `TAURI_DEV_HOST` is set. `src-tauri/` is excluded from Vite watch.
- **Tauri window:** 1000x700 default, min 700x700, no decorations, transparent, macOS private API enabled.
- **CI:** Triggers on `v*` tags. Matrix: macOS (aarch64 + x86_64), ubuntu-24.04, windows-latest. Uses `tauri-apps/tauri-action@v1`.
- **Auto-updater** configured via GitHub releases (pubkey in `tauri.conf.json`). Only Windows has `passive` install mode.

## Gotchas

- **Svelte 5 runes** ($state, $derived, $effect) are used throughout. Do not use Svelte 4 `store` patterns (no `writable`, `derived`, etc.).
- **No SSR.** SvelteKit is in SPA mode (`adapter-static` with `fallback: "index.html"`). `$page`, `$app/environment`, and server-side code patterns do not apply.
- **Bun is required.** The `beforeDevCommand` and `beforeBuildCommand` in `tauri.conf.json` use `bun run`.
- **Known bug:** The file watcher in `sync.rs` does not handle `Modify(ModifyKind::Name(_))` events on Linux (files moved to Trash). See `.opencode/plans/fix-syncer-deletions.md` for the planned fix.
- **Tray icon** uses `toggle_popup()` on left-click, which shows/hides the mini-player webview window at `/miniplayer`.

## Installed OpenCode Skills

From `skills-lock.json`: `rust-skills`, `shadcn-svelte`, `tauri`. Skill files are in `.agents/skills/`.
