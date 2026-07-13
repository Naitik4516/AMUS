//! Human-readable CLI output.

use super::protocol::{
    CliData, QueueTrackLine, SearchNamedLine, SearchTrackLine,
};

pub fn format_data(data: &CliData) -> String {
    match data {
        CliData::Message { text } => {
            if text.is_empty() {
                String::new()
            } else {
                text.clone()
            }
        }
        CliData::Status {
            is_playing,
            has_track,
            title,
            artist,
            album,
            position_sec,
            duration_sec,
            volume_percent,
            muted,
            shuffle,
            repeat,
        } => {
            if !has_track {
                return "⏹ Stopped\n\nNo track loaded.".into();
            }
            let state = if *is_playing {
                "▶ Playing"
            } else {
                "⏸ Paused"
            };
            let vol = if *muted {
                format!("{volume_percent}% (muted)")
            } else {
                format!("{volume_percent}%")
            };
            format!(
                "{state}\n\n\
Song      : {}\n\
Artist    : {}\n\
Album     : {}\n\n\
Time       {} / {}\n\
Volume     {vol}\n\
Shuffle    {}\n\
Repeat     {}",
                title.as_deref().unwrap_or("Unknown"),
                artist.as_deref().unwrap_or("Unknown"),
                album.as_deref().unwrap_or("Unknown"),
                format_duration(*position_sec as u32),
                format_duration(*duration_sec),
                if *shuffle { "On" } else { "Off" },
                format_repeat(repeat),
            )
        }
        CliData::Queue { tracks } => format_queue(tracks),
        CliData::SearchResults {
            tracks,
            artists,
            albums,
        } => format_search(tracks, artists, albums),
        CliData::NamedList { items } => format_named_list(items),
        CliData::TrackList { label, tracks } => {
            let mut s = format!("{label}\n\n");
            if tracks.is_empty() {
                s.push_str("(empty)\n");
            } else {
                for (i, t) in tracks.iter().enumerate() {
                    s.push_str(&format!(
                        "{:>3}. {} — {}  [{}]\n",
                        i + 1,
                        t.title,
                        t.artist,
                        format_duration(t.duration_sec)
                    ));
                }
            }
            s
        }
        CliData::Version { version } => format!("AMUS {version}"),
        CliData::UpdateResult { message } => message.clone(),
    }
}

fn format_queue(tracks: &[QueueTrackLine]) -> String {
    if tracks.is_empty() {
        return "Queue is empty.".into();
    }
    let mut s = String::from("Queue\n\n");
    for (i, t) in tracks.iter().enumerate() {
        s.push_str(&format!(
            "{:>3}. {} — {} ({})  [{}]\n",
            i + 1,
            t.title,
            t.artist,
            t.album,
            format_duration(t.duration_sec)
        ));
    }
    s
}

fn format_search(
    tracks: &[SearchTrackLine],
    artists: &[SearchNamedLine],
    albums: &[SearchNamedLine],
) -> String {
    let mut s = String::new();
    if !tracks.is_empty() {
        s.push_str("Tracks\n");
        for t in tracks {
            s.push_str(&format!(
                "  [{id}] {title} — {artist} ({album})\n",
                id = t.id,
                title = t.title,
                artist = t.artist,
                album = t.album
            ));
        }
        s.push('\n');
    }
    if !artists.is_empty() {
        s.push_str("Artists\n");
        for a in artists {
            s.push_str(&format!("  [{id}] {name}\n", id = a.id, name = a.name));
        }
        s.push('\n');
    }
    if !albums.is_empty() {
        s.push_str("Albums\n");
        for a in albums {
            s.push_str(&format!("  [{id}] {name}\n", id = a.id, name = a.name));
        }
        s.push('\n');
    }
    if s.is_empty() {
        s.push_str("No results.\n");
    }
    s
}

fn format_named_list(items: &[SearchNamedLine]) -> String {
    if items.is_empty() {
        return "(none)\n".into();
    }
    let mut s = String::new();
    for item in items {
        s.push_str(&format!("[{id}] {name}\n", id = item.id, name = item.name));
    }
    s
}

pub fn format_duration(sec: u32) -> String {
    let m = sec / 60;
    let s = sec % 60;
    format!("{m}:{s:02}")
}

fn format_repeat(repeat: &str) -> &'static str {
    match repeat {
        "ALL" => "Queue",
        "ONE" => "One",
        _ => "Off",
    }
}

pub fn format_file_info(
    title: &str,
    artist: &str,
    album: &str,
    genre: Option<&str>,
    bitrate_kbps: Option<u32>,
    sample_rate_hz: Option<u32>,
    duration_sec: u32,
) -> String {
    let mut lines = vec![
        format!("Title: {title}"),
        format!("Artist: {artist}"),
        format!("Album: {album}"),
    ];
    if let Some(g) = genre {
        lines.push(format!("Genre: {g}"));
    }
    if let Some(br) = bitrate_kbps {
        lines.push(format!("Bitrate: {br} kbps"));
    }
    if let Some(sr) = sample_rate_hz {
        let khz = sr as f64 / 1000.0;
        if (khz - khz.round()).abs() < 0.01 {
            lines.push(format!("Sample Rate: {:.0} kHz", khz));
        } else {
            lines.push(format!("Sample Rate: {:.1} kHz", khz));
        }
    }
    lines.push(format!("Duration: {}", format_duration(duration_sec)));
    lines.join("\n")
}
