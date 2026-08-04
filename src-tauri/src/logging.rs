#[cfg(debug_assertions)]
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing_appender::non_blocking::WorkerGuard;
#[cfg(debug_assertions)]
use tracing_appender::non_blocking::NonBlocking;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const LOG_KEEP_DAYS: u64 = 7;
const APP_DATA_DIR_NAME: &str = "AMUS";

fn cleanup_old_logs(logs_dir: &Path) {
    let cutoff = match std::time::SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(LOG_KEEP_DAYS * 24 * 60 * 60))
    {
        Some(c) => c,
        None => return,
    };

    if let Ok(entries) = std::fs::read_dir(logs_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("amus.log") {
                continue;
            }
            let expired = entry
                .metadata()
                .and_then(|meta| meta.modified())
                .map(|modified| modified < cutoff)
                .unwrap_or(false);
            if expired {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
}

fn env_override() -> Option<EnvFilter> {
    std::env::var("AMUS_LOG")
        .ok()
        .and_then(|v| EnvFilter::try_new(v).ok())
        .or_else(|| EnvFilter::try_from_default_env().ok())
}

/// Filter for the rotating log file. Defaults to `info` for every target so
/// third-party crate debug noise (html5ever, rustls, hyper, ...) doesn't
/// flood it; `AMUS_LOG`/`RUST_LOG` overrides.
fn file_filter() -> EnvFilter {
    env_override().unwrap_or_else(|| EnvFilter::new("info"))
}

/// Filter for the debug console. Defaults to `info` for all targets plus
/// `debug` for our own crate, keeping dev output readable while still
/// showing AMUS debug logs; `AMUS_LOG`/`RUST_LOG` overrides.
#[cfg(debug_assertions)]
fn console_filter() -> EnvFilter {
    env_override().unwrap_or_else(|| EnvFilter::new("info,amus_lib=debug"))
}

/// Resolves the app data directory without a Tauri handle, mirroring
/// `app.path().app_data_dir()` (identifier `AMUS`). Needed to initialize
/// logging before the Tauri builder is constructed.
pub fn early_app_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join(APP_DATA_DIR_NAME)
}

/// Installs the global tracing subscriber: daily-rotated file in
/// `app_dir/logs/amus.log`, plus a stdout mirror in debug builds.
/// The filter can be overridden via `AMUS_LOG` or `RUST_LOG`.
///
/// If a global subscriber already exists (the devtools plugin claims it in
/// debug builds), this is a no-op — file logging is then handled by
/// [`build_file_adapter`] through the devtools bridge.
///
/// Returns a `WorkerGuard` that must be kept alive for the app's lifetime
/// (store it in Tauri state).
pub fn init(app_dir: &Path) -> WorkerGuard {
    let logs_dir = app_dir.join("logs");
    let _ = std::fs::create_dir_all(&logs_dir);

    cleanup_old_logs(&logs_dir);

    let file_appender = tracing_appender::rolling::daily(&logs_dir, "amus.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_target(false)
        .with_writer(file_writer)
        .with_filter(file_filter());

    let subscriber = tracing_subscriber::registry().with(file_layer);

    #[cfg(debug_assertions)]
    let subscriber = subscriber.with(
        tracing_subscriber::fmt::layer()
            .with_target(false)
            .with_writer(std::io::stdout)
            .with_filter(console_filter()),
    );

    // Ignore failure: the devtools plugin already owns the global default.
    let _ = subscriber.try_init();

    guard
}

/// A `log::Log` adapter that writes to the rotating log file (and stdout in
/// debug builds). Attached to the devtools plugin via `attach_logger`, which
/// forwards every traced event with a message to it — this is how file
/// logging survives when the devtools plugin holds the global subscriber.
#[cfg(debug_assertions)]
pub struct FileLogAdapter {
    writer: std::sync::Mutex<NonBlocking>,
    file_level: log::LevelFilter,
    stdout_level: log::LevelFilter,
    /// When set, DEBUG-level console output is limited to our own crate
    /// (targets starting with `amus`), keeping third-party debug spam
    /// (html5ever, rustls, ...) off the terminal.
    stdout_amus_only: bool,
}

#[cfg(debug_assertions)]
impl log::Log for FileLogAdapter {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.file_level || metadata.level() <= self.stdout_level
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let level = record.metadata().level();
        let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
        let line = format!(
            "{} {:>5} {}: {}\n",
            ts,
            level,
            record.target(),
            record.args()
        );
        if level <= self.file_level {
            let mut writer = self.writer.lock().unwrap_or_else(|e| e.into_inner());
            let _ = writer.write_all(line.as_bytes());
        }
        if level <= self.stdout_level {
            let show = !self.stdout_amus_only
                || level <= log::LevelFilter::Info
                || record.target().starts_with("amus");
            if show {
                let mut out = std::io::stdout().lock();
                let _ = out.write_all(line.as_bytes());
            }
        }
    }

    fn flush(&self) {
        if let Ok(mut writer) = self.writer.lock() {
            let _ = writer.flush();
        }
        let _ = std::io::stdout().lock().flush();
    }
}

#[cfg(debug_assertions)]
pub fn build_file_adapter(app_dir: &Path) -> Box<dyn log::Log> {
    let logs_dir = app_dir.join("logs");
    let _ = std::fs::create_dir_all(&logs_dir);

    cleanup_old_logs(&logs_dir);

    let file_appender = tracing_appender::rolling::daily(&logs_dir, "amus.log");
    let (writer, guard) = tracing_appender::non_blocking(file_appender);
    // The adapter lives inside the devtools plugin for the app's lifetime;
    // leak the guard so the writer thread is not shut down early.
    std::mem::forget(guard);

    let (file_level, stdout_level, stdout_amus_only) = adapter_levels();
    Box::new(FileLogAdapter {
        writer: std::sync::Mutex::new(writer),
        file_level,
        stdout_level,
        stdout_amus_only,
    })
}

/// The log file defaults to `info` so third-party crate noise (rustls, hyper,
/// primp, html5ever, ...) doesn't flood it; the debug console mirrors our
/// crate at `debug` and everything else at `info`. `AMUS_LOG`/`RUST_LOG`
/// overrides both and disables the console target restriction.
#[cfg(debug_assertions)]
fn adapter_levels() -> (log::LevelFilter, log::LevelFilter, bool) {
    match std::env::var("AMUS_LOG").or_else(|_| std::env::var("RUST_LOG")) {
        Ok(level) => {
            let filter = parse_level(&level);
            (filter, filter, false)
        }
        Err(_) => (log::LevelFilter::Info, log::LevelFilter::Debug, true),
    }
}

#[cfg(debug_assertions)]
fn parse_level(level: &str) -> log::LevelFilter {
    match level.to_lowercase().as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        "off" => log::LevelFilter::Off,
        _ => log::LevelFilter::Info,
    }
}
