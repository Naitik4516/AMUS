# AMUS — Agent Guide

**Local music player.** Tauri v2 + SvelteKit 5 SPA (no SSR) + Rust (rodio, rusqlite, lofty).

## Commands

| Action | Command |
|---|---|
| Frontend dev server (port 1420) | `bun run dev` |
| Full Tauri dev (hot-reload) | `bun run tauri dev` |
| Frontend type-check | `bun run check` (`svelte-kit sync && svelte-check`) |
| Rust type-check | `cargo check` (run inside `src-tauri/`) |
| Production build | `bun run tauri build` |

No linter, formatter, or test infrastructure exists. CI only runs the release workflow (push to `release` branch).

## Architecture

- **Entrypoints:** `src-tauri/src/main.rs` → `amus_lib::run()`, `src/routes/+layout.svelte` (root layout)
- **Player state:** `src/lib/player.svelte.ts` — singleton class using Svelte 5 runes (`$state`, `$derived`, `$effect`)
- **Dual queue:** `userQueue` (user-added) + `playNext` (auto-suggested) → derived `fullQueue`
- **Rust crate is named `amus_lib`** (avoids collision with binary on Windows)
- **Rust edition 2024** (requires Rust 1.95+)
- **No CSP** (`"csp": null` in tauri.conf.json)
- **Cover art** stored at `$APPDATA/covers/`, `$APPDATA/artists/`, `$APPDATA/artist_banner/`

## UI

- **Borderless window** (`decorations: false`) — custom resize handles on bottom/right/bottom-right edges (`src/routes/+layout.svelte`)
- **Background tray:** app stays alive when window closes (intercepts `CloseRequested`, hides instead). Tray icon toggles a popup mini-player window.
- **Tauri capabilities split by window** — separate files for `main`, `popup`, `desktop` in `src-tauri/capabilities/`
- **Smooth scroll** via locomotive-scroll (toggleable in settings, dynamically initialized/destroyed)

## Key source files

| File | What it is |
|---|---|
| `src-tauri/src/lib.rs` | App builder, 75+ command handlers, tray, popup window setup |
| `src-tauri/src/commands.rs` | ~50 `#[tauri::command]` handlers (CRUD, playback, queue, stats) |
| `src-tauri/src/db.rs` | SQLite schema + queries + stats aggregation (~2340 lines) |
| `src-tauri/src/engine/engine.rs` | rodio-based playback, queue, shuffle/repeat logic |
| `src-tauri/src/scanner.rs` | Library scanner (lofty metadata, rayon parallelism, cover art) |
| `src-tauri/src/sync.rs` | Startup scan + notify-based file watcher |
| `src-tauri/migrations/001_initial_schema.sql` | SQLite schema |
| `src/lib/player.svelte.ts` | Central frontend player state (runes) |
| `src/lib/commands.svelte.ts` | Tauri invoke wrappers |
| `src/lib/data.svelte.ts` | Data fetching helpers |
| `src/lib/settings.svelte.ts` | Settings via Tauri plugin-store |

## Package management

- **Frontend:** Bun (`bun.lock`, no npm/yarn/pnpm lockfiles)
- **Backend:** Cargo (`src-tauri/Cargo.lock`)
- Dual `.gitignore` files: root covers frontend, `src-tauri/.gitignore` covers Rust/Tauri artifacts

## Gotchas

- SvelteKit is SPA-only: `adapter-static` with `fallback: "index.html"`, `export const ssr = false` in `+layout.ts`
- Settings are persisted via `@tauri-apps/plugin-store`, not localStorage
- The dev server must be on port **1420** (configured in vite.config.js and tauri.conf.json)
- If you add a new Tauri plugin, register it in both `lib.rs` (Rust) and the relevant capability file
- Type aliases in `src/lib/types.d.ts` — always check there before defining new interfaces
