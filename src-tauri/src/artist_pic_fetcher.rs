use crate::db;
use crate::scanner::ScanProgress;
use futures::stream::{self, StreamExt, TryStreamExt};
use primp::{Client, Impersonate};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const MAX_IMAGE_BYTES: u64 = 50 * 1024 * 1024;
const MAX_IMAGE_HEIGHT: u32 = 1000;
const MAX_CONCURRENT_DOWNLOADS: usize = 10;
const FETCH_WARNING_EVENT: &str = "artist-fetch-warning";
const LASTFM_BASE: &str = "https://www.last.fm";
const DEEZER_API: &str = "https://api.deezer.com";

static FETCH_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
enum FetchError {
    NoImageFound(String),
    Network(String),
    Http { code: u16, url: String },
    TooLarge(u64),
    Decode(String),
    Write(String),
}

impl FetchError {
    fn is_permanent(&self) -> bool {
        matches!(self, FetchError::NoImageFound(_))
    }
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchError::NoImageFound(a) => write!(f, "no image found for {a}"),
            FetchError::Network(e) => write!(f, "network error: {e}"),
            FetchError::Http { code, url } => write!(f, "http {code} for {url}"),
            FetchError::TooLarge(size) => write!(f, "image too large ({size} bytes)"),
            FetchError::Decode(e) => write!(f, "image decode failed: {e}"),
            FetchError::Write(e) => write!(f, "write failed: {e}"),
        }
    }
}

impl std::error::Error for FetchError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceStatus {
    Reachable,
    Unreachable,
}

#[derive(Debug, Clone, Copy)]
struct Sources {
    lastfm: SourceStatus,
    deezer: SourceStatus,
}

impl Sources {
    fn use_lastfm(&self) -> bool {
        self.lastfm == SourceStatus::Reachable
    }
    fn use_deezer(&self) -> bool {
        self.deezer == SourceStatus::Reachable
    }
    fn any(&self) -> bool {
        self.use_lastfm() || self.use_deezer()
    }
}

#[derive(serde::Serialize, Clone)]
struct FetchWarning {
    level: &'static str,
    message: String,
}

fn emit_warning(app_handle: &AppHandle, level: &'static str, message: String) {
    let _ = app_handle.emit(FETCH_WARNING_EVENT, FetchWarning { level, message });
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .to_lowercase()
}

fn download_exceeds_cap(content_length: Option<u64>, actual_bytes: u64) -> bool {
    actual_bytes > MAX_IMAGE_BYTES || content_length.is_some_and(|l| l > MAX_IMAGE_BYTES)
}

fn maybe_downscale(img: &image::DynamicImage) -> image::DynamicImage {
    if img.height() > MAX_IMAGE_HEIGHT {
        let ratio = MAX_IMAGE_HEIGHT as f32 / img.height() as f32;
        let width = ((img.width() as f32 * ratio).round() as u32).max(1);
        img.resize(
            width,
            MAX_IMAGE_HEIGHT,
            image::imageops::FilterType::Triangle,
        )
    } else {
        img.clone()
    }
}

async fn probe_source(client: &Client, url: &str) -> SourceStatus {
    let reachable = client
        .get(url)
        .timeout(Duration::from_secs(4))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);
    if reachable {
        SourceStatus::Reachable
    } else {
        SourceStatus::Unreachable
    }
}

async fn probe_sources(client: &Client) -> Sources {
    let (lastfm, deezer) = tokio::join!(
        probe_source(client, LASTFM_BASE),
        probe_source(client, DEEZER_API),
    );
    Sources { lastfm, deezer }
}

async fn get_lastfm_image_url(client: &Client, artist: &str) -> Result<Option<String>, FetchError> {
    let encoded_name = urlencoding::encode(artist);
    let target_url = format!("{LASTFM_BASE}/music/{}/+images", encoded_name);

    let response = client
        .get(&target_url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| FetchError::Network(e.to_string()))?;

    if response.status() == primp::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !response.status().is_success() {
        return Err(FetchError::Http {
            code: response.status().as_u16(),
            url: target_url,
        });
    }

    let html_content = response
        .text()
        .await
        .map_err(|e| FetchError::Network(e.to_string()))?;
    let document = Html::parse_document(&html_content);
    let image_list_selector = Selector::parse("ul.image-list").unwrap();
    let img_tag_selector = Selector::parse("img").unwrap();

    if let Some(src) = document
        .select(&image_list_selector)
        .next()
        .and_then(|list_element| list_element.select(&img_tag_selector).next())
        .and_then(|img_element| img_element.value().attr("src"))
    {
        let high_res_url = src.replace("avatar170s", "770x0").replace("180s", "770x0");
        return Ok(Some(high_res_url));
    }

    Ok(None)
}

async fn get_deezer_image_url(client: &Client, artist: &str) -> Result<Option<String>, FetchError> {
    let encoded_name = urlencoding::encode(artist);
    let target_url = format!("{DEEZER_API}/search/artist?q={}", encoded_name);

    let response = client
        .get(&target_url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| FetchError::Network(e.to_string()))?;
    if !response.status().is_success() {
        return Err(FetchError::Http {
            code: response.status().as_u16(),
            url: target_url,
        });
    }

    let text = response
        .text()
        .await
        .map_err(|e| FetchError::Network(e.to_string()))?;
    let json: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| FetchError::Decode(e.to_string()))?;

    if let Some(picture_xl) = json["data"]
        .as_array()
        .and_then(|data| data.first())
        .and_then(|artist_data| artist_data["picture_xl"].as_str())
    {
        return Ok(Some(picture_xl.to_string()));
    }

    Ok(None)
}

async fn find_image_url(
    client: &Client,
    artist: &str,
    sources: Sources,
) -> Result<Option<String>, FetchError> {
    if sources.use_lastfm() {
        match get_lastfm_image_url(client, artist).await {
            Ok(Some(url)) => return Ok(Some(url)),
            Ok(None) => {
                if !sources.use_deezer() {
                    return Ok(None);
                }
            }
            Err(e) => {
                tracing::warn!(artist = %artist, error = %e, "last.fm lookup failed");
                if !sources.use_deezer() {
                    return Err(e);
                }
            }
        }
    } else if !sources.use_deezer() {
        return Ok(None);
    }

    get_deezer_image_url(client, artist).await
}

async fn process_artist_image(
    client: &Client,
    artist_id: i64,
    artist: &str,
    images_dir: &Path,
    pool: &Pool<SqliteConnectionManager>,
    sources: Sources,
) -> Result<String, FetchError> {
    let img_url = match find_image_url(client, artist, sources).await? {
        Some(url) => url,
        None => return Err(FetchError::NoImageFound(artist.to_string())),
    };

    let response = client
        .get(&img_url)
        .timeout(Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| FetchError::Network(e.to_string()))?;
    if !response.status().is_success() {
        return Err(FetchError::Http {
            code: response.status().as_u16(),
            url: img_url,
        });
    }

    if download_exceeds_cap(response.content_length(), 0) {
        return Err(FetchError::TooLarge(response.content_length().unwrap_or(0)));
    }

    let image_bytes: Vec<u8> = response
        .bytes_stream()
        .take((MAX_IMAGE_BYTES + 1) as usize)
        .map(|chunk| chunk.map(|b| b.to_vec()))
        .try_concat()
        .await
        .map_err(|e| FetchError::Network(e.to_string()))?;

    if download_exceeds_cap(None, image_bytes.len() as u64) {
        return Err(FetchError::TooLarge(image_bytes.len() as u64));
    }

    let artist = artist.to_string();
    let images_dir = images_dir.to_path_buf();
    let pool = pool.clone();

    tokio::task::spawn_blocking(move || -> Result<String, FetchError> {
        let img =
            image::load_from_memory(&image_bytes).map_err(|e| FetchError::Decode(e.to_string()))?;
        let img = maybe_downscale(&img);

        let encoder = webp::Encoder::from_image(&img)
            .map_err(|e| FetchError::Decode(format!("webp encoder: {e}")))?;
        let webp_data = encoder.encode(90.0).to_vec();

        let filename = format!("{artist_id}_{}.webp", sanitize_filename(&artist));
        let output_path = images_dir.join(&filename);
        std::fs::write(&output_path, &webp_data).map_err(|e| FetchError::Write(e.to_string()))?;

        if let Ok(conn) = pool.get() {
            let _ = conn.execute(
                "UPDATE artist SET profile_image = ?, banner_image = ? WHERE id = ?",
                rusqlite::params![filename, filename, artist_id],
            );
            let _ = db::report_fetch_success(&conn, artist_id);
        } else {
            tracing::warn!(artist_id, "failed to get db connection for image record");
        }

        Ok(filename)
    })
    .await
    .map_err(|e| FetchError::Write(format!("blocking task join: {e}")))?
}

pub async fn fetch_artist_images(
    artists: &HashMap<i64, String>,
    app_dir: &Path,
    pool: Pool<SqliteConnectionManager>,
    app_handle: &AppHandle,
) -> Result<(), Box<dyn Error>> {
    if artists.is_empty() {
        return Ok(());
    }
    if FETCH_IN_PROGRESS.swap(true, Ordering::SeqCst) {
        tracing::debug!("artist image fetch already in progress; skipping duplicate batch");
        return Ok(());
    }

    struct FetchGuard;
    impl Drop for FetchGuard {
        fn drop(&mut self) {
            FETCH_IN_PROGRESS.store(false, Ordering::SeqCst);
        }
    }
    let _guard = FetchGuard;

    let images_dir = app_dir.join("artists");
    tokio::fs::create_dir_all(&images_dir).await?;

    let client = Client::builder().impersonate(Impersonate::Random).build()?;

    let sources = probe_sources(&client).await;
    if !sources.any() {
        tracing::warn!("artist photo sources unreachable; skipping fetch batch");
        emit_warning(
            app_handle,
            "error",
            "No internet connection, or artist photo services (last.fm, Deezer) are blocked. Artist photos were skipped.".into(),
        );
        return Ok(());
    }
    if !sources.use_lastfm() {
        tracing::warn!("last.fm unreachable; using Deezer for artist photos");
        emit_warning(
            app_handle,
            "warn",
            "Last.fm is unreachable or blocked — using Deezer for artist photos.".into(),
        );
    } else if !sources.use_deezer() {
        tracing::warn!("deezer unreachable; using last.fm for artist photos");
        emit_warning(
            app_handle,
            "warn",
            "Deezer is unreachable or blocked — using Last.fm for artist photos.".into(),
        );
    }

    let total = artists.len();
    let completed = Arc::new(AtomicUsize::new(0));
    let successes = Arc::new(AtomicUsize::new(0));
    let failures = Arc::new(AtomicUsize::new(0));

    stream::iter(artists.iter().map(|(&id, name)| (id, name.clone())))
        .for_each_concurrent(MAX_CONCURRENT_DOWNLOADS, {
            let client = client.clone();
            let pool = pool.clone();
            let images_dir = images_dir.clone();
            let completed = Arc::clone(&completed);
            let successes = Arc::clone(&successes);
            let failures = Arc::clone(&failures);
            let app_handle = app_handle.clone();
            move |(artist_id, artist_name)| {
                let client = client.clone();
                let pool = pool.clone();
                let images_dir = images_dir.clone();
                let completed = Arc::clone(&completed);
                let successes = Arc::clone(&successes);
                let failures = Arc::clone(&failures);
                let app_handle = app_handle.clone();
                async move {
                    let skipped = artist_name.is_empty() || artist_name == "Unknown Artist";
                    if !skipped {
                        match process_artist_image(
                            &client,
                            artist_id,
                            &artist_name,
                            &images_dir,
                            &pool,
                            sources,
                        )
                        .await
                        {
                            Ok(_) => {
                                successes.fetch_add(1, Ordering::SeqCst);
                            }
                            Err(e) => {
                                failures.fetch_add(1, Ordering::SeqCst);
                                if e.is_permanent() {
                                    tracing::info!(
                                        artist = %artist_name,
                                        error = %e,
                                        "no artist image found on any source"
                                    );
                                    let pool = pool.clone();
                                    let _ = tokio::task::spawn_blocking(move || {
                                        if let Ok(conn) = pool.get() {
                                            let _ = db::report_fetch_failure(&conn, artist_id);
                                        }
                                    })
                                    .await;
                                } else {
                                    tracing::warn!(
                                        artist = %artist_name,
                                        error = %e,
                                        "transient artist image fetch failure"
                                    );
                                }
                            }
                        }
                    }
                    let idx = completed.fetch_add(1, Ordering::SeqCst) + 1;
                    let _ = app_handle.emit(
                        "fetch-progress",
                        ScanProgress {
                            current: idx,
                            total,
                            message: format!("Fetching artist image: {artist_name}"),
                        },
                    );
                }
            }
        })
        .await;

    let failures = failures.load(Ordering::SeqCst);
    let successes = successes.load(Ordering::SeqCst);
    tracing::info!(successes, failures, "artist image fetch batch finished");
    if failures > 0 {
        emit_warning(
            app_handle,
            "warn",
            format!("Couldn't find artist photos for {failures} artist(s)."),
        );
    }

    Ok(())
}

pub async fn fetch_single_artist_image(
    artist_id: i64,
    artist: &str,
    app_dir: &Path,
    pool: Pool<SqliteConnectionManager>,
) -> Result<String, Box<dyn Error>> {
    let images_dir = app_dir.join("artists");
    tokio::fs::create_dir_all(&images_dir).await?;

    let client = Client::builder().impersonate(Impersonate::Random).build()?;

    let sources = probe_sources(&client).await;
    if !sources.any() {
        return Err(
            "No internet connection, or artist photo services (last.fm, Deezer) are blocked."
                .to_string()
                .into(),
        );
    }

    process_artist_image(&client, artist_id, artist, &images_dir, &pool, sources)
        .await
        .map_err(|e| Box::<dyn Error>::from(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filename_includes_artist_id_so_variants_dont_collide() {
        let a = format!("{}_{}.webp", 1, sanitize_filename("AC/DC"));
        let b = format!("{}_{}.webp", 2, sanitize_filename("Ac Dc"));
        assert_ne!(a, b);
        assert!(a.ends_with("ac_dc.webp"));
        assert_eq!(format!("{}_{}.webp", 1, sanitize_filename("AC/DC")), a);
    }

    #[test]
    fn fetch_error_classification() {
        assert!(FetchError::NoImageFound("x".into()).is_permanent());
        assert!(!FetchError::Network("err".into()).is_permanent());
        assert!(
            !FetchError::Http {
                code: 500,
                url: "u".into()
            }
            .is_permanent()
        );
        assert!(!FetchError::TooLarge(1).is_permanent());
        assert!(!FetchError::Decode("e".into()).is_permanent());
        assert!(!FetchError::Write("e".into()).is_permanent());
    }

    #[test]
    fn downscales_only_when_height_exceeds_limit() {
        use image::RgbaImage;

        let big = image::DynamicImage::ImageRgba8(RgbaImage::from_pixel(
            2000,
            2000,
            image::Rgba([10, 20, 30, 255]),
        ));
        let resized = maybe_downscale(&big);
        assert!(resized.height() <= MAX_IMAGE_HEIGHT);
        assert_eq!(resized.width(), MAX_IMAGE_HEIGHT);

        let small = image::DynamicImage::ImageRgba8(RgbaImage::from_pixel(
            800,
            600,
            image::Rgba([10, 20, 30, 255]),
        ));
        let resized = maybe_downscale(&small);
        assert_eq!(resized.height(), 600);
        assert_eq!(resized.width(), 800);
    }

    #[test]
    fn size_cap_enforced() {
        assert!(download_exceeds_cap(Some(MAX_IMAGE_BYTES + 1), 0));
        assert!(!download_exceeds_cap(Some(MAX_IMAGE_BYTES), 0));
        assert!(download_exceeds_cap(None, MAX_IMAGE_BYTES + 1));
        assert!(!download_exceeds_cap(None, MAX_IMAGE_BYTES));
        assert!(download_exceeds_cap(Some(10), MAX_IMAGE_BYTES + 1));
    }
}
