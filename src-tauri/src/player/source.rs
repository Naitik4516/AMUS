use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "id")]
pub enum PlaybackSource {
    Album(i64),
    Playlist(i64),
    Artist(i64),
    Favorites,
    Queue, 
    Other,
}

impl PlaybackSource {
    pub fn type_str(&self) -> &'static str {
        match self {
            PlaybackSource::Album(_) => "ALBUM",
            PlaybackSource::Playlist(_) => "PLAYLIST",
            PlaybackSource::Artist(_) => "ARTIST",
            PlaybackSource::Favorites => "FAVORITES",
            PlaybackSource::Queue | PlaybackSource::Other => "OTHER",
        }
    }

    pub fn source_id(&self) -> Option<i64> {
        match self {
            PlaybackSource::Album(id) | PlaybackSource::Playlist(id) | PlaybackSource::Artist(id) => Some(*id),
            _ => None,
        }
    }

    pub fn from_db(type_str: &str, id: Option<i64>) -> Self {
        match type_str {
            "ALBUM" => PlaybackSource::Album(id.unwrap_or_default()),
            "PLAYLIST" => PlaybackSource::Playlist(id.unwrap_or_default()),
            "ARTIST" => PlaybackSource::Artist(id.unwrap_or_default()),
            "FAVORITES" => PlaybackSource::Favorites,
            _ => PlaybackSource::Other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RepeatMode {
    Off,
    All,
    One,
}

impl RepeatMode {
    pub fn cycle(self) -> Self {
        match self {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            RepeatMode::Off => "OFF",
            RepeatMode::All => "ALL",
            RepeatMode::One => "ONE",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "ALL" => RepeatMode::All,
            "ONE" => RepeatMode::One,
            _ => RepeatMode::Off,
        }
    }
}