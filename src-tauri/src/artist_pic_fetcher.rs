use crate::db;
use crate::scanner::ScanProgress;
use futures::stream::{self, StreamExt};
use primp::{Client, Impersonate};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .to_lowercase()
}

async fn get_lastfm_image_url(
    client: &Client,
    artist: &str,
) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
    let encoded_name = urlencoding::encode(artist);
    let target_url = format!("https://www.last.fm/music/{}/+images", encoded_name);

    let response = client
        .get(&target_url)
        .timeout(Duration::from_secs(10))
        .send()
        .await?;
    if !response.status().is_success() {
        return Ok(None);
    }

    let html_content = response.text().await?;
    let document = Html::parse_document(&html_content);
    let image_list_selector = Selector::parse("ul.image-list").unwrap();
    let img_tag_selector = Selector::parse("img").unwrap();

    if let Some(list_element) = document.select(&image_list_selector).next() {
        if let Some(img_element) = list_element.select(&img_tag_selector).next() {
            if let Some(src) = img_element.value().attr("src") {
                let high_res_url = src.replace("avatar170s", "770x0").replace("180s", "770x0");
                return Ok(Some(high_res_url));
            }
        }
    }

    Ok(None)
}

async fn process_artist_image(
    client: &Client,
    artist: &str,
    images_dir: &Path,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let Some(img_url) = get_lastfm_image_url(client, artist).await? else {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("No image found for: {}", artist),
        )));
    };

    let image_bytes = client
        .get(&img_url)
        .timeout(Duration::from_secs(15))
        .send()
        .await?
        .bytes()
        .await?;

    let artist = artist.to_string();
    let images_dir = images_dir.to_path_buf();

    let result =
        tokio::task::spawn_blocking(move || -> Result<String, Box<dyn Error + Send + Sync>> {
            let img = image::load_from_memory(&image_bytes)?;

            let encoder = webp::Encoder::from_image(&img)
                .map_err(|e| format!("Failed to create WebP encoder: {e}"))?;
            let webp_data = encoder.encode(80.0).to_vec();

            let filename = format!("{}.webp", sanitize_filename(&artist));
            let output_path = images_dir.join(&filename);
            std::fs::write(&output_path, &webp_data)?;

            println!("✅ Successfully saved: {} -> {:?}", artist, output_path);
            Ok(filename)
        })
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)??;

    Ok(result)
}

pub async fn fetch_artist_images(
    artists: &HashMap<i64, String>,
    app_dir: &Path,
    pool: Pool<SqliteConnectionManager>,
    app_handle: &AppHandle,
) -> Result<(), Box<dyn Error>> {
    let images_dir = app_dir.join("artists");
    tokio::fs::create_dir_all(&images_dir).await?;

    let client = Client::builder().impersonate(Impersonate::Random).build()?;

    let max_concurrent_downloads = 15;
    let total = artists.len();
    let completed = Arc::new(AtomicUsize::new(0));

    stream::iter(artists.iter().map(|(&id, name)| (id, name.clone())))
        .for_each_concurrent(
            max_concurrent_downloads,
            {
                let client = client.clone();
                let pool = pool.clone();
                let completed = Arc::clone(&completed);
                let images_dir = images_dir.clone();
                let app_handle = app_handle.clone();
                move |(artist_id, artist_name)| {
                    let client = client.clone();
                    let pool = pool.clone();
                    let images_dir = images_dir.clone();
                    let completed = Arc::clone(&completed);
                    let app_handle = app_handle.clone();
                    async move {
                        if artist_name.is_empty() || artist_name == "Unknown Artist" {
                            let idx = completed.fetch_add(1, Ordering::SeqCst) + 1;
                            let _ = app_handle.emit(
                                "fetch-progress",
                                ScanProgress {
                                    current: idx,
                                    total,
                                    message: format!("Fetching artist image: {artist_name}"),
                                },
                            );
                            return;
                        }
                        match process_artist_image(&client, &artist_name, &images_dir).await {
                            Ok(filename) => {
                                if !filename.is_empty() {
                                    if let Ok(conn) = pool.get() {
                                       let _ = conn.execute(
                                            "UPDATE artist SET profile_image = ?, banner_image = ? WHERE id = ?",
                                            rusqlite::params![filename, filename, artist_id],
                                        );
                                        let _ = db::report_fetch_success(&conn, artist_id);
                                    }
                                }
                            }
                            Err(e) => {
                                println!("❌ Error processing {}: {}", artist_name, e);
                                if let Ok(conn) = pool.get() {
                                    let _ = db::report_fetch_failure(&conn, artist_id);
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
            },
        )
        .await;
    Ok(())
}
