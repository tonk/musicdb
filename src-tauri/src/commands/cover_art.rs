use std::path::PathBuf;

use image::{imageops::FilterType, DynamicImage, ImageFormat};
use tauri::{AppHandle, Manager, State};

use crate::{error::Result, state::AppState};

const THUMB_SIZE: u32 = 500;

#[tauri::command]
pub async fn save_cover_art(
    item_id: i64,
    image_base64: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<String> {
    let bytes = base64_decode(&image_base64)?;
    let img = image::load_from_memory(&bytes)?;
    let thumb = resize_cover(img);

    let covers_dir = covers_dir(&app)?;
    tokio::fs::create_dir_all(&covers_dir).await?;

    let filename = format!("{}.jpg", item_id);
    let filepath = covers_dir.join(&filename);
    thumb.save_with_format(&filepath, ImageFormat::Jpeg)?;

    let abs_path = filepath.to_string_lossy().to_string();
    let db = state.pool().await;
    sqlx::query!(
        "UPDATE items SET cover_art_path=?, updated_at=datetime('now') WHERE id=?",
        abs_path,
        item_id
    )
    .execute(&db)
    .await?;

    Ok(abs_path)
}

#[tauri::command]
pub async fn fetch_caa_cover(
    item_id: i64,
    mbid: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<String> {
    let url = format!("https://coverartarchive.org/release/{}/front", mbid);
    let response = reqwest::get(&url).await?;
    let bytes = response.bytes().await?;

    let img = image::load_from_memory(&bytes)?;
    let thumb = resize_cover(img);

    let covers_dir = covers_dir(&app)?;
    tokio::fs::create_dir_all(&covers_dir).await?;

    let filename = format!("{}.jpg", item_id);
    let filepath = covers_dir.join(&filename);
    thumb.save_with_format(&filepath, ImageFormat::Jpeg)?;

    let abs_path = filepath.to_string_lossy().to_string();
    let db = state.pool().await;
    sqlx::query!(
        "UPDATE items SET cover_art_path=?, updated_at=datetime('now') WHERE id=?",
        abs_path,
        item_id
    )
    .execute(&db)
    .await?;

    Ok(abs_path)
}

fn resize_cover(img: DynamicImage) -> DynamicImage {
    img.resize(THUMB_SIZE, THUMB_SIZE, FilterType::Lanczos3)
}

fn covers_dir(app: &AppHandle) -> Result<PathBuf> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| crate::error::AppError::Parse(e.to_string()))?;
    Ok(data_dir.join("covers"))
}

fn base64_decode(s: &str) -> Result<Vec<u8>> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    // Strip data URI prefix if present: "data:image/...;base64,"
    let b64 = if let Some(pos) = s.find("base64,") {
        &s[pos + 7..]
    } else {
        s
    };
    STANDARD
        .decode(b64)
        .map_err(|e| crate::error::AppError::Parse(e.to_string()))
}
