use serde::Serialize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repeat_mode_cycle() {
        assert_eq!(RepeatMode::Off.cycle(), RepeatMode::All);
        assert_eq!(RepeatMode::All.cycle(), RepeatMode::One);
        assert_eq!(RepeatMode::One.cycle(), RepeatMode::Off);
    }

    #[test]
    fn test_repeat_mode_as_str() {
        assert_eq!(RepeatMode::Off.as_str(), "OFF");
        assert_eq!(RepeatMode::All.as_str(), "ALL");
        assert_eq!(RepeatMode::One.as_str(), "ONE");
    }

    #[test]
    fn test_repeat_mode_from_str() {
        assert_eq!(RepeatMode::from_str("ALL"), RepeatMode::All);
        assert_eq!(RepeatMode::from_str("ONE"), RepeatMode::One);
        assert_eq!(RepeatMode::from_str("OFF"), RepeatMode::Off);
        assert_eq!(RepeatMode::from_str("invalid"), RepeatMode::Off);
        assert_eq!(RepeatMode::from_str(""), RepeatMode::Off);
    }

    #[test]
    fn test_playback_source_type_str() {
        assert_eq!(PlaybackSource::Album(1).type_str(), "ALBUM");
        assert_eq!(PlaybackSource::Playlist(2).type_str(), "PLAYLIST");
        assert_eq!(PlaybackSource::Artist(3).type_str(), "ARTIST");
        assert_eq!(PlaybackSource::Favorites.type_str(), "FAVORITES");
        assert_eq!(PlaybackSource::Queue.type_str(), "QUEUE");
        assert_eq!(PlaybackSource::Direct.type_str(), "DIRECT");
        assert_eq!(PlaybackSource::Other.type_str(), "OTHER");
    }

    #[test]
    fn test_playback_source_source_id() {
        assert_eq!(PlaybackSource::Album(42).source_id(), Some(42));
        assert_eq!(PlaybackSource::Playlist(7).source_id(), Some(7));
        assert_eq!(PlaybackSource::Artist(99).source_id(), Some(99));
        assert_eq!(PlaybackSource::Favorites.source_id(), None);
        assert_eq!(PlaybackSource::Queue.source_id(), None);
        assert_eq!(PlaybackSource::Direct.source_id(), None);
        assert_eq!(PlaybackSource::Other.source_id(), None);
    }

    #[test]
    fn test_playback_source_from_db() {
        assert_eq!(PlaybackSource::from_db("ALBUM", Some(1)), PlaybackSource::Album(1));
        assert_eq!(PlaybackSource::from_db("ALBUM", None), PlaybackSource::Album(0));
        assert_eq!(PlaybackSource::from_db("PLAYLIST", Some(5)), PlaybackSource::Playlist(5));
        assert_eq!(PlaybackSource::from_db("ARTIST", Some(3)), PlaybackSource::Artist(3));
        assert_eq!(PlaybackSource::from_db("FAVORITES", None), PlaybackSource::Favorites);
        assert_eq!(PlaybackSource::from_db("DIRECT", None), PlaybackSource::Direct);
        assert_eq!(PlaybackSource::from_db("QUEUE", None), PlaybackSource::Queue);
        assert_eq!(PlaybackSource::from_db("SEARCH", None), PlaybackSource::Other);
        assert_eq!(PlaybackSource::from_db("UNKNOWN", None), PlaybackSource::Other);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "id")]
pub enum PlaybackSource {
    Album(i64),
    Playlist(i64),
    Artist(i64),
    Favorites,
    Queue,
    Direct,
    Other,
}

impl PlaybackSource {
    pub fn type_str(&self) -> &'static str {
        match self {
            PlaybackSource::Album(_) => "ALBUM",
            PlaybackSource::Playlist(_) => "PLAYLIST",
            PlaybackSource::Artist(_) => "ARTIST",
            PlaybackSource::Favorites => "FAVORITES",
            PlaybackSource::Queue => "QUEUE",
            PlaybackSource::Direct => "DIRECT",
            PlaybackSource::Other => "OTHER",
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
            "DIRECT" => PlaybackSource::Direct,
            "QUEUE" => PlaybackSource::Queue,
            "SEARCH" => PlaybackSource::Other,
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