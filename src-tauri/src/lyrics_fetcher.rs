use crate::error::Result;
use crate::models::Lyrics;
use serde::Deserialize;

const LRCLIB_API: &str = "https://lrclib.net/api/search";
const DURATION_TOLERANCE: f64 = 2.0;

#[derive(Deserialize)]
#[allow(dead_code)]
struct LrclibResponse {
    id: u64,
    #[serde(alias = "trackName")]
    track_name: String,
    #[serde(alias = "artistName")]
    artist_name: String,
    duration: f64,
    instrumental: bool,
    #[serde(alias = "plainLyrics")]
    plain_lyrics: Option<String>,
    #[serde(alias = "syncedLyrics")]
    synced_lyrics: Option<String>,
}

fn find_best_match(results: Vec<LrclibResponse>, target_duration: u32) -> Option<LrclibResponse> {
    let target = target_duration as f64;

    let candidates: Vec<LrclibResponse> = results
        .into_iter()
        .filter(|r| !r.instrumental)
        .filter(|r| (r.duration - target).abs() <= DURATION_TOLERANCE)
        .collect();

    if candidates.is_empty() {
        return None;
    }

    candidates
        .into_iter()
        .min_by(|a, b| {
            (a.duration - target)
                .abs()
                .partial_cmp(&(b.duration - target).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

pub async fn fetch_lyrics(
    artist_name: &str,
    track_name: &str,
    duration_secs: u32,
) -> Result<Option<Lyrics>> {
    let encoded_artist = urlencoding::encode(artist_name);
    let encoded_track = urlencoding::encode(track_name);
    let url = format!(
        "{LRCLIB_API}?artist_name={encoded_artist}&track_name={encoded_track}"
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "AMUS/0.7.0 (music player)")
        .send()
        .await
        .map_err(|e| crate::error::Error::Unknown(format!("LRCLIB request failed: {e}")))?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let results: Vec<LrclibResponse> = response
        .json()
        .await
        .map_err(|e| crate::error::Error::Unknown(format!("LRCLIB parse failed: {e}")))?;

    let best = match find_best_match(results, duration_secs) {
        Some(r) => r,
        None => return Ok(None),
    };

    let has_lyrics = best.plain_lyrics.is_some() || best.synced_lyrics.is_some();
    if !has_lyrics {
        return Ok(None);
    }

    Ok(Some(Lyrics {
        plain_lyrics: best.plain_lyrics,
        synced_lyrics: best.synced_lyrics,
        source: "lrclib".to_string(),
    }))
}
