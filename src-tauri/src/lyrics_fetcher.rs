use crate::error::Result;
use crate::models::Lyrics;
use serde::Deserialize;
use std::time::Duration;
use tracing::warn;

const LRCLIB_API: &str = "https://lrclib.net/api/search";
const DURATION_TOLERANCE: f64 = 2.0;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Deserialize, Default)]
#[allow(dead_code)]
struct LrclibResponse {
    id: Option<u64>,
    #[serde(alias = "trackName")]
    track_name: Option<String>,
    #[serde(alias = "artistName")]
    artist_name: Option<String>,
    duration: Option<f64>,
    instrumental: Option<bool>,
    #[serde(alias = "plainLyrics")]
    plain_lyrics: Option<String>,
    #[serde(alias = "syncedLyrics")]
    synced_lyrics: Option<String>,
}

fn find_best_match(results: Vec<LrclibResponse>, target_duration: u32) -> Option<LrclibResponse> {
    let target = target_duration as f64;

    let candidates: Vec<LrclibResponse> = results
        .into_iter()
        .filter(|r| !r.instrumental.unwrap_or(false))
        .filter(|r| {
            r.duration
                .is_some_and(|d| (d - target).abs() <= DURATION_TOLERANCE)
        })
        .collect();

    if candidates.is_empty() {
        return None;
    }

    candidates.into_iter().min_by(|a, b| {
        (a.duration.unwrap_or(0.0) - target)
            .abs()
            .partial_cmp(&(b.duration.unwrap_or(0.0) - target).abs())
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
    let url = format!("{LRCLIB_API}?artist_name={encoded_artist}&track_name={encoded_track}");

    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|e| crate::error::Error::Unknown(format!("LRCLIB client failed: {e}")))?;

    let response = match client
        .get(&url)
        .header("User-Agent", "AMUS/0.7.0 (music player)")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            warn!(error = %e, "lrclib request failed");
            return Ok(None);
        }
    };

    if !response.status().is_success() {
        warn!(
            status = response.status().as_u16(),
            "lrclib returned a non-success status"
        );
        return Ok(None);
    }

    let results: Vec<LrclibResponse> = match response.json().await {
        Ok(v) => v,
        Err(e) => {
            warn!(error = %e, "lrclib returned a body that could not be parsed");
            return Ok(None);
        }
    };

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
