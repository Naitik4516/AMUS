//! CLI IPC protocol (length-prefixed JSON frames).

use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliRequest {
    pub id: u64,
    pub cmd: CliCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum CliCommand {
    // Playback
    Play,
    Pause,
    Stop,
    Toggle,
    Next,
    Previous,
    Seek {
        /// Absolute position in seconds when `relative` is false.
        value: f64,
        relative: bool,
    },
    Volume {
        /// Absolute 0–100 when `relative` is false; delta percent when true.
        value: f64,
        relative: bool,
    },
    Mute,
    Status,
    QueueShow,
    QueueClear,
    QueueShuffle,
    QueueAddPaths {
        paths: Vec<String>,
    },
    QueueAddSearch {
        query: String,
        kind: SearchKind,
    },
    PlayPaths {
        paths: Vec<String>,
    },
    PlaySearch {
        query: String,
        kind: SearchKind,
    },
    LibraryRescan,
    Import {
        path: String,
    },
    Search {
        query: String,
        kind: SearchKind,
    },
    // Playlists
    PlaylistCreate {
        name: String,
    },
    PlaylistAdd {
        playlist: String,
        path: String,
    },
    PlaylistRemove {
        playlist: String,
        path: String,
    },
    PlaylistPlay {
        playlist: String,
    },
    PlaylistDelete {
        playlist: String,
    },
    PlaylistShow {
        playlist: Option<String>,
    },
    // Browse
    ListAlbums,
    ListArtists,
    ListPlaylists,
    ShowAlbum {
        id_or_name: String,
    },
    ShowArtist {
        id_or_name: String,
    },
    // Window
    Open,
    Hide,
    Show,
    Close,
    // Meta
    Update,
    Version,
    /// Reset all app data and restart
    Reset {
        force: bool,
    },
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SearchKind {
    #[default]
    All,
    Track,
    Artist,
    Album,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliResponse {
    pub id: u64,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<CliData>,
}

impl CliResponse {
    pub fn ok(id: u64, data: CliData) -> Self {
        Self {
            id,
            ok: true,
            error: None,
            data: Some(data),
        }
    }

    pub fn ok_empty(id: u64) -> Self {
        Self {
            id,
            ok: true,
            error: None,
            data: Some(CliData::Message {
                text: String::new(),
            }),
        }
    }

    pub fn err(id: u64, error: impl Into<String>) -> Self {
        Self {
            id,
            ok: false,
            error: Some(error.into()),
            data: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum CliData {
    Message {
        text: String,
    },
    Status {
        is_playing: bool,
        has_track: bool,
        title: Option<String>,
        artist: Option<String>,
        album: Option<String>,
        position_sec: f64,
        duration_sec: u32,
        volume_percent: u32,
        muted: bool,
        shuffle: bool,
        repeat: String,
    },
    Queue {
        tracks: Vec<QueueTrackLine>,
    },
    SearchResults {
        tracks: Vec<SearchTrackLine>,
        artists: Vec<SearchNamedLine>,
        albums: Vec<SearchNamedLine>,
    },
    NamedList {
        items: Vec<SearchNamedLine>,
    },
    TrackList {
        label: String,
        tracks: Vec<QueueTrackLine>,
    },
    Version {
        version: String,
    },
    UpdateResult {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueTrackLine {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_sec: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchTrackLine {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub album: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchNamedLine {
    pub id: i64,
    pub name: String,
}

/// Read a length-prefixed JSON frame (u32 LE length + payload).
pub fn read_frame<R: std::io::Read>(
    reader: &mut R,
) -> std::io::Result<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf) as usize;
    if len > 16 * 1024 * 1024 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "frame too large",
        ));
    }
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

/// Write a length-prefixed JSON frame.
pub fn write_frame<W: std::io::Write>(writer: &mut W, data: &[u8]) -> std::io::Result<()> {
    let len = data.len() as u32;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(data)?;
    writer.flush()?;
    Ok(())
}
