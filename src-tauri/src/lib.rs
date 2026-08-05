pub mod artist_pic_fetcher;
pub mod cli;
pub mod commands;
pub mod db;
pub mod error;
pub mod logging;
pub mod lyrics_fetcher;
pub mod media_controls;
pub mod models;
pub mod player;
pub mod scanner;
pub mod startup;
pub mod sync;

use crate::player::actor::PlayerCommand;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use sync::SyncManager;
use tauri::{Emitter, Manager, WindowEvent};
use tauri_plugin_window_state::{StateFlags, WindowExt};

#[cfg(not(target_os = "linux"))]
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

pub(crate) struct MiniPlayerPinned(AtomicBool);

#[cfg(target_os = "linux")]
pub(crate) struct MiniPlayerUsesLayerShell(AtomicBool);

#[derive(Clone)]
pub(crate) enum TrayAnchor {
    Point(i32, i32),
    #[cfg_attr(target_os = "linux", allow(dead_code))]
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
}

pub(crate) struct ClickState {
    last_click: Mutex<Option<Instant>>,
    pending_single: Arc<AtomicBool>,
    delay: Duration,
}

impl ClickState {
    pub(crate) fn new(delay: Duration) -> Self {
        Self {
            last_click: Mutex::new(None),
            pending_single: Arc::new(AtomicBool::new(false)),
            delay,
        }
    }

    fn is_double_click(&self) -> bool {
        let now = Instant::now();
        let mut last = self.last_click.lock().unwrap_or_else(|e| e.into_inner());
        let double = last
            .map(|t| now.duration_since(t) < self.delay)
            .unwrap_or(false);
        *last = Some(now);
        double
    }

    fn run_single(&self, single: impl FnOnce() + Send + 'static) {
        self.pending_single.store(true, Ordering::Relaxed);
        let pending = self.pending_single.clone();
        let delay = self.delay;
        std::thread::spawn(move || {
            std::thread::sleep(delay);
            if pending.swap(false, Ordering::Relaxed) {
                single();
            }
        });
    }

    fn cancel_pending(&self) {
        self.pending_single.store(false, Ordering::Relaxed);
    }
}

#[cfg(not(target_os = "linux"))]
fn build_tray_menu(
    app: &tauri::AppHandle,
) -> tauri::Result<(Menu<tauri::Wry>, MenuItem<tauri::Wry>, MenuItem<tauri::Wry>)> {
    let play_pause = MenuItem::with_id(app, "play_pause", "Play/Pause", true, None::<&str>)?;
    let previous = MenuItem::with_id(app, "previous", "Previous", true, None::<&str>)?;
    let next = MenuItem::with_id(app, "next", "Next", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let show_miniplayer = MenuItem::with_id(
        app,
        "show_miniplayer",
        "Show Miniplayer",
        true,
        None::<&str>,
    )?;
    let show = MenuItem::with_id(app, "show", "Show/Hide", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &play_pause,
            &previous,
            &next,
            &separator,
            &show_miniplayer,
            &show,
            &separator,
            &quit,
        ],
    )?;
    Ok((menu, show, show_miniplayer))
}

#[cfg(not(target_os = "linux"))]
fn update_tray_labels(
    app: &tauri::AppHandle,
    show: &MenuItem<tauri::Wry>,
    show_miniplayer: &MenuItem<tauri::Wry>,
) {
    let main_visible = app
        .get_webview_window("main")
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false);
    let _ = show.set_text(if main_visible { "Hide" } else { "Show" });
    let mini_visible = app
        .get_webview_window("mini-player")
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false);
    let _ = show_miniplayer.set_text(if mini_visible {
        "Hide Miniplayer"
    } else {
        "Show Miniplayer"
    });
}

#[cfg(not(target_os = "linux"))]
fn handle_tray_menu(
    app: &tauri::AppHandle,
    event: tauri::menu::MenuEvent,
    show: MenuItem<tauri::Wry>,
    show_miniplayer: MenuItem<tauri::Wry>,
) {
    let handle = app.state::<commands::PlayerHandle>();
    match event.id().as_ref() {
        "play_pause" => {
            let _ = commands::send(&handle, PlayerCommand::PlayPause);
        }
        "next" => {
            let _ = commands::send(&handle, PlayerCommand::Next);
        }
        "previous" => {
            let _ = commands::send(&handle, PlayerCommand::Previous);
        }
        "show_miniplayer" => {
            toggle_miniplayer(app);
            update_tray_labels(app, &show, &show_miniplayer);
        }
        "show" => {
            toggle_main_visibility(app);
            update_tray_labels(app, &show, &show_miniplayer);
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}

fn toggle_miniplayer(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("mini-player") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

fn toggle_main_visibility(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

const TRAY_SNAP_MARGIN: i32 = 8;

fn snap_miniplayer_to_anchor(mini_win: &tauri::WebviewWindow<tauri::Wry>, anchor: &TrayAnchor) {
    let (ax, ay, aw, ah) = match anchor {
        TrayAnchor::Point(x, y) => (*x, *y, 0, 0),
        TrayAnchor::Rect {
            x,
            y,
            width,
            height,
        } => (*x as i32, *y as i32, *width as i32, *height as i32),
    };
    let size = match mini_win.outer_size() {
        Ok(s) => s,
        Err(_) => return,
    };
    let w = size.width as i32;
    let h = size.height as i32;

    let monitor = mini_win
        .app_handle()
        .available_monitors()
        .ok()
        .and_then(|monitors| {
            monitors.into_iter().find(|m| {
                let p = m.position();
                let s = m.size();
                ax >= p.x && ax < p.x + s.width as i32 && ay >= p.y && ay < p.y + s.height as i32
            })
        })
        .or_else(|| mini_win.current_monitor().ok().flatten());
    let Some(monitor) = monitor else { return };
    let area = *monitor.work_area();
    let area_right = area.position.x + area.size.width as i32;
    let area_bottom = area.position.y + area.size.height as i32;

    let center_x = ax + aw / 2;
    let above = ay < area.position.y + area.size.height as i32 / 2;
    let x = (center_x - w / 2).clamp(area.position.x, area_right - w);
    let y = if above {
        ay + ah + TRAY_SNAP_MARGIN
    } else {
        ay - h - TRAY_SNAP_MARGIN
    };
    let y = y.clamp(area.position.y, area_bottom - h);

    let _ = mini_win.set_position(tauri::PhysicalPosition { x, y });
}


fn toggle_miniplayer_from_tray(app: &tauri::AppHandle, anchor: Option<TrayAnchor>) {
    let Some(window) = app.get_webview_window("mini-player") else {
        return;
    };
    if !window.is_visible().unwrap_or(false)
        && let Some(anchor) = anchor
    {
        let can_position = {
            #[cfg(target_os = "linux")]
            {
                // Layer-shell (Wayland) windows are compositor-anchored and can't be freely positioned.
                !app.try_state::<MiniPlayerUsesLayerShell>()
                    .map(|s| s.0.load(Ordering::Relaxed))
                    .unwrap_or(false)
            }
            #[cfg(not(target_os = "linux"))]
            {
                true
            }
        };
        if can_position {
            snap_miniplayer_to_anchor(&window, &anchor);
        }
    }
    toggle_miniplayer(app);
}

fn saved_miniplayer_position(app: &tauri::AppHandle) -> Option<(i32, i32)> {
    let config_dir = app.path().app_config_dir().ok()?;
    let content = std::fs::read_to_string(config_dir.join(".window-state.json")).ok()?;
    let value: serde_json::Value = serde_json::from_str(&content).ok()?;
    let entry = value.get("mini-player")?;
    let x = entry.get("x")?.as_i64()?;
    let y = entry.get("y")?.as_i64()?;
    (x != 0 || y != 0).then_some((x as i32, y as i32))
}

fn position_bottom_right(mini_win: &tauri::WebviewWindow<tauri::Wry>) -> tauri::Result<()> {
    let monitor = mini_win
        .current_monitor()?
        .ok_or(tauri::Error::WindowNotFound)?;
    let area = *monitor.work_area();
    let size = mini_win.outer_size()?;
    const MARGIN: i32 = 8;
    mini_win.set_position(tauri::PhysicalPosition {
        x: area.position.x + area.size.width as i32 - size.width as i32 - MARGIN,
        y: area.position.y + area.size.height as i32 - size.height as i32 - MARGIN,
    })
}

fn setup_miniplayer_window(app: &tauri::AppHandle, mini_win: &tauri::WebviewWindow<tauri::Wry>) {
    #[cfg(target_os = "linux")]
    let layer_shell = {
        use gtk_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

        if gtk_layer_shell::is_supported() {
            if let Ok(gtk_win) = mini_win.gtk_window() {
                gtk_win.init_layer_shell();
                gtk_win.set_layer(Layer::Top);
                gtk_win.set_anchor(Edge::Bottom, true);
                gtk_win.set_anchor(Edge::Right, true);
                gtk_win.set_layer_shell_margin(Edge::Bottom, 12);
                gtk_win.set_layer_shell_margin(Edge::Right, 12);
                // Without keyboard interactivity the compositor never grants focus, so the unpin "hide on focus loss" never triggers on  Wayland. OnDemand lets the window be focused/unfocused like a normal window where the compositor supports it.
                gtk_win.set_keyboard_mode(KeyboardMode::OnDemand);
            }
            true
        } else {
            false
        }
    };

    #[cfg(target_os = "linux")]
    app.manage(MiniPlayerUsesLayerShell(AtomicBool::new(layer_shell)));

    if saved_miniplayer_position(app).is_some() {
        let _ = mini_win.restore_state(StateFlags::SIZE | StateFlags::POSITION);
    } else {
        #[cfg(target_os = "linux")]
        if !layer_shell {
            let _ = position_bottom_right(mini_win);
        }
        #[cfg(not(target_os = "linux"))]
        let _ = position_bottom_right(mini_win);
    }

    let _ = mini_win.hide();
}

#[cfg(target_os = "linux")]
mod linux_tray {
    use super::*;
    use ksni::{Icon, Orientation, Tray, blocking::TrayMethods, menu::StandardItem};

    const VOLUME_STEP: f32 = 0.05;

    pub(crate) struct AmusTray {
        pub app: tauri::AppHandle,
        pub icon: Option<Icon>,
    }

    impl Tray for AmusTray {
        fn id(&self) -> String {
            "amus".into()
        }

        fn title(&self) -> String {
            "AMUS".into()
        }

        fn icon_name(&self) -> String {
            "amus".into()
        }

        fn icon_pixmap(&self) -> Vec<Icon> {
            self.icon.clone().into_iter().collect()
        }

        fn activate(&mut self, x: i32, y: i32) {
            let click_state = self.app.state::<ClickState>();
            if click_state.is_double_click() {
                click_state.cancel_pending();
                toggle_main_visibility(&self.app);
            } else {
                let app = self.app.clone();
                click_state.run_single(move || {
                    toggle_miniplayer_from_tray(&app, Some(TrayAnchor::Point(x, y)))
                });
            }
        }

        fn secondary_activate(&mut self, _x: i32, _y: i32) {
            let handle = self.app.state::<commands::PlayerHandle>();
            let _ = commands::send(&handle, PlayerCommand::ToggleMute);
        }

        fn scroll(&mut self, delta: i32, orientation: Orientation) {
            if orientation == Orientation::Vertical && delta != 0 {
                let handle = self.app.state::<commands::PlayerHandle>();
                let _ = commands::send(
                    &handle,
                    PlayerCommand::AdjustVolume(delta.signum() as f32 * VOLUME_STEP),
                );
            }
        }

        fn menu_about_to_show(&mut self) {
            // ksni only rebuilds the menu when this hook is implemented.
            // Labels are computed from live window visibility in `menu()`.
        }

        fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
            let main_visible = self
                .app
                .get_webview_window("main")
                .and_then(|w| w.is_visible().ok())
                .unwrap_or(false);
            let mini_visible = self
                .app
                .get_webview_window("mini-player")
                .and_then(|w| w.is_visible().ok())
                .unwrap_or(false);

            let play = StandardItem {
                label: "Play/Pause".into(),
                activate: Box::new(|this: &mut Self| {
                    let handle = this.app.state::<commands::PlayerHandle>();
                    let _ = commands::send(&handle, PlayerCommand::PlayPause);
                }),
                ..Default::default()
            }
            .into();
            let previous = StandardItem {
                label: "Previous".into(),
                activate: Box::new(|this: &mut Self| {
                    let handle = this.app.state::<commands::PlayerHandle>();
                    let _ = commands::send(&handle, PlayerCommand::Previous);
                }),
                ..Default::default()
            }
            .into();
            let next = StandardItem {
                label: "Next".into(),
                activate: Box::new(|this: &mut Self| {
                    let handle = this.app.state::<commands::PlayerHandle>();
                    let _ = commands::send(&handle, PlayerCommand::Next);
                }),
                ..Default::default()
            }
            .into();
            let show_miniplayer = StandardItem {
                label: if mini_visible {
                    "Hide Miniplayer"
                } else {
                    "Show Miniplayer"
                }
                .into(),
                activate: Box::new(|this: &mut Self| toggle_miniplayer(&this.app)),
                ..Default::default()
            }
            .into();
            let show = StandardItem {
                label: if main_visible { "Hide" } else { "Show" }.into(),
                activate: Box::new(|this: &mut Self| toggle_main_visibility(&this.app)),
                ..Default::default()
            }
            .into();
            let quit = StandardItem {
                label: "Quit".into(),
                activate: Box::new(|this: &mut Self| this.app.exit(0)),
                ..Default::default()
            }
            .into();

            vec![
                play,
                previous,
                next,
                ksni::MenuItem::Separator,
                show_miniplayer,
                show,
                ksni::MenuItem::Separator,
                quit,
            ]
        }
    }

    #[allow(dead_code)]
    struct TrayHandle(ksni::blocking::Handle<AmusTray>);

    pub(crate) fn spawn_ksni_tray(app: &tauri::AppHandle) {
        let icon = app.default_window_icon().map(|img| {
            let rgba = img.rgba();
            let mut data = Vec::with_capacity(rgba.len());
            for px in rgba.chunks_exact(4) {
                data.extend_from_slice(&[px[3], px[0], px[1], px[2]]);
            }
            Icon {
                width: img.width() as i32,
                height: img.height() as i32,
                data,
            }
        });

        let tray = AmusTray {
            app: app.clone(),
            icon,
        };
        match tray.spawn() {
            Ok(handle) => {
                let _ = app.manage(TrayHandle(handle));
            }
            Err(e) => tracing::error!(error = %e, "failed to spawn ksni tray"),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().plugin(
        tauri_plugin_window_state::Builder::new()
            .skip_initial_state("mini-player")
            .build(),
    );

    #[cfg(debug_assertions)]
    let builder = {
        // The devtools plugin claims the global tracing subscriber. Attach a  log bridge so our file logging still receives events (falls back to the plain fmt subscriber in setup if devtools can't init).
        let mut devtools_builder = tauri_plugin_devtools::Builder::default();
        devtools_builder.attach_logger(logging::build_file_adapter(&logging::early_app_data_dir()));
        match devtools_builder.try_init() {
            Ok(plugin) => builder.plugin(plugin),
            Err(e) => {
                tracing::warn!(error = %e, "devtools plugin unavailable");
                builder
            }
        }
    };

    let startup_status = Arc::new(startup::StartupStatus::new());
    let startup_status_clone = startup_status.clone();

    let app = builder
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let w = app.get_webview_window("main").expect("no main window");
            let _ = w.show();
            let _ = w.set_focus();
        }))
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        let action_id = match shortcut.to_string().as_str() {
                            "MediaPlayPause" => "global_play_pause",
                            "MediaTrackNext" => "global_next_track",
                            "MediaTrackPrevious" => "global_prev_track",
                            "MediaStop" => "global_stop",
                            _ => return,
                        };
                        let _ = app.emit("global-shortcut", action_id);
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(startup_status)
        .setup(move |app| {
            let app_handle = app.handle();

            if let Ok(app_dir) = app_handle.path().app_data_dir() {
                let guard = logging::init(&app_dir);
                app.manage(guard);
            }

            if let Err(e) = (|| -> Result<(), String> {
                let app_dir = app_handle
                    .path()
                    .app_data_dir()
                    .map_err(|e| format!("failed to get app data dir: {e}"))?;
                std::fs::create_dir_all(&app_dir)
                    .map_err(|e| format!("failed to create app data dir: {e}"))?;
                let db_path = app_dir.join("music.db");

                let manager = SqliteConnectionManager::file(db_path).with_init(|c| {
                    c.execute_batch(
                        "PRAGMA foreign_keys = ON;\n\
                        PRAGMA journal_mode = WAL;\n\
                        PRAGMA synchronous = NORMAL;\n\
                        PRAGMA temp_store = MEMORY;\n\
                        PRAGMA busy_timeout = 5000;",
                    )
                });
                let pool =
                    Pool::new(manager).map_err(|e| format!("failed to create db pool: {e}"))?;

                {
                    let mut conn = pool
                        .get()
                        .map_err(|e| format!("failed to get db connection: {e}"))?;
                    db::init_db(&mut conn)
                        .map_err(|e| format!("failed to initialize database: {e}"))?;
                }

                let handle =
                    crate::player::actor::PlayerActor::spawn(app.handle().clone(), pool.clone());
                app.manage(commands::PlayerHandle(handle));
                app.manage(pool);

                let sync_manager = SyncManager::new();
                let sync_manager_clone = sync_manager.clone();
                app.manage(sync_manager);
                sync_manager_clone.init(app_handle);

                Ok(())
            })() {
                startup_status_clone.fail(&e);
                tracing::error!(error = %e, "startup error");
            } else {
                startup_status_clone.succeed();
            }

            app.manage(MiniPlayerPinned(AtomicBool::new(true)));
            app.manage(ClickState::new(Duration::from_millis(300)));

            cli::start_server(app_handle.clone());

            if sync::get_setting(app_handle, "osMediaControls", true).unwrap_or(true) {
                let _ = media_controls::init(app_handle.clone());
            }

            #[cfg(target_os = "linux")]
            linux_tray::spawn_ksni_tray(app_handle);

            #[cfg(not(target_os = "linux"))]
            let (tray_menu, show_item, show_miniplayer_item) = build_tray_menu(app_handle)?;

            #[cfg(not(target_os = "linux"))]
            {
                let show_for_events = show_item.clone();
                let show_miniplayer_for_events = show_miniplayer_item.clone();

                TrayIconBuilder::new()
                    .icon(app_handle.default_window_icon().cloned().unwrap())
                    .menu(&tray_menu)
                    .show_menu_on_left_click(false)
                    .on_tray_icon_event(|tray, event| {
                        tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);
                        let app = tray.app_handle();
                        let click_state = app.state::<ClickState>();
                        match event {
                            TrayIconEvent::Click {
                                button: MouseButton::Left,
                                button_state: MouseButtonState::Up,
                                rect,
                                ..
                            } => {
                                let app = app.clone();
                                click_state.run_single(move || {
                                    let pos = rect.position.to_physical(1.0);
                                    let size = rect.size.to_physical(1.0);
                                    toggle_miniplayer_from_tray(
                                        &app,
                                        Some(TrayAnchor::Rect {
                                            x: pos.x,
                                            y: pos.y,
                                            width: size.width,
                                            height: size.height,
                                        }),
                                    )
                                });
                            }
                            // Windows only
                            TrayIconEvent::DoubleClick { .. } => {
                                click_state.cancel_pending();
                                toggle_main_visibility(app);
                            }
                            TrayIconEvent::Click {
                                button: MouseButton::Middle,
                                button_state: MouseButtonState::Up,
                                ..
                            } => {
                                let handle = app.state::<commands::PlayerHandle>();
                                let _ = commands::send(&handle, PlayerCommand::ToggleMute);
                            }
                            _ => {}
                        }
                    })
                    .on_menu_event(move |app, event| {
                        handle_tray_menu(
                            app,
                            event,
                            show_for_events.clone(),
                            show_miniplayer_for_events.clone(),
                        );
                    })
                    .build(app_handle)?;
            }

            if let Some(mini_win) = app_handle.get_webview_window("mini-player") {
                let app_clone = app_handle.clone();

                setup_miniplayer_window(&app_clone, &mini_win);

                #[cfg(not(target_os = "linux"))]
                let show_miniplayer_label = show_miniplayer_item.clone();

                mini_win.on_window_event(move |event| match event {
                    WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        if let Some(w) = app_clone.get_webview_window("mini-player") {
                            let _ = w.hide();
                        }
                        #[cfg(not(target_os = "linux"))]
                        {
                            let _ = show_miniplayer_label.set_text("Show Miniplayer");
                        }
                    }
                    WindowEvent::Focused(false) => {
                        if let Some(state) = app_clone.try_state::<MiniPlayerPinned>() {
                            if !state.0.load(Ordering::Relaxed) {
                                if let Some(w) = app_clone.get_webview_window("mini-player") {
                                    let _ = w.hide();
                                }
                            }
                        }
                        #[cfg(not(target_os = "linux"))]
                        {
                            let still_visible = app_clone
                                .get_webview_window("mini-player")
                                .and_then(|w| w.is_visible().ok())
                                .unwrap_or(false);
                            if !still_visible {
                                let _ = show_miniplayer_label.set_text("Show Miniplayer");
                            }
                        }
                    }
                    WindowEvent::Focused(true) => {
                        #[cfg(not(target_os = "linux"))]
                        {
                            let _ = show_miniplayer_label.set_text("Hide Miniplayer");
                        }
                    }
                    _ => {}
                });
            }

            let handle = app_handle.clone();

            if let Some(main_win) = app_handle.get_webview_window("main") {
                let was_maximized = AtomicBool::new(main_win.is_maximized().unwrap_or(false));
                let win = main_win.clone();

                #[cfg(not(target_os = "linux"))]
                let show_label = show_item.clone();

                main_win.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        let keep_in_bg =
                            sync::get_setting(&handle, "keepRunningInBg", true).unwrap_or(true);
                        if keep_in_bg {
                            api.prevent_close();
                            if let Some(w) = handle.get_webview_window("main") {
                                let _ = w.hide();
                            }
                            #[cfg(not(target_os = "linux"))]
                            {
                                let _ = show_label.set_text("Show");
                            }
                        } else {
                            handle.exit(0);
                        }
                    }
                    #[cfg(not(target_os = "linux"))]
                    if let WindowEvent::Focused(false) = event {
                        let still_visible = handle
                            .get_webview_window("main")
                            .and_then(|w| w.is_visible().ok())
                            .unwrap_or(false);
                        if !still_visible {
                            let _ = show_label.set_text("Show");
                        }
                    }
                    #[cfg(not(target_os = "linux"))]
                    if let WindowEvent::Focused(true) = event {
                        let _ = show_label.set_text("Hide");
                    }
                    if let WindowEvent::Resized(_) = event {
                        let is_maximized = win.is_maximized().unwrap_or(false);

                        let prev_state = was_maximized.swap(is_maximized, Ordering::Relaxed);

                        if is_maximized != prev_state {
                            let _ = win.emit("window-maximize-changed", is_maximized);
                        }
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::add_source,
            commands::get_source_dirs,
            commands::remove_source,
            commands::refresh_watcher,
            commands::scan_library,
            commands::get_all_tracks,
            commands::get_recently_played,
            commands::get_most_played_tracks,
            commands::get_track_details,
            commands::get_track_playlist_ids,
            commands::get_artists,
            commands::get_all_albums,
            commands::get_playlists,
            commands::get_tracks_by_playlist,
            commands::get_tracks_by_album,
            commands::get_tracks_by_artist,
            commands::get_favorite_tracks,
            commands::create_playlist,
            commands::add_track_to_playlist,
            commands::remove_track_from_playlist,
            commands::delete_playlist,
            commands::get_playlist,
            commands::toggle_favorite,
            commands::play_context,
            commands::play_pause,
            commands::next,
            commands::previous,
            commands::seek,
            commands::set_volume,
            commands::set_repeat,
            commands::toggle_shuffle,
            commands::toggle_mute,
            commands::enqueue_next,
            commands::enqueue_end,
            commands::enqueue_end_many,
            commands::remove_from_queue,
            commands::clear_queue,
            commands::reorder_queue,
            commands::reorder_context,
            commands::set_autoplay,
            commands::play_track_from_context,
            commands::restore_session,
            commands::get_current_state,
            commands::close_player,
            commands::get_top_artists,
            commands::get_top_albums,
            commands::get_forgotten_tracks,
            commands::get_unplayed_tracks,
            commands::get_recently_added,
            commands::save_image,
            commands::fetch_artist_image,
            commands::update_artist,
            commands::update_album,
            commands::update_playlist,
            commands::get_stats_overview,
            commands::get_top_tracks_with_stats,
            commands::get_top_artists_with_stats,
            commands::get_top_albums_with_stats,
            commands::get_top_genres_with_stats,
            commands::get_listening_time_trend,
            commands::get_streak_data,
            commands::get_library_growth,
            commands::get_format_distribution,
            commands::get_data_age,
            commands::get_heatmap_hourly,
            commands::get_heatmap_weekday,
            commands::get_favorite_trends,
            commands::get_playback_history_timeline,
            commands::toggle_mini_player_pin,
            commands::quit_app,
            commands::toggle_mini_player,
            commands::set_os_media_controls,
            commands::get_startup_status,
            commands::get_update_install_support,
            commands::reset_app_data,
            commands::update_track_metadata,
            commands::get_track_lyrics,
            commands::update_track_lyrics,
            commands::fetch_lyrics_from_lrclib,
            commands::get_genres,
            commands::get_tracks_by_genre,
            commands::create_genre,
            commands::update_genre,
            commands::delete_genre,
            commands::set_track_cover_art,
            commands::set_track_artists,
            commands::set_track_album,
            commands::set_track_genre,
            commands::delete_track,
            commands::get_scan_blacklist,
            commands::unblacklist_path,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    // Event loop: handle file opens (macOS application:openFiles:, etc.)
    app.run(move |_handle, _event| {
        #[cfg(any(target_os = "macos", target_os = "ios", target_os = "android"))]
        if let tauri::RunEvent::Opened { urls } = _event {
            let paths: Vec<String> = urls
                .iter()
                .filter_map(|u| {
                    if u.scheme() == "file" {
                        u.to_file_path()
                            .ok()
                            .map(|p| p.to_string_lossy().into_owned())
                    } else {
                        None
                    }
                })
                .collect();
            if !paths.is_empty() {
                if let Err(e) = cli::play_paths(_handle, &paths) {
                    tracing::error!(error = %e, "file association failed");
                }
            }
        }
    });

    cli::cleanup_server();
}
