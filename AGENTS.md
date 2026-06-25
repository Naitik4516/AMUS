# Amus — Local Music Player

Tauri v2 + SvelteKit 5 (SPA mode, SSR disabled) + Rust desktop app.

## Commands

```bash
bun run dev            # Frontend dev server (port 1420)
bun run build          # Production frontend build
bun run check          # svelte-check (run `svelte-kit sync` + typecheck)
bun run tauri dev      # Full Tauri dev (frontend + Rust)
bun run tauri build    # Full production build

# Rust-only
cargo check            # Type-check Rust code
cargo build            # Build Rust code
```

Tauri CLI invokes `bun run dev` / `bun run build` automatically via `tauri.conf.json` `beforeDevCommand` / `beforeBuildCommand`.

## Architecture

### Rust backend (`src-tauri/src/`)

| File | Purpose |
|------|---------|
| `lib.rs` | App setup, plugin registration, all command handler wiring, DB pool + audio engine init |
| `main.rs` | Windows subsystem attr + calls `amus_lib::run()` |
| `commands.rs` | All `#[tauri::command]` fns (~50 commands) |
| `db.rs` | SQLite schema + queries (rusqlite + r2d2 pool); includes `track_stats` view, `playback_history`, `user_queue` |
| `engine/` | Audio playback via rodio (`AudioEngine`, queue, shuffle/repeat, playback monitor) |
| `scanner.rs` | Library scanning via rayon parallelism; differential scan (mtime comparison), orphan cleanup, embedded cover art extraction via lofty |
| `sync.rs` | `SyncManager` — startup auto-scan on first launch + `notify`-based real-time file watcher; controlled by `settings.json` store keys `syncOnStartup`, `realtimeSync` |
| `artist_pic_fetcher.rs` | Scrapes Bing/DuckDuckGo for artist profile+banner images via `primp` HTTP client, saves as WebP to `app_data_dir/artists/` and `artist_banner/` |
| `models.rs` | Shared types: `Track`, `TrackDetails`, `Artist`, `Album`, `Playlist`, `RepeatMode`, `SourceType` |
| `error.rs` | `Error` enum via `thiserror`, serializable for frontend |

### Frontend (`src/`)

| Path | Role |
|------|------|
| `lib/player.svelte.ts` | `PlayerState` singleton (Svelte 5 runes); manages queue/playback state, listens to `playback-state` and `track-changed` events |
| `lib/commands.svelte.ts` | `invoke` wrappers for source management + scan |
| `lib/data.svelte.ts` | Data-fetching wrappers (albums, artists, playlists, artist images) |
| `lib/stores/toast.svelte.ts` | `ToastStore` singleton (success/error/info/warning) |
| `lib/utils.ts` | `getImageUrl()` — converts cover/artist/banner filenames to Tauri `asset://` URLs via `convertFileSrc` + `appDataDir` |
| `components/` | Header (titlebar/drag), Sidebar, Player, ScanProgress, ToastPortal, various UI components |
| `routes/` | SvelteKit pages: library dashboard, artists, albums, playlists, favourites, stats, track details, settings |
| `styles/theme.css` | Tailwind v4 `@theme` directive; CSS custom properties + `data-theme` for 6 themes |

### Key conventions

- **Svelte 5 runes**: `$state`, `$props`, `$derived`, `.svelte.ts` files — NOT old `$:` / stores pattern
- **Tailwind CSS v4**: `@theme` in CSS (no `tailwind.config.js`), `@tailwindcss/vite` plugin
- **Window**: borderless (`decorations: false`), custom Header handles titlebar/dragging
- **Asset protocol**: enabled with `$APPDATA/**/*` scope — images served via `asset://` URLs
- **SQLite**: auto-created at `app_data_dir/music.db` on startup, no migrations (`CREATE TABLE IF NOT EXISTS`)
- **SvelteKit alias**: `$components` → `src/components`
- **Rust edition 2024**: requires rust 1.95+
- **locomotive-scroll**: imported in `+layout.svelte` for smooth scrolling
