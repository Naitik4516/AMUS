//! Path resolution helpers for the CLI.

use std::path::{Path, PathBuf};

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "wav", "ogg", "m4a", "aac", "opus"];

/// Resolve app data directory to match Tauri's `app_data_dir` for identifier `AMUS`.
pub fn app_data_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
            return PathBuf::from(xdg).join("AMUS");
        }
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        return PathBuf::from(home).join(".local/share/AMUS");
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        return PathBuf::from(home).join("Library/Application Support/AMUS");
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata).join("AMUS");
        }
        return PathBuf::from(r"C:\AMUS");
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        PathBuf::from(".").join("AMUS")
    }
}

pub fn socket_path() -> PathBuf {
    #[cfg(unix)]
    {
        app_data_dir().join("cli.sock")
    }
    #[cfg(not(unix))]
    {
        app_data_dir().join("cli.port")
    }
}

/// Absolute path for a user-supplied path relative to `cwd`.
pub fn absolutize(path: &str, cwd: &Path) -> PathBuf {
    let p = PathBuf::from(path);
    if p.is_absolute() {
        p
    } else {
        cwd.join(p)
    }
}

pub fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| AUDIO_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

/// Expand paths/globs/directories into a list of absolute audio file paths.
pub fn collect_audio_paths(inputs: &[String], cwd: &Path) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    for input in inputs {
        let expanded = expand_one(input, cwd)?;
        out.extend(expanded);
    }
    // de-dupe while preserving order
    let mut seen = std::collections::HashSet::new();
    out.retain(|p| seen.insert(p.clone()));
    if out.is_empty() {
        return Err("no audio files found".into());
    }
    Ok(out)
}

fn expand_one(input: &str, cwd: &Path) -> Result<Vec<PathBuf>, String> {
    // Shell may already expand globs; also handle unexpanded patterns.
    if input.contains('*') || input.contains('?') || input.contains('[') {
        let pattern = if Path::new(input).is_absolute() {
            input.to_string()
        } else {
            cwd.join(input).to_string_lossy().into_owned()
        };
        let mut files = Vec::new();
        for entry in glob::glob(&pattern).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?;
            if path.is_dir() {
                files.extend(walk_dir(&path));
            } else if is_audio_file(&path) {
                files.push(canonicalize_soft(&path));
            }
        }
        return Ok(files);
    }

    let path = absolutize(input, cwd);
    if !path.exists() {
        return Err(format!("path not found: {}", path.display()));
    }
    if path.is_dir() {
        return Ok(walk_dir(&path));
    }
    if is_audio_file(&path) {
        return Ok(vec![canonicalize_soft(&path)]);
    }
    Err(format!(
        "not an audio file: {} (supported: {})",
        path.display(),
        AUDIO_EXTENSIONS.join(", ")
    ))
}

fn walk_dir(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && is_audio_file(path) {
            files.push(canonicalize_soft(path));
        }
    }
    files.sort();
    files
}

fn canonicalize_soft(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}
