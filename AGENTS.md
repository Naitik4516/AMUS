# AMUS — Agent Guide

**Local music player.** Tauri v2 + SvelteKit 5 SPA + Rust (rodio, rusqlite, lofty, rayon).

## Commands

| Action | Command |
|---|---|
| Frontend dev server (port 1420) | `bun run dev` |
| Full Tauri dev (hot-reload) | `bun run tauri dev` |
| Frontend type-check | `bun run check` (`svelte-kit sync && svelte-check`) |
| Frontend type-check (watch) | `bun run check:watch` |
| Rust type-check | `cargo check` (run inside `src-tauri/`) |
| Production build | `bun run tauri build` |

No linter, formatter, or test infrastructure exists. CI triggers on `v*` tags and manual dispatch (`.github/workflows/release.yml`).

## Architecture

- **Entrypoints:** `src-tauri/src/main.rs` → `amus_lib::run()`, `src/routes/+layout.svelte` (root Svelte layout)
- **Routes grouped** under `(main)/` route group — pages in `src/routes/(main)/library/`, `albums/`, `artists/`, `playlists/`, `favourites/`, `stats/`, `settings/`, `track/[id]`
- **Player state:** `src/lib/player.svelte.ts` — singleton class using Svelte 5 runes (`$state`, `$derived`, `$effect`)
- **Dual queue:** `userQueue` (user-added) + `playNext` (auto-suggested) → derived `fullQueue`
- **Rust crate named `amus_lib`** (avoids collision with binary on Windows). Edition 2024 (Rust 1.95+).
- **No CSP** (`"csp": null` in tauri.conf.json)
- **Cover art** at `$APPDATA/covers/`, artists at `$APPDATA/artists/`, banners at `$APPDATA/artist_banner/`

## UI conventions

- **Borderless window** (`decorations: false`) — custom resize handles on bottom/right/bottom-right edges
- **Background tray:** app hides instead of closing (`CloseRequested` intercepted). Tray icon toggles a separate mini-player window.
- **Tauri capabilities split by window** — `default.json` (main), `mini-player.json`, `desktop.json` (global shortcuts, autostart, updater, positioner)
- **Smooth scroll** via locomotive-scroll, toggleable in settings (dynamically initialized/destroyed)
- **Tailwind v4** via `@tailwindcss/vite` plugin (no PostCSS config)
- **shadcn-svelte** configured in `components.json` — `$components/ui/` for UI primitives, `$lib/utils` for `cn()`, icon library is Lucide
- **Dark mode** via `mode-watcher` package
- **6 themes** defined in `src/styles/theme.css` as Tailwind v4 `@theme` blocks

## Key source files

| File | What it is |
|---|---|
| `src-tauri/src/main.rs` | Binary entrypoint → `amus_lib::run()`. `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` |
| `src-tauri/src/lib.rs` | App builder, tray setup, popup window, 70+ command registrations |
| `src-tauri/src/commands.rs` | 72 `#[tauri::command]` handlers (CRUD, playback, queue, stats) |
| `src-tauri/src/db.rs` | SQLite schema + queries + stats aggregation (~2340 lines) |
| `src-tauri/src/player/` | `actor.rs` (command loop, `PlayerCommand` enum), `engine.rs` (rodio playback), `queue.rs`, `playback.rs`, `source.rs` (source/`RepeatMode`), `events.rs` |
| `src-tauri/src/scanner.rs` | Library scanner (lofty metadata, rayon parallelism, cover art extraction) |
| `src-tauri/src/sync.rs` | Startup scan + notify-based file watcher |
| `src-tauri/src/artist_pic_fetcher.rs` | Bing/DuckDuckGo image scraping for artist art |
| `src-tauri/src/models.rs` | Shared types (`Track`, `Album`, `Artist`, etc.) |
| `src-tauri/migrations/` | 3 SQLite migration files (`001_initial_schema`, `002_add_album_artist`, `003_add_fetch_tracking`) |
| `src/lib/player.svelte.ts` | Central frontend player state (Svelte 5 runes) |
| `src/lib/commands.svelte.ts` | Tauri invoke wrappers |
| `src/lib/data.svelte.ts` | Data fetching helpers |
| `src/lib/settings.svelte.ts` | Settings via `@tauri-apps/plugin-store` |
| `src/lib/types.d.ts` | TypeScript type aliases — check before defining new interfaces |

## Package management

- **Frontend:** Bun (`bun.lock`, no npm/yarn/pnpm lockfiles)
- **Backend:** Cargo (`src-tauri/Cargo.lock`, `src-tauri/Cargo.toml`)
- Dual `.gitignore` files: root covers frontend; `src-tauri/.gitignore` covers Rust/Tauri build artifacts

## Gotchas

- SvelteKit is SPA-only: `adapter-static` with `fallback: "index.html"`, `export const ssr = false` in `+layout.ts`
- Tailwind v4 uses `@tailwindcss/vite` plugin — no `tailwind.config.*` or PostCSS config file
- Svelte compiler option `experimental.async: true` enabled in `svelte.config.js`
- Path alias `$components` → `src/components/` defined in `svelte.config.js`
- Settings persisted via `@tauri-apps/plugin-store` (not localStorage)
- Dev server locked to port **1420** (strictPort in vite.config.js, referenced in tauri.conf.json)
- New Tauri plugins: register in both `lib.rs` (Rust `.plugin()`) and the appropriate capability file
- Type aliases live in `src/lib/types.d.ts` — refer there before defining new interfaces
- Generated `.svelte-kit/` dir is gitignored; `tsconfig.json` extends `.svelte-kit/tsconfig.json`
