use std::path::{Path, PathBuf};

const APP_NAME: &str = "AMUS";
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "wav", "ogg", "m4a", "aac", "opus"];

pub fn app_data_dir() -> PathBuf {
    dirs::data_dir()
        .map(|d| d.join(format!("{}", APP_NAME)))
        .unwrap_or_else(|| PathBuf::from("."))
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

pub fn absolutize(path: &str, cwd: &Path) -> PathBuf {
    let p = PathBuf::from(path);
    if p.is_absolute() { p } else { cwd.join(p) }
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
