use crate::db;
use crate::scanner::ScanProgress;
use anyhow::{Result, anyhow};
use image::ImageFormat;
use primp::{Client, Impersonate};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::fs;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .impersonate(Impersonate::ChromeV146)
        .timeout(Duration::from_secs(15))
        .build()
        .expect("failed to create HTTP client")
});

static FETCH_SEMAPHORE: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(15));

#[derive(Deserialize)]
struct BingMetadata {
    murl: String,
}

#[derive(Deserialize)]
struct DdgResult {
    image: String,
}

#[derive(Deserialize)]
struct DdgResponse {
    results: Vec<DdgResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRecord {
    pub filename: String,
    pub hash: String,
    pub attempts: u32,
    pub status: ImageStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageStatus {
    Ok,
    Failed,
}
#[derive(Clone, Copy)]
enum AspectRatio {
    Square,
    Wide,
}

async fn search_bing(
    client: &Client,
    query: &str,
    aspect_ratio: AspectRatio,
) -> Result<Vec<String>> {
    let aspect_ratio = match aspect_ratio {
        AspectRatio::Square => "square",
        AspectRatio::Wide => "wide",
    };
    let url = format!(
        "https://www.bing.com/images/async?q={}&async=1&first=1&count=5&qft=+filterui:aspect-{}",
        urlencoding::encode(query),
        aspect_ratio
    );
    let resp = client.get(&url).send().await?.text().await?;
    let document = Html::parse_document(&resp);
    let selector = Selector::parse("a.iusc").unwrap();

    let mut urls = Vec::new();
    for element in document.select(&selector) {
        if let Some(m_attr) = element.value().attr("m") {
            if let Ok(metadata) = serde_json::from_str::<BingMetadata>(m_attr) {
                urls.push(metadata.murl);
            }
        }
    }

    if urls.is_empty() {
        Err(anyhow!("No images found on Bing"))
    } else {
        Ok(urls)
    }
}

async fn search_duckduckgo(
    client: &Client,
    query: &str,
    aspect_ratio: AspectRatio,
) -> Result<Vec<String>> {
    fn extract_vqd(html: &str) -> Option<String> {
        for marker in [r#"vqd=""#, "vqd=", "vqd='"] {
            if let Some(start) = html.find(marker) {
                let start = start + marker.len();
                let rest = &html[start..];
                let end_chars = if marker.contains('"') {
                    "\""
                } else if marker.contains('\'') {
                    "'"
                } else {
                    "&"
                };
                if let Some(end) = rest.find(|c| end_chars.contains(c)) {
                    return Some(rest[..end].to_string());
                }
            }
        }
        None
    }

    let aspect_filter = match aspect_ratio {
        AspectRatio::Square => "layout%3ASquare",
        AspectRatio::Wide => "layout%3AWide",
    };
    let vqd_url = format!(
        "https://duckduckgo.com/?q={}&iaf={}",
        urlencoding::encode(query),
        aspect_filter
    );
    let vqd_resp = client.get(&vqd_url).send().await?.text().await?;
    let vqd =
        extract_vqd(&vqd_resp).ok_or_else(|| anyhow!("Could not extract VQD from DuckDuckGo"))?;

    let search_url = format!(
        "https://duckduckgo.com/i.js?o=json&q={}&vqd={}&count=5",
        urlencoding::encode(query),
        vqd
    );
    let resp = client.get(&search_url).send().await?.text().await?;
    let data: DdgResponse = serde_json::from_str(&resp)?;

    let urls: Vec<String> = data.results.into_iter().map(|r| r.image).collect();
    if urls.is_empty() {
        Err(anyhow!("No images found on DuckDuckGo"))
    } else {
        Ok(urls)
    }
}

async fn search_images(
    client: &Client,
    query: &str,
    aspect_ratio: AspectRatio,
) -> Result<Vec<String>> {
    let (bing, ddg) = tokio::join!(
        search_bing(client, query, aspect_ratio),
        search_duckduckgo(client, query, aspect_ratio),
    );
    bing.or_else(|_| ddg)
}

async fn download_image(
    query: &str,
    aspect_ratio: AspectRatio,
    subdir: &str,
    thumbnail: Option<(u32, u32)>,
    app_dir: &Path,
) -> Result<String> {
    let _permit = FETCH_SEMAPHORE.acquire().await.map_err(|e| anyhow!(e))?;
    let client = &*CLIENT;

    let image_urls = search_images(client, query, aspect_ratio).await?;

    let out_dir = app_dir.join(subdir);
    fs::create_dir_all(&out_dir).await?;

    let mut last_err = anyhow!("No candidate URLs to try");
    for url in &image_urls {
        let response = match client.get(url).send().await {
            Ok(r) => r,
            Err(e) => {
                last_err = anyhow!("error sending request for url ({url}): {e}");
                continue;
            }
        };
        let response = match response.error_for_status() {
            Ok(r) => r,
            Err(e) => {
                last_err = anyhow!("{e}");
                continue;
            }
        };
        let bytes = match response.bytes().await {
            Ok(b) => b,
            Err(e) => {
                last_err = anyhow!("failed to read body from ({url}): {e}");
                continue;
            }
        };
        let format = image::guess_format(&bytes);
        if format.is_err() {
            last_err = anyhow!("Downloaded file is not a valid image ({url})");
            continue;
        }
        if format.unwrap() == image::ImageFormat::Avif {
            last_err = anyhow!("AVIF format not supported, skipping ({url})");
            continue;
        }

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash = hex::encode(hasher.finalize());
        let filename = format!("{hash}.webp");
        let dest_path = out_dir.join(&filename);

        if fs::metadata(&dest_path).await.is_err() {
            let img = image::load_from_memory(&bytes)?;
            let img = match thumbnail {
                Some((w, h)) => img.thumbnail(w, h),
                None => img,
            };
            img.save_with_format(dest_path, ImageFormat::WebP)?;
        }

        return Ok(filename);
    }

    Err(last_err)
}

async fn download_and_save(artist_name: &str, app_dir: &Path) -> Result<String> {
    let query = format!("\"{artist_name}\" music artist");
    download_image(
        &query,
        AspectRatio::Square,
        "artists",
        Some((250, 250)),
        app_dir,
    )
    .await
}

async fn download_and_save_banner(artist_name: &str, app_dir: &Path) -> Result<String> {
    let query = format!("\"{artist_name}\" music artist");
    download_image(&query, AspectRatio::Wide, "artist_banner", None, app_dir).await
}

pub async fn fetch_single_artist_images(
    artist_id: i64,
    artist_name: &str,
    app_dir: &Path,
    pool: Pool<SqliteConnectionManager>,
    fetch_pic: bool,
    fetch_banner: bool,
) -> Result<()> {
    if (artist_name == "Unknown Artist") || (!fetch_pic && !fetch_banner) {
        return Ok(());
    }

    let conn = pool.get().map_err(|e| anyhow!(e))?;
    let mut any_attempted = false;
    let mut any_succeeded = false;

    if fetch_pic {
        let has_photo = db::artist_has_photo(&conn, artist_id).unwrap_or(false);
        if !has_photo {
            any_attempted = true;
            match download_and_save(artist_name, app_dir).await {
                Ok(filename) => {
                    let _ = db::update_artist_profile_image(&conn, artist_id, &filename);
                    any_succeeded = true;
                }
                Err(e) => {
                    eprintln!("Failed to fetch profile image for {artist_name}: {e}");
                }
            }
        }
    }

    if fetch_banner {
        let has_banner = db::artist_has_banner(&conn, artist_id).unwrap_or(false);
        if !has_banner {
            any_attempted = true;
            match download_and_save_banner(artist_name, app_dir).await {
                Ok(filename) => {
                    let _ = db::update_artist_banner_image(&conn, artist_id, &filename);
                    any_succeeded = true;
                }
                Err(e) => {
                    eprintln!("Failed to fetch banner image for {artist_name}: {e}");
                }
            }
        }
    }

    if any_succeeded {
        let _ = db::report_fetch_success(&conn, artist_id);
    } else if any_attempted {
        let _ = db::report_fetch_failure(&conn, artist_id);
    }

    Ok(())
}

pub async fn fetch_artist_images(
    artists: &HashMap<i64, String>,
    app_dir: &Path,
    pool: Pool<SqliteConnectionManager>,
    app_handle: &AppHandle,
    fetch_pic: bool,
    fetch_banner: bool,
) -> Result<()> {
    if !fetch_pic && !fetch_banner {
        return Ok(());
    }

    let total = artists.len();
    let mut set = JoinSet::new();
    let mut spawned_count: usize = 0;

    // Reuse one connection for the pre-check loop
    let check_conn = pool.get().ok();

    for (aid, name) in artists.iter() {
        if name == "Unknown Artist" {
            continue;
        }

        let skip = check_conn.as_ref().map_or(false, |conn| {
            let needs_photo = fetch_pic && !db::artist_has_photo(conn, *aid).unwrap_or(false);
            let needs_banner = fetch_banner && !db::artist_has_banner(conn, *aid).unwrap_or(false);
            !needs_photo && !needs_banner
        });
        if skip {
            continue;
        }

        spawned_count += 1;
        let idx = spawned_count;

        let app_dir = app_dir.to_path_buf();
        let artist_name = name.clone();
        let artist_id = *aid;
        let pool_clone = pool.clone();
        let app_handle_clone = app_handle.clone();

        set.spawn(async move {
            // Get one connection to reuse across all DB ops
            let conn = pool_clone.get().ok();
            let mut any_attempted = false;
            let mut any_succeeded = false;

            if fetch_pic {
                match download_and_save(&artist_name, &app_dir).await {
                    Ok(filename) => {
                        if let Some(ref conn) = conn {
                            let _ = db::update_artist_profile_image(conn, artist_id, &filename);
                        }
                        any_succeeded = true;
                    }
                    Err(e) => {
                        any_attempted = true;
                        eprintln!("Failed to fetch profile image for {artist_name}: {e}");
                    }
                }
            }

            if fetch_banner {
                match download_and_save_banner(&artist_name, &app_dir).await {
                    Ok(filename) => {
                        if let Some(ref conn) = conn {
                            let _ = db::update_artist_banner_image(conn, artist_id, &filename);
                        }
                        any_succeeded = true;
                    }
                    Err(e) => {
                        any_attempted = true;
                        eprintln!("Failed to fetch banner image for {artist_name}: {e}");
                    }
                }
            }

            if let Some(ref conn) = conn {
                if any_succeeded {
                    let _ = db::report_fetch_success(conn, artist_id);
                } else if any_attempted {
                    let _ = db::report_fetch_failure(conn, artist_id);
                }
            }

            let _ = app_handle_clone.emit(
                "fetch-progress",
                ScanProgress {
                    current: idx,
                    total,
                    message: format!("Fetching artist: {artist_name}"),
                },
            );
        });
    }

    let mut done = 0;
    while set.join_next().await.is_some() {
        done += 1;
        let _ = app_handle.emit(
            "fetch-progress",
            ScanProgress {
                current: done,
                total,
                message: format!("Artist images ({}/{})", done, total),
            },
        );
    }

    let _ = app_handle.emit(
        "fetch-progress",
        ScanProgress {
            current: total,
            total,
            message: "Artist images updated".to_string(),
        },
    );

    Ok(())
}
