use tauri::State;

use crate::{error::Result, models::genre::Genre, state::AppState};

#[tauri::command]
pub async fn list_genres(state: State<'_, AppState>) -> Result<Vec<Genre>> {
    let db = state.pool().await;
    let genres = sqlx::query!(
        r#"SELECT id as "id!", name as "name!" FROM genres ORDER BY name COLLATE NOCASE"#
    )
    .fetch_all(&db)
    .await?
    .into_iter()
    .map(|r| Genre { id: r.id, name: r.name })
    .collect();
    Ok(genres)
}

#[tauri::command]
pub async fn create_genre(name: String, state: State<'_, AppState>) -> Result<Genre> {
    let db = state.pool().await;
    let id: i64 = sqlx::query_scalar!(
        r#"INSERT INTO genres(name) VALUES(?) ON CONFLICT(name) DO UPDATE SET name=excluded.name RETURNING id as "id!""#,
        name
    )
    .fetch_one(&db)
    .await?;
    Ok(Genre { id, name })
}
