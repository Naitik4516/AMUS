# Amus - Local Music Player

Tauri v2 + SvelteKit + Rust desktop app. SPA mode (SSR disabled).

## Commands

```bash
bun run dev          # Frontend dev server (port 1420)
bun run build        # Production frontend build
bun run check        # svelte-check type checking (run after frontend changes)
bun run tauri dev    # Full Tauri dev (frontend + Rust)
bun run tauri build  # Full production build

# Rust-only
cargo check          # Type-check Rust code
cargo build          # Build Rust code
```

Package manager is **bun**. The Tauri CLI invokes `bun run dev`/`bun run build` automatically via `tauri.conf.json` beforeDevCommand/beforeBuildCommand.

## Architecture

- `src-tauri/src/` — Rust backend
  - `lib.rs` — app setup, plugin registration, command handler wiring
  - `commands.rs` — all `#[tauri::command]` functions (frontend invokes these)
  - `db.rs` — SQLite schema + queries (rusqlite + r2d2 pool)
  - `engine/` — audio playback via rodio (AudioEngine, queue, playback monitor)
  - `scanner.rs` — library scanning with rayon parallelism
  - `models.rs` — shared data types (Track, TrackDetails, Artist, Album, Playlist)
  - `error.rs` — Error enum with thiserror, serializable for frontend
- `src/lib/` — shared TypeScript
  - `player.svelte.ts` — PlayerState singleton (Svelte 5 runes), manages queue/playback state
  - `commands.svelte.ts` — Tauri invoke wrappers
- `src/components/` — Svelte components
- `src/routes/` — SvelteKit pages (library: artists, albums, playlists, favourites, trackdetails)
- `src/styles/theme.css` — theme system via CSS custom properties + `data-theme` attribute

## Key Conventions

- **Svelte 5 runes**: use `$state`, `$props`, `.svelte.ts` files — NOT the old `$:` / `stores` pattern
- **Tailwind CSS v4**: uses `@theme` directive in CSS (not `tailwind.config.js`). Custom properties in `src/styles/theme.css`.
- **No native window decorations**: window is borderless (`decorations: false`), custom Header component handles titlebar/dragging
- **Custom URL schemes**: album art at `amus-art://localhost/{filename}`, artist pics at `amus-artist://localhost/{filename}`
- **SQLite database**: auto-created at app_data_dir/music.db on startup (no migrations, uses `CREATE TABLE IF NOT EXISTS`)
- **Rust edition 2024**: requires rust 1.95+

## Testing

No test framework is currently configured. No `test` script in package.json. If adding tests, check existing tooling choices first.

## Linting / Formatting

No linter or formatter is configured. No eslint, prettier, or similar config exists.
