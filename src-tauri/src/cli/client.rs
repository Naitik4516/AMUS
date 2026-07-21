//! CLI client: connect to running instance, auto-start if needed.

use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use super::paths::{app_data_dir, socket_path};
use super::protocol::{CliRequest, CliResponse, read_frame, write_frame};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
const POLL_INTERVAL: Duration = Duration::from_millis(150);

#[cfg(unix)]
type Stream = std::os::unix::net::UnixStream;

#[cfg(not(unix))]
type Stream = std::net::TcpStream;

pub fn send_command(cmd: super::protocol::CliCommand) -> Result<CliResponse, String> {
    let mut stream = connect_or_start()?;
    let req = CliRequest { id: 1, cmd };
    let body = serde_json::to_vec(&req).map_err(|e| e.to_string())?;
    write_frame(&mut stream, &body).map_err(|e| format!("write failed: {e}"))?;
    let resp_bytes = read_frame(&mut stream).map_err(|e| format!("read failed: {e}"))?;
    serde_json::from_slice(&resp_bytes).map_err(|e| format!("invalid response: {e}"))
}

fn connect_or_start() -> Result<Stream, String> {
    if let Ok(s) = try_connect() {
        return Ok(s);
    }
    start_server()?;
    let deadline = Instant::now() + CONNECT_TIMEOUT;
    while Instant::now() < deadline {
        if let Ok(s) = try_connect() {
            return Ok(s);
        }
        thread::sleep(POLL_INTERVAL);
    }
    Err("timed out waiting for AMUS to start".into())
}

#[cfg(unix)]
fn try_connect() -> Result<Stream, String> {
    let path = socket_path();
    Stream::connect(&path).map_err(|e| e.to_string())
}

#[cfg(not(unix))]
fn try_connect() -> Result<Stream, String> {
    let port_file = socket_path();
    let port_str = std::fs::read_to_string(&port_file).map_err(|e| e.to_string())?;
    let port: u16 = port_str
        .trim()
        .parse()
        .map_err(|e| format!("bad port file: {e}"))?;
    Stream::connect(("127.0.0.1", port)).map_err(|e| e.to_string())
}

fn start_server() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    // Ensure app data dir exists so server can bind.
    let _ = std::fs::create_dir_all(app_data_dir());

    let mut cmd = Command::new(exe);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // Detach so CLI can exit independently.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // SAFETY: we only call this before spawn; no other threads share state we care about.
        unsafe {
            cmd.pre_exec(|| {
                // New session so we don't die with the terminal group necessarily —
                // still fine for desktop app launched from CLI.
                libc_setsid();
                Ok(())
            });
        }
    }

    cmd.spawn()
        .map_err(|e| format!("failed to start AMUS: {e}"))?;
    Ok(())
}

#[cfg(unix)]
fn libc_setsid() {
    // Avoid extra libc dep: syscall via nix not available; use libc if present.
    // Best-effort — if setsid fails, spawn still works.
    unsafe {
        // libc is a transitive dep of many crates; use raw if not linked.
        #[allow(non_camel_case_types)]
        unsafe extern "C" {
            fn setsid() -> i32;
        }
        let _ = setsid();
    }
}

/// Local file info without contacting the server.
pub fn local_info(path: &Path) -> Result<String, String> {
    use lofty::prelude::*;
    use lofty::probe::Probe;

    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    if !path.exists() {
        return Err(format!("file not found: {}", path.display()));
    }

    let tagged = Probe::open(&path)
        .map_err(|e| e.to_string())?
        .read()
        .map_err(|e| e.to_string())?;
    let props = tagged.properties();
    let duration = props.duration().as_secs() as u32;
    let bitrate = props.audio_bitrate();
    let sample_rate = props.sample_rate();

    let tag = tagged.primary_tag().or_else(|| tagged.first_tag());
    let (title, artist, album, genre) = if let Some(t) = tag {
        (
            t.title()
                .map(|s| s.into_owned())
                .unwrap_or_else(|| file_stem(&path)),
            t.artist()
                .map(|s| s.into_owned())
                .unwrap_or_else(|| "Unknown Artist".into()),
            t.album()
                .map(|s| s.into_owned())
                .unwrap_or_else(|| "Unknown Album".into()),
            t.genre().map(|s| s.into_owned()),
        )
    } else {
        (
            file_stem(&path),
            "Unknown Artist".into(),
            "Unknown Album".into(),
            None,
        )
    };

    Ok(super::format::format_file_info(
        &title,
        &artist,
        &album,
        genre.as_deref(),
        bitrate,
        sample_rate,
        duration,
    ))
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string()
}
