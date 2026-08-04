# AMUS — Agent Guide

## Stack

- **Desktop shell:** Tauri v2 (Rust backend, SvelteKit 5 frontend)
- **Frontend:** SvelteKit 5 (SPA mode, SSR off — `src/routes/+layout.ts` sets `ssr = false`)
- **UI:** shadcn-svelte (maia style, mauve base), Tailwind CSS v4, Lucide icons
- **Rust:** edition 2024, `rust-version = "1.95"`, `crate-type` includes `staticlib`/`cdylib`/`rlib` (lib named `amus_lib` to avoid Windows bin/lib collision)
- **Package manager:** Bun (not npm/pnpm); builds go through **Vite+** (`vp`) — see the Vite+ section at the bottom

## Commands

| Command           | Purpose                                                |
| ----------------- | ------------------------------------------------------ |
| `bun tauri dev`   | Run full dev (Vite + Tauri backend with hot-reload)    |
| `bun run dev`     | Vite frontend only (no Tauri backend)                  |
| `bun run build`   | Build frontend only (output: `build/`)                 |
| `bun run check`   | Typecheck frontend (`svelte-kit sync && svelte-check`) |
| `bun run test`    | Vitest frontend tests (183 tests across 6 files)       |
| `cargo test`      | Rust backend tests (162 tests across 6 files) — run from `src-tauri/` |
| `bun run test:watch` / `bun run check:watch` | Watch modes for the above |

Linux system deps for `bun tauri dev` / release builds (from `.github/workflows/release.yml`): `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libasound2-dev libgtk-layer-shell-dev`.

## Project Map

```
amus/
├── src/                         # SvelteKit frontend (SSR off, adapter-static with fallback)
│   ├── lib/
│   │   ├── player.svelte.ts     # PlayerState singleton (Svelte 5 $state runes), Tauri event listener
│   │   ├── commands.svelte.ts   # invoke() wrappers for all Tauri commands (89 in commands.rs)
│   │   ├── settings.svelte.ts   # tauri-plugin-store backed settings (defaults hardcoded here)
│   │   ├── data.svelte.ts       # Data-fetching helpers (playlists, cover art)
│   │   ├── stats.svelte.ts      # Stats state management
│   │   ├── startup.svelte.ts    # Boot-time flow (first-run folder pick, scanning)
│   │   ├── shortcuts.svelte.ts + shortcut-handler.svelte.ts  # Global shortcuts
│   │   ├── session.svelte.ts    # Per-window session state (main vs miniplayer)
│   │   ├── stores.svelte.ts     # Smaller UI state stores
│   │   ├── fullscreen.svelte.ts # Fullscreen player state
│   │   ├── context-menu.svelte.ts, update.svelte.ts, utils.ts, types.d.ts
│   ├── components/              # Player, Sidebar, FullscreenPlayer, LyricsView, QueueView, GlobalSearch,
│   │                            # ShortcutSettingsModal, ScanProgress, stats/, ui/ (shadcn-svelte)...
│   ├── routes/
│   │   ├── (main)/              # Main window: +page.svelte, library/ (albums|artists|blacklisted|
│   │   │                        #   favourites|genres|playlists|stats|track/[id]), settings/
│   │   └── miniplayer/          # Separate Tauri window ("mini-player" label, url: /miniplayer)
│   └── styles/                  # theme.css (Tailwind v4 @theme + 6 themes), fonts.css
├── src-tauri/
│   ├── migrations/              # rusqlite_migration SQL files (001..011) — NOT at repo root
│   ├── capabilities/            # Tauri v2 permissions: default.json, desktop.json, mini-player.json
│   ├── src/
│   │   ├── main.rs              # Entry point: decides CLI vs GUI mode (see Gotchas)
│   │   ├── lib.rs               # App setup: plugins, DB pool, tray, sync, player actor, ~89 commands
│   │   ├── commands.rs          # Tauri command handlers (async, State<DbPool> / State<PlayerHandle>)
│   │   ├── db.rs                # SQLite schema (11 migrations), queries, stats (rusqlite + r2d2 pool)
│   │   ├── player/              # actor.rs (dedicated thread, mpsc + oneshot), engine.rs (rodio),
│   │   │                        # events.rs (PLAYER_EVENT_NAME = "player://event"), playback.rs,
│   │   │                        # queue.rs (user_queue + context_queue), source.rs (PlaybackSource/RepeatMode)
│   │   ├── scanner.rs           # Library scanning (rayon parallel, lofty metadata, cover art extraction)
│   │   ├── sync.rs              # SyncManager: startup scan + notify file watcher
│   │   ├── media_controls.rs    # OS media keys via souvlaki, subscribes to player://event
│   │   ├── lyrics_fetcher.rs, artist_pic_fetcher.rs  # Network scrapers (requisitioned lazily)
│   │   ├── cli/                 # CLI mode: clap args, localhost server, protocol.rs (CliCommand enum)
│   │   ├── logging.rs           # tracing + tracing-appender setup
│   │   ├── startup.rs, models.rs, error.rs
│   ├── installer/nsis/path.nsh  # Windows NSIS installer hook
│   └── tauri.conf.json          # Windows: main + mini-player (see Architecture), updater pubkey
├── static/                      # Static assets (favicon, icons)
└── components.json              # shadcn-svelte config (alias: $components -> src/components)
```

## Architecture Notes

- **Player is actor-based:** `PlayerActor` runs on its own thread, receives `PlayerCommand` via `mpsc::Sender`, returns responses via `oneshot` channels. `PlayerHandle` wraps the sender and is managed as Tauri state.
- **Events flow one-way:** Rust emits on `"player://event"` (constant `PLAYER_EVENT_NAME` in `player/events.rs`). Frontend `player.svelte.ts` listens with `listen(...)` and updates `$state` runes.
- **DB pool** (`r2d2::Pool<SqliteConnectionManager>` at `$APPDATA/music.db`) is Tauri state; commands access it via `State<'_, DbPool>`. Migrations in `src-tauri/migrations/` applied at startup. Pragmas: `foreign_keys=ON`, `journal_mode=WAL`, `synchronous=NORMAL`, `temp_store=MEMORY`, `busy_timeout=5000`.
- **Windows:** `main` (1000x700, min 700x500) and `mini-player` (420x250, always-on-top, skipTaskbar, hidden until shown). Both: no decorations, transparent, `shadow: false`. `macOSPrivateApi` enabled.
- **Tray icon:** left-click toggles the mini-player window (`toggle_miniplayer` in `lib.rs`); menu items Show/Show Miniplayer/Quit; on Linux uses `ksni` for the tray.
- **CLI:** running the binary with any second arg other than `--gui` invokes CLI mode (`cli/mod.rs::is_cli_invocation`), which talks to a running instance over a localhost server (`cli/protocol.rs`). `amus <files...>` plays paths; `amus play -s "query"` searches; see `cli/args.rs`.
- **Settings** stored via `tauri-plugin-store` (JSON file). Defaults are hardcoded in `settings.svelte.ts`.
- **Frontend alias:** `$components` -> `src/components` (svelte.config.js and tsconfig.json).
- **CI:** `release.yml` builds/upload on `v*` tags (also `workflow_dispatch` with a tag input) — matrix macOS aarch64+x86_64, ubuntu-24.04, windows-latest, `tauri-apps/tauri-action@v1`, releases are drafts. `publish-aur.yml` manually publishes a tag to AUR (requires `v`-prefixed tag, PKGBUILD in `.github/aur/`).
- **Auto-updater** via GitHub releases (pubkey in `tauri.conf.json`), endpoints point at `Naitik4516/AMUS`. Only Windows has `passive` install mode.
- **File associations:** mp3/flac/wav/ogg/m4a/aac/opus are registered as file types; the CLI/`PlayPaths` flow handles "open with AMUS".

## Gotchas

- **Svelte 5 runes** ($state, $derived, $effect) are used throughout. Do not use Svelte 4 store patterns (no `writable`, `derived`, etc.).
- **No SSR.** SvelteKit is SPA-only (`adapter-static` with `fallback: "index.html"`). `$page`, `$app/environment`, and server-side patterns do not apply.
- **Bun is required.** `beforeDevCommand`/`beforeBuildCommand` in `tauri.conf.json` use `bun run`; `devEngines` enforces bun.
- **Tauri v2 permissions** live in `src-tauri/capabilities/` — new IPC/plugins need permission entries there (e.g. `mini-player.json` scopes the miniplayer window).
- **Updating test counts:** the numbers in the Commands table drift; run `bun run test` / `cargo test` and update if you change tests.

## Installed OpenCode Skills

From `skills-lock.json`: `rust-skills`, `shadcn-svelte`, `tauri`. Skill files are in `.agents/skills/`.

<!--VITE PLUS START-->

# Using Vite+, the Unified Toolchain for the Web

This project is using Vite+, a unified toolchain built on top of Vite, Rolldown, Vitest, tsdown, Oxlint, Oxfmt, and Vite Task. Vite+ wraps runtime management, package management, and frontend tooling in a single global CLI called `vp`. Vite+ is distinct from Vite, and it invokes Vite through `vp dev` and `vp build`. Run `vp help` to print a list of commands and `vp <command> --help` for information about a specific command.

Docs are local at `node_modules/vite-plus/docs` or online at https://viteplus.dev/guide/.

## Review Checklist

- [ ] Run `vp install` after pulling remote changes and before getting started.
- [ ] Run `vp check` and `vp test` to format, lint, type check and test changes.
- [ ] Check if there are `vite.config.ts` tasks or `package.json` scripts necessary for validation, run via `vp run <script>`.
- [ ] If setup, runtime, or package-manager behavior looks wrong, run `vp env doctor` and include its output when asking for help.

<!--VITE PLUS END-->
