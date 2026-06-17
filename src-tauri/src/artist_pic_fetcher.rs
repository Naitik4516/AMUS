use anyhow::{Result, anyhow};
use primp::{Client, Impersonate};
use reqwest::header::CONTENT_TYPE;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::fs;
use tokio::sync::{Mutex, Semaphore, mpsc};

static FETCH_SEMAPHORE: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(5));

const MAX_RETRY_ATTEMPTS: u32 = 5;
const WATCHDOG_INTERVAL_SECS: u64 = 300; // re-scan disk every 5 minutes
const MANIFEST_FILENAME: &str = "manifest.json";

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

/// One entry in the manifest. Filenames on disk are content hashes, so this
/// is the only thing that remembers "this artist's image lives at this path".
/// Without it the watchdog would have no way to know who to re-download for.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ImageRecord {
    filename: String,
    hash: String,
    attempts: u32,
    status: ImageStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum ImageStatus {
    Ok,
    Failed,
}

type Manifest = HashMap<String, ImageRecord>;

/// A unit of work on the retry queue: "(re)download this artist's image".
#[derive(Debug, Clone)]
struct RetryJob {
    artist: String,
    attempt: u32,
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
    let vqd_url = format!("https://duckduckgo.com/?q={}", urlencoding::encode(query));
    let vqd_resp = client.get(&vqd_url).send().await?.text().await?;
    let vqd =
        extract_vqd(&vqd_resp).ok_or_else(|| anyhow!("Could not extract VQD from DuckDuckGo"))?;

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

/// Fully decodes the downloaded bytes (not just header sniffing) to confirm
/// they're a real, complete image rather than a truncated/garbled download.
fn validate_image_bytes(bytes: &[u8], content_type: &str) -> Result<&'static str> {
    let img = image::load_from_memory(bytes)
        .map_err(|e| anyhow!("image failed to decode (corrupted): {e}"))?;

    if img.width() == 0 || img.height() == 0 {
        return Err(anyhow!("decoded image has zero dimensions"));
    }

    Ok(match content_type {
        "image/png" => "png",
        "image/webp" => "webp",
        "image/jpeg" | "image/jpg" => "jpg",
        _ => "jpg",
    })
}

/// Re-validates a file already sitting on disk. Used by the watchdog to catch
/// corruption that happens *after* a successful download (disk errors,
/// partial writes from an older version of this tool, manual tampering, etc).
async fn file_is_valid(path: &Path) -> bool {
    match fs::read(path).await {
        Ok(bytes) if !bytes.is_empty() => image::load_from_memory(&bytes).is_ok(),
        _ => false,
    }
}

/// Downloads, validates, and atomically writes one artist's image.
/// "Atomic" means: write to a `.tmp` file, then rename it into place. A
/// crash or kill signal mid-write leaves a stray `.tmp` file, never a
/// half-written file at the real path that a future scan would mistake
/// for a finished (but corrupted) image.
async fn download_and_save(artist_name: &str, app_dir: &Path) -> Result<ImageRecord> {
    let _permit = FETCH_SEMAPHORE.acquire().await.map_err(|e| anyhow!(e))?;

    let client = get_client().await?;
    let query = format!("{artist_name} artist profile picture");

    let image_url = match search_bing(&client, &query).await {
        Ok(url) => url,
        Err(_) => search_duckduckgo(&client, &query).await?,
    };

    let response = client.get(&image_url).send().await?.error_for_status()?;
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let bytes = response.bytes().await?;
    let ext = validate_image_bytes(&bytes, &content_type)?;

    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hash = hex::encode(hasher.finalize());
    let filename = format!("{hash}.{ext}");

    let artists_dir = app_dir.join("artists");
    fs::create_dir_all(&artists_dir).await?;
    let dest_path = artists_dir.join(&filename);
    let tmp_path = artists_dir.join(format!("{hash}.tmp"));

    fs::write(&tmp_path, &bytes).await?;
    fs::rename(&tmp_path, &dest_path).await?;

    Ok(ImageRecord {
        filename,
        hash,
        attempts: 0,
        status: ImageStatus::Ok,
    })
}

/// Shared state handed to the background tasks.
struct AppState {
    app_dir: PathBuf,
    manifest: Mutex<Manifest>,
    retry_tx: mpsc::UnboundedSender<RetryJob>,
}

impl AppState {
    fn manifest_path(&self) -> PathBuf {
        self.app_dir.join(MANIFEST_FILENAME)
    }

    async fn load_manifest(app_dir: &Path) -> Manifest {
        match fs::read_to_string(app_dir.join(MANIFEST_FILENAME)).await {
            Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
            Err(_) => HashMap::new(),
        }
    }

    async fn save_manifest(&self) {
        let manifest = self.manifest.lock().await;
        if let Ok(json) = serde_json::to_string_pretty(&*manifest) {
            let _ = fs::write(self.manifest_path(), json).await;
        }
    }
}

/// Drains the retry queue forever. Each job backs off exponentially
/// (2, 4, 8... seconds, capped) before attempting a (re)download, and
/// re-queues itself on failure up to MAX_RETRY_ATTEMPTS.
async fn retry_worker(state: Arc<AppState>, mut rx: mpsc::UnboundedReceiver<RetryJob>) {
    while let Some(job) = rx.recv().await {
        let backoff = Duration::from_secs(2u64.saturating_pow(job.attempt.min(6)));
        tokio::time::sleep(backoff).await;

        println!("[retry] attempt {} for '{}'", job.attempt + 1, job.artist);

        match download_and_save(&job.artist, &state.app_dir).await {
            Ok(record) => {
                println!("[retry] recovered '{}' -> {}", job.artist, record.filename);
                state
                    .manifest
                    .lock()
                    .await
                    .insert(job.artist.clone(), record);
                state.save_manifest().await;
            }
            Err(e) => {
                eprintln!("[retry] failed '{}': {e}", job.artist);
                if job.attempt + 1 < MAX_RETRY_ATTEMPTS {
                    let _ = state.retry_tx.send(RetryJob {
                        artist: job.artist.clone(),
                        attempt: job.attempt + 1,
                    });
                } else {
                    eprintln!(
                        "[retry] giving up on '{}' after {MAX_RETRY_ATTEMPTS} attempts",
                        job.artist
                    );
                    state.manifest.lock().await.insert(
                        job.artist.clone(),
                        ImageRecord {
                            filename: String::new(),
                            hash: String::new(),
                            attempts: job.attempt + 1,
                            status: ImageStatus::Failed,
                        },
                    );
                    state.save_manifest().await;
                }
            }
        }
    }
}

/// Periodically walks every "Ok" entry in the manifest, fully re-decodes the
/// file it points to, and if that fails (missing, truncated, bit-rotted,
/// whatever) deletes the file and pushes a fresh retry job for that artist.
/// This is what makes corruption-handling "automatic" rather than only
/// happening at download time.
async fn watchdog(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(Duration::from_secs(WATCHDOG_INTERVAL_SECS));
    loop {
        interval.tick().await;
        println!("[watchdog] scanning for corrupted images...");

        let entries: Vec<(String, ImageRecord)> = {
            let manifest = state.manifest.lock().await;
            manifest
                .iter()
                .filter(|(_, r)| r.status == ImageStatus::Ok)
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        };

        for (artist, record) in entries {
            let path = state.app_dir.join("artists").join(&record.filename);

            if !file_is_valid(&path).await {
                eprintln!(
                    "[watchdog] corrupted/missing image for '{artist}' — deleting and re-queueing"
                );
                let _ = fs::remove_file(&path).await;
                state.manifest.lock().await.remove(&artist);
                let _ = state.retry_tx.send(RetryJob { artist, attempt: 0 });
            }
        }

        state.save_manifest().await;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let artists = [
        // "Taylor Swift",
        "Ed Sheeran",
        "Adele",
        // "Drake",
        // "Beyoncé",
        "Imagine Dragons",
        // "Billie Eilish",
        // "Bruno Mars",
        // "Ariana Grande",
        // "The Weeknd",
        "Arijit Singh",
        // "Neha Kakkar",
        "Badshah",
        // "Shreya Ghoshal",
        "Sonu Nigam",
        // "A.R. Rahman",
        // "Jubin Nautiyal",
        // "Shashwat Sachdev",
        "Diljit Dosanjh",
        // "Reble",
        // "Osho Jain",
        // "Vishal Mishra",
        "Anu Malik",
        // "Shankar Mahadevan",
        // "Sunidhi Chauhan",
        "Pritam",
        "Jasmine Sandlas",
        // "Madhur Sharma",
        // "Talwiinder",
        // "Kishor Kumar",
        // "Sidhu Moosewala",
        // "Guru Randhawa",
        // "Sanam",
        // "Rahat Fateh Ali Khan",
        // "Atif Aslam",
        "Rahgir",
        // "Honey Singh",
        "Hanshraj Raghuwanshi",
        "Subh",
        // "Khalid",
        // "Kailash Kher",
        // "Mohammed Rafi",
        // "Lata Mangeshkar",
        // "Asha Bhosle",
        // "Sukhwinder Singh",
        // "Tulsi Kumar",
        // "Agam Agarwal",
        // "Alka Yagnik",
        // "K.K.",
        // "Asees Kaur",
    ];

    let app_dir = PathBuf::from(".");
    fs::create_dir_all(&app_dir).await?;

    let manifest = AppState::load_manifest(&app_dir).await;
    let (retry_tx, retry_rx) = mpsc::unbounded_channel();

    let state = Arc::new(AppState {
        app_dir: app_dir.clone(),
        manifest: Mutex::new(manifest),
        retry_tx: retry_tx.clone(),
    });

    // Background workers — these keep running for the life of the process,
    // independent of the initial batch fetch below.
    tokio::spawn(retry_worker(state.clone(), retry_rx));
    tokio::spawn(watchdog(state.clone()));

    // Initial concurrent fetch (bounded to 5 at a time by FETCH_SEMAPHORE).
    let mut set = tokio::task::JoinSet::new();
    for artist in artists {
        let app_dir = app_dir.clone();
        set.spawn(async move {
            let result = download_and_save(artist, &app_dir).await;
            (artist.to_string(), result)
        });
    }

    while let Some(res) = set.join_next().await {
        let (artist, result) = res?;
        match result {
            Ok(record) => {
                println!("Fetched image for {artist}: {}", record.filename);
                state.manifest.lock().await.insert(artist, record);
            }
            Err(e) => {
                eprintln!("Error fetching image for {artist}: {e} — queueing retry");
                let _ = state.retry_tx.send(RetryJob { artist, attempt: 0 });
            }
        }
    }

    state.save_manifest().await;
    println!("Initial fetch complete. Watchdog + retry worker still running in the background.");
    println!("Press Ctrl+C to exit.");

    // Background tasks only run as long as the process does — keep it alive.
    tokio::signal::ctrl_c().await?;
    state.save_manifest().await;

    Ok(())
}
