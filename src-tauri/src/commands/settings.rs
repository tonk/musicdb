use tauri::{AppHandle, Manager, State};

use crate::{
    error::{AppError, Result},
    state::{AppState, DatabaseEntry},
};

// ─── Settings table helpers ───────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct SettingEntry {
    pub key: String,
    pub value: Option<String>,
}

#[tauri::command]
pub async fn get_setting(key: String, state: State<'_, AppState>) -> Result<Option<String>> {
    let db = state.pool().await;
    let value: Option<Option<String>> = sqlx::query_scalar!(
        "SELECT value FROM settings WHERE key = ?",
        key
    )
    .fetch_optional(&db)
    .await?;
    Ok(value.flatten())
}

#[tauri::command]
pub async fn set_setting(
    key: String,
    value: Option<String>,
    state: State<'_, AppState>,
) -> Result<()> {
    let db = state.pool().await;
    sqlx::query!(
        "INSERT INTO settings(key, value) VALUES(?,?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        key,
        value
    )
    .execute(&db)
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn get_all_settings(state: State<'_, AppState>) -> Result<Vec<SettingEntry>> {
    let db = state.pool().await;
    let rows = sqlx::query!(
        r#"SELECT key as "key!", value FROM settings ORDER BY key"#
    )
    .fetch_all(&db)
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
    let db = state.pool().await;
    let mut tx = db.begin().await?;
    sqlx::query!("DELETE FROM items").execute(&mut *tx).await?;
    sqlx::query!("DELETE FROM artists").execute(&mut *tx).await?;
    sqlx::query!("DELETE FROM genres").execute(&mut *tx).await?;
    tx.commit().await?;

    *state.undo_buffer.lock().unwrap() = None;

    // Remove cover images for the current database
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
    let db = state.pool().await;
    let rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query("PRAGMA database_list")
        .fetch_all(&db)
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
            .execute(&db)
            .await?;
        }
    }
    Ok(())
}

// ─── Multi-database management ───────────────────────────────────────────────

fn save_db_config(
    app: &AppHandle,
    config: &crate::state::DatabaseConfig,
) -> Result<()> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Parse(e.to_string()))?;
    let cfg_path = data_dir.join("databases.json");
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| AppError::Parse(e.to_string()))?;
    std::fs::write(cfg_path, json)?;
    Ok(())
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[tauri::command]
pub async fn list_databases(state: State<'_, AppState>) -> Result<Vec<DatabaseEntry>> {
    let config = state.db_config.lock().unwrap();
    Ok(config.databases.clone())
}

#[tauri::command]
pub async fn current_database(state: State<'_, AppState>) -> Result<String> {
    let config = state.db_config.lock().unwrap();
    Ok(config.current.clone())
}

#[tauri::command]
pub async fn create_database(
    name: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<Vec<DatabaseEntry>> {
    // Validate name uniqueness
    {
        let config = state.db_config.lock().unwrap();
        if config.databases.iter().any(|db| db.name == name) {
            return Err(AppError::Parse(format!("A database named '{name}' already exists")));
        }
    }

    // Derive path from name
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Parse(e.to_string()))?;
    let filename = format!("{}.sqlite", sanitize_filename(&name));
    let path = data_dir.join(&filename).to_string_lossy().to_string();

    // Create and migrate the new database
    crate::db::init_pool(&path).await?;

    // Add to config
    let mut config = state.db_config.lock().unwrap();
    config.databases.push(DatabaseEntry {
        name,
        path,
    });
    save_db_config(&app, &config)?;
    Ok(config.databases.clone())
}

#[tauri::command]
pub async fn switch_database(
    name: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<()> {
    // Get the path for the target database (brief lock)
    let path = {
        let config = state.db_config.lock().unwrap();
        config
            .databases
            .iter()
            .find(|db| db.name == name)
            .map(|db| db.path.clone())
            .ok_or_else(|| AppError::NotFound(format!("Database '{name}'")))?
    };

    // Open the new pool (no locks held during async I/O)
    let new_pool = crate::db::init_pool(&path).await?;

    // Swap the pool (brief write lock)
    {
        let mut db = state.db.write().await;
        *db = new_pool;
    }

    // Clear undo buffer — it references items from the old database
    *state.undo_buffer.lock().unwrap() = None;

    // Update config
    {
        let mut config = state.db_config.lock().unwrap();
        config.current = name;
        save_db_config(&app, &config)?;
    }

    Ok(())
}

#[tauri::command]
pub async fn rename_database(
    old_name: String,
    new_name: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<Vec<DatabaseEntry>> {
    let mut config = state.db_config.lock().unwrap();

    if !config.databases.iter().any(|db| db.name == old_name) {
        return Err(AppError::NotFound(format!("Database '{old_name}'")));
    }
    if config.databases.iter().any(|db| db.name == new_name) {
        return Err(AppError::Parse(format!("A database named '{new_name}' already exists")));
    }

    for db in config.databases.iter_mut() {
        if db.name == old_name {
            db.name = new_name.clone();
        }
    }
    if config.current == old_name {
        config.current = new_name;
    }
    save_db_config(&app, &config)?;
    Ok(config.databases.clone())
}

#[tauri::command]
pub async fn backup_database(
    dest_path: String,
    state: State<'_, AppState>,
) -> Result<String> {
    let db = state.pool().await;
    let rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query("PRAGMA database_list")
        .fetch_all(&db)
        .await?;

    let src = rows
        .first()
        .and_then(|row| {
            use sqlx::Row;
            row.try_get::<String, _>(2).ok()
        })
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::Parse("Could not determine database path".to_string()))?;

    std::fs::copy(&src, &dest_path)?;
    Ok(dest_path)
}

#[tauri::command]
pub async fn delete_database(
    name: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<Vec<DatabaseEntry>> {
    let mut config = state.db_config.lock().unwrap();

    if config.current == name {
        return Err(AppError::Parse(
            "Cannot delete the active database. Switch to another database first.".to_string(),
        ));
    }

    let path = config
        .databases
        .iter()
        .find(|db| db.name == name)
        .map(|db| db.path.clone());

    config.databases.retain(|db| db.name != name);
    save_db_config(&app, &config)?;

    // Delete the SQLite file (non-fatal if it fails)
    if let Some(p) = path {
        let _ = std::fs::remove_file(&p);
    }

    Ok(config.databases.clone())
}
