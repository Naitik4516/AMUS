//! Clap CLI definition for AMUS.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use super::protocol::SearchKind;

#[derive(Debug, Parser)]
#[command(
    name = "amus",
    version,
    about = "AMUS — control the AMUS music player from the terminal",
    long_about = "Send commands to a running AMUS instance. If none is running, one is started automatically."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Audio files, folders, or globs to play (when no subcommand is given).
    #[arg(trailing_var_arg = true, allow_hyphen_values = false)]
    pub paths: Vec<String>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Resume playback
    Play {
        /// Play top search result instead of resuming
        #[arg(short = 's', long = "search")]
        search: Option<String>,
        /// Files/folders to play (alternative to -s)
        #[arg(trailing_var_arg = true)]
        paths: Vec<String>,
    },
    /// Pause playback
    Pause,
    /// Stop playback
    Stop,
    /// Toggle play/pause
    Toggle,
    /// Next track
    Next,
    /// Previous track
    Prev,
    /// Seek to position or by offset (e.g. +10, -5, 90)
    Seek {
        position: String,
    },
    /// Set or adjust volume (e.g. 80, +5, -10)
    Volume {
        level: String,
    },
    /// Mute / unmute
    Mute,
    /// Show player status
    Status,
    /// Queue operations
    Queue {
        #[command(subcommand)]
        action: QueueCmd,
    },
    /// Library operations
    Library {
        #[command(subcommand)]
        action: LibraryCmd,
    },
    /// Search the library
    Search {
        /// Optional scope: artist | album (default: all)
        #[arg(value_parser = parse_search_scope)]
        scope_or_query: String,
        /// Query when scope is provided as first arg
        query: Option<String>,
    },
    /// Playlist operations
    Playlist {
        #[command(subcommand)]
        action: Option<PlaylistCmd>,
        /// Show playlist by name/id when no subcommand
        #[arg(trailing_var_arg = true)]
        name: Vec<String>,
    },
    /// List all playlists
    Playlists,
    /// List all albums
    Albums,
    /// List all artists
    Artists,
    /// Show album by name or id
    Album {
        id_or_name: String,
    },
    /// Show artist by name or id
    Artist {
        id_or_name: String,
    },
    /// Open / show the main window
    Open,
    /// Hide the main window
    Hide,
    /// Show the main window
    Show,
    /// Close the main window (or quit depending on settings)
    Close,
    /// Import a folder as a library source and scan
    Import {
        path: PathBuf,
    },
    /// Print metadata for an audio file
    Info {
        path: PathBuf,
    },
    /// Check for and install updates
    Update,
    /// Print version
    Version,
    /// Reset all app data and restart
    Reset {
        /// Skip confirmation prompt
        #[arg(long, short)]
        force: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum QueueCmd {
    /// Add files/folders or search results to the queue
    Add {
        #[arg(short = 's', long = "search")]
        search: Option<String>,
        #[arg(trailing_var_arg = true)]
        paths: Vec<String>,
    },
    /// Clear the user queue
    Clear,
    /// Toggle shuffle
    Shuffle,
    /// Show the queue
    Show,
}

#[derive(Debug, Subcommand)]
pub enum LibraryCmd {
    /// Rescan all library sources
    Rescan,
}

#[derive(Debug, Subcommand)]
pub enum PlaylistCmd {
    Create {
        name: String,
    },
    Add {
        playlist: String,
        path: PathBuf,
    },
    Remove {
        playlist: String,
        path: PathBuf,
    },
    Play {
        playlist: String,
    },
    Delete {
        playlist: String,
    },
}

fn parse_search_scope(s: &str) -> Result<String, String> {
    Ok(s.to_string())
}

/// Parse seek string: +10, -5, or absolute 30.
pub fn parse_signed_number(s: &str) -> Result<(f64, bool), String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty number".into());
    }
    let relative = s.starts_with('+') || s.starts_with('-');
    // For relative, keep the sign as part of the value
    let value: f64 = if s.starts_with('+') {
        s[1..]
            .parse()
            .map_err(|_| format!("invalid number: {s}"))?
    } else {
        s.parse().map_err(|_| format!("invalid number: {s}"))?
    };
    // If starts with +, value is positive; if starts with - and we used full parse, value is negative
    let value = if s.starts_with('+') {
        value
    } else if relative && s.starts_with('-') {
        // already negative from parse
        value
    } else {
        value
    };
    Ok((value, relative))
}

/// Parse search kind from query prefixes like `artist:Foo` or bare text.
pub fn parse_search_query(raw: &str) -> (String, SearchKind) {
    let raw = raw.trim();
    if let Some(rest) = raw.strip_prefix("artist:") {
        return (rest.trim().trim_matches('"').to_string(), SearchKind::Artist);
    }
    if let Some(rest) = raw.strip_prefix("album:") {
        return (rest.trim().trim_matches('"').to_string(), SearchKind::Album);
    }
    if let Some(rest) = raw.strip_prefix("track:") {
        return (rest.trim().trim_matches('"').to_string(), SearchKind::Track);
    }
    (raw.trim_matches('"').to_string(), SearchKind::Track)
}

pub fn search_kind_from_scope(scope: &str) -> Option<SearchKind> {
    match scope.to_ascii_lowercase().as_str() {
        "artist" | "artists" => Some(SearchKind::Artist),
        "album" | "albums" => Some(SearchKind::Album),
        "track" | "tracks" | "song" | "songs" => Some(SearchKind::Track),
        _ => None,
    }
}
