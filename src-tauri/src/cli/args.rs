use clap::{Parser, Subcommand};
use std::path::PathBuf;

use super::protocol::SearchKind;

#[derive(Debug, Parser)]
#[command(
    name = "amus",
    version,
    about = "AMUS — control the AMUS music player from the terminal"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(trailing_var_arg = true, allow_hyphen_values = false)]
    pub paths: Vec<String>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Play {
        /// Play top search result instead of resuming
        #[arg(short = 's', long = "search")]
        search: Option<String>,
        /// Files/folders to play (alternative to -s)
        #[arg(trailing_var_arg = true)]
        paths: Vec<String>,
    },
    Pause,
    Stop,
    Toggle,
    Next,
    Prev,
    Seek {
        position: String,
    },
    Volume {
        level: String,
    },
    Mute,
    Status,
    Queue {
        #[command(subcommand)]
        action: QueueCmd,
    },
    Library {
        #[command(subcommand)]
        action: LibraryCmd,
    },
    Search {
        /// Optional scope: artist | album (default: all)
        #[arg(value_parser = parse_search_scope)]
        scope_or_query: String,
        /// Query when scope is provided as first arg
        query: Option<String>,
    },

    Playlist {
        #[command(subcommand)] // Show playlist by name/id when no subcommand
        action: Option<PlaylistCmd>,

        #[arg(trailing_var_arg = true)]
        name: Vec<String>,
    },
    Playlists,
    Albums,
    Artists,
    Album {
        id_or_name: String,
    },
    Artist {
        id_or_name: String,
    },
    Open,
    Hide,
    Show,
    Close,
    Import {
        path: PathBuf,
    },
    Info {
        path: PathBuf,
    },
    Update,
    Version,
    Reset {
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
    Clear,
    Shuffle,
    Show,
}

#[derive(Debug, Subcommand)]
pub enum LibraryCmd {
    Rescan,
}

#[derive(Debug, Subcommand)]
pub enum PlaylistCmd {
    Create { name: String },
    Add { playlist: String, path: PathBuf },
    Remove { playlist: String, path: PathBuf },
    Play { playlist: String },
    Delete { playlist: String },
}

fn parse_search_scope(s: &str) -> Result<String, String> {
    Ok(s.to_string())
}

pub fn parse_signed_number(s: &str) -> Result<(f64, bool), String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty number".into());
    }
    let relative = s.starts_with('+') || s.starts_with('-');
    // For relative, keep the sign as part of the value
    let value: f64 = if s.starts_with('+') {
        s[1..].parse().map_err(|_| format!("invalid number: {s}"))?
    } else {
        s.parse().map_err(|_| format!("invalid number: {s}"))?
    };
    let value = if s.starts_with('+') {
        value
    } else if relative && s.starts_with('-') {
        value
    } else {
        value
    };
    Ok((value, relative))
}

pub fn parse_search_query(raw: &str) -> (String, SearchKind) {
    let raw = raw.trim();
    if let Some(rest) = raw.strip_prefix("artist:") {
        return (
            rest.trim().trim_matches('"').to_string(),
            SearchKind::Artist,
        );
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
