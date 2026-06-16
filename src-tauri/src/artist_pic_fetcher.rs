use anyhow::{anyhow, Result};
use primp::{Client, Impersonate};
use scraper::{Html, Selector};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::sync::Semaphore;
use std::sync::LazyLock;

static FETCH_SEMAPHORE: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(3));

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

async fn get_client() -> Result<Client> {
    Ok(Client::builder()
        .impersonate(Impersonate::ChromeV146)
        .build()?)
}

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

async fn search_bing(client: &Client, query: &str) -> Result<String> {
    let url = format!(
        "https://www.bing.com/images/async?q={}&async=1&first=1&count=35",
        urlencoding::encode(query)
    );
    let resp = client.get(&url).send().await?.text().await?;
    let document = Html::parse_document(&resp);
    let selector = Selector::parse("a.iusc").unwrap();

    for element in document.select(&selector) {
        if let Some(m_attr) = element.value().attr("m") {
            if let Ok(metadata) = serde_json::from_str::<BingMetadata>(m_attr) {
                return Ok(metadata.murl);
            }
        }
    }

    Err(anyhow!("No images found on Bing"))
}

async fn search_duckduckgo(client: &Client, query: &str) -> Result<String> {
    // Step 1: Get VQD
    let vqd_url = format!("https://duckduckgo.com/?q={}", urlencoding::encode(query));
    let vqd_resp = client.get(&vqd_url).send().await?.text().await?;
    let vqd =
        extract_vqd(&vqd_resp).ok_or_else(|| anyhow!("Could not extract VQD from DuckDuckGo"))?;

    // Step 2: Get Images
    let search_url = format!(
        "https://duckduckgo.com/i.js?o=json&q={}&vqd={}",
        urlencoding::encode(query),
        vqd
    );
    let resp = client.get(&search_url).send().await?.text().await?;
    let data: DdgResponse = serde_json::from_str(&resp)?;

    data.results
        .first()
        .map(|r| r.image.clone())
        .ok_or_else(|| anyhow!("No images found on DuckDuckGo"))
}

pub async fn fetch_artist_image(artist_name: &str, app_dir: &Path) -> Result<String> {
    let _permit = FETCH_SEMAPHORE.acquire().await.map_err(|e| anyhow!(e))?;
    
    let client = get_client().await?;
    let query = format!("{} artist profile picture", artist_name);

    let image_url = match search_bing(&client, &query).await {
        Ok(url) => url,
        Err(_) => search_duckduckgo(&client, &query).await?,
    };

    // Download the image
    let resp = client.get(&image_url).send().await?;
    let bytes = resp.bytes().await?;

    // Hash the image
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hash = hex::encode(hasher.finalize());

    // Guess extension from URL or use generic
    let ext = if image_url.ends_with(".png") {
        "png"
    } else if image_url.ends_with(".jpg") || image_url.ends_with(".jpeg") {
        "jpg"
    } else if image_url.ends_with(".webp") {
        "webp"
    } else {
        "jpg"
    };

    let filename = format!("{}.{}", hash, ext);
    let artists_dir = app_dir.join("artists");
    if !artists_dir.exists() {
        tokio::fs::create_dir_all(&artists_dir).await?;
    }

    let dest_path = artists_dir.join(&filename);
    if !dest_path.exists() {
        tokio::fs::write(dest_path, bytes).await?;
    }

    Ok(filename)
}
