//! CLI IPC server running inside the GUI process.

use std::io::BufReader;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use tauri::{AppHandle, Listener};

use super::dispatch;
use super::paths::{app_data_dir, socket_path};
use super::protocol::{read_frame, write_frame, CliRequest};

static SERVER_RUNNING: AtomicBool = AtomicBool::new(false);

pub fn start(app: AppHandle) {
    if SERVER_RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }

    let _ = std::fs::create_dir_all(app_data_dir());

    #[cfg(unix)]
    {
        start_unix(app);
    }
    #[cfg(not(unix))]
    {
        start_tcp(app);
    }
}

#[cfg(unix)]
fn start_unix(app: AppHandle) {
    use std::os::unix::net::UnixListener;

    let path = socket_path();
    let _ = std::fs::remove_file(&path);

    let listener = match UnixListener::bind(&path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("cli server: failed to bind {}: {e}", path.display());
            SERVER_RUNNING.store(false, Ordering::SeqCst);
            return;
        }
    };

    // Clean up socket on drop of process — also register a best-effort remove.
    let path_cleanup = path.clone();
    let app_for_exit = app.clone();
    app_for_exit.once("cli-server-stop", move |_| {
        let _ = std::fs::remove_file(&path_cleanup);
    });

    thread::Builder::new()
        .name("cli-server".into())
        .spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let app = app.clone();
                        thread::spawn(move || handle_client(app, stream));
                    }
                    Err(e) => {
                        eprintln!("cli server accept error: {e}");
                    }
                }
            }
            SERVER_RUNNING.store(false, Ordering::SeqCst);
        })
        .expect("failed to spawn cli-server");
}

#[cfg(not(unix))]
fn start_tcp(app: AppHandle) {
    use std::net::TcpListener;

    let listener = match TcpListener::bind("127.0.0.1:0") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("cli server: failed to bind tcp: {e}");
            SERVER_RUNNING.store(false, Ordering::SeqCst);
            return;
        }
    };
    let port = listener.local_addr().map(|a| a.port()).unwrap_or(0);
    let port_file = socket_path();
    if let Err(e) = std::fs::write(&port_file, port.to_string()) {
        eprintln!("cli server: failed to write port file: {e}");
    }

    thread::Builder::new()
        .name("cli-server".into())
        .spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let app = app.clone();
                        thread::spawn(move || handle_client(app, stream));
                    }
                    Err(e) => eprintln!("cli server accept error: {e}"),
                }
            }
            SERVER_RUNNING.store(false, Ordering::SeqCst);
        })
        .expect("failed to spawn cli-server");
}

fn handle_client<S>(app: AppHandle, stream: S)
where
    S: std::io::Read + std::io::Write + Send,
{
    let mut reader = BufReader::new(stream);
    // We need Write on the same stream — re-split by taking ownership carefully.
    // BufReader only reads; for simplicity re-open pattern: use a duplex by cloning on unix.
    // Instead, read fully then write on the underlying stream via get_mut.
    loop {
        let frame = match read_frame(&mut reader) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => {
                eprintln!("cli server read error: {e}");
                break;
            }
        };

        let response = match serde_json::from_slice::<CliRequest>(&frame) {
            Ok(req) => dispatch::handle(&app, req.cmd, req.id),
            Err(e) => super::protocol::CliResponse::err(0, format!("bad request: {e}")),
        };

        let body = match serde_json::to_vec(&response) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("cli server serialize error: {e}");
                break;
            }
        };

        let stream = reader.get_mut();
        if let Err(e) = write_frame(stream, &body) {
            eprintln!("cli server write error: {e}");
            break;
        }
    }
}

/// Remove stale socket/port file (call on graceful shutdown if needed).
pub fn cleanup() {
    let path = socket_path();
    let _ = std::fs::remove_file(path);
    SERVER_RUNNING.store(false, Ordering::SeqCst);
}

// Silence unused import on non-unix for Arc if needed
#[allow(dead_code)]
fn _arc_marker() -> Arc<()> {
    Arc::new(())
}
