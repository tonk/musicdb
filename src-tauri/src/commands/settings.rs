use tauri::{AppHandle, Manager, State};

use crate::{error::Result, state::AppState};

#[derive(serde::Serialize)]
pub struct SettingEntry {
    pub key: String,
    pub value: Option<String>,
}

#[tauri::command]
pub async fn get_setting(key: String, state: State<'_, AppState>) -> Result<Option<String>> {
    let value: Option<Option<String>> = sqlx::query_scalar!(
        "SELECT value FROM settings WHERE key = ?",
        key
    )
    .fetch_optional(&state.db)
    .await?;
    Ok(value.flatten())
}

#[tauri::command]
pub async fn set_setting(
    key: String,
    value: Option<String>,
    state: State<'_, AppState>,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO settings(key, value) VALUES(?,?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        key,
        value
    )
    .execute(&state.db)
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn get_all_settings(state: State<'_, AppState>) -> Result<Vec<SettingEntry>> {
    let rows = sqlx::query!(
        r#"SELECT key as "key!", value FROM settings ORDER BY key"#
    )
    .fetch_all(&state.db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| SettingEntry {
            key: r.key,
            value: r.value,
        })
        .collect())
}

#[tauri::command]
pub async fn reset_database(state: State<'_, AppState>, app: AppHandle) -> Result<()> {
    // Wipe all music data inside a transaction.
    // FK cascades handle tracks, item_artists, item_genres, track_artists.
    // The items_ad trigger removes rows from items_fts on DELETE.
    let mut tx = state.db.begin().await?;
    sqlx::query!("DELETE FROM items").execute(&mut *tx).await?;
    sqlx::query!("DELETE FROM artists").execute(&mut *tx).await?;
    sqlx::query!("DELETE FROM genres").execute(&mut *tx).await?;
    tx.commit().await?;

    // Clear the in-memory undo buffer
    *state.undo_buffer.lock().unwrap() = None;

    // Remove all saved cover images
    if let Ok(data_dir) = app.path().app_data_dir() {
        let covers_dir = data_dir.join("covers");
        if covers_dir.exists() {
            let _ = tokio::fs::remove_dir_all(&covers_dir).await;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn move_database(new_path: String, state: State<'_, AppState>) -> Result<()> {
    let rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query("PRAGMA database_list")
        .fetch_all(&state.db)
        .await?;

    if let Some(row) = rows.first() {
        use sqlx::Row;
        let src: String = row.try_get(2).unwrap_or_default();
        if !src.is_empty() {
            tokio::fs::copy(&src, &new_path).await?;
            sqlx::query!(
                "INSERT INTO settings(key,value) VALUES('db_path',?)
                 ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                new_path
            )
            .execute(&state.db)
            .await?;
        }
    }
    Ok(())
}
