use tauri::State;

use crate::{
    error::Result,
    models::{artist::Artist, item::ItemSummary},
    state::AppState,
};

#[tauri::command]
pub async fn list_artists(state: State<'_, AppState>) -> Result<Vec<Artist>> {
    let db = state.pool().await;
    let artists = sqlx::query_as!(
        Artist,
        r#"SELECT id as "id!", name as "name!", sort_name as "sort_name!", created_at as "created_at!"
         FROM artists ORDER BY sort_name COLLATE NOCASE"#
    )
    .fetch_all(&db)
    .await?;
    Ok(artists)
}

#[tauri::command]
pub async fn autocomplete_artists(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<Artist>> {
    let db = state.pool().await;
    let pattern = format!("%{}%", query);
    let artists = sqlx::query_as!(
        Artist,
        r#"SELECT id as "id!", name as "name!", sort_name as "sort_name!", created_at as "created_at!"
         FROM artists
         WHERE name LIKE ? OR sort_name LIKE ?
         ORDER BY name COLLATE NOCASE LIMIT 20"#,
        pattern,
        pattern
    )
    .fetch_all(&db)
    .await?;
    Ok(artists)
}

#[tauri::command]
pub async fn create_artist(
    name: String,
    sort_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<Artist> {
    let db = state.pool().await;
    let sname = sort_name.unwrap_or_else(|| name.clone());
    let id: i64 = sqlx::query_scalar!(
        r#"INSERT INTO artists(name, sort_name) VALUES(?,?)
         ON CONFLICT(sort_name) DO UPDATE SET name=excluded.name
         RETURNING id as "id!""#,
        name,
        sname
    )
    .fetch_one(&db)
    .await?;
    let artist = sqlx::query_as!(
        Artist,
        r#"SELECT id as "id!", name as "name!", sort_name as "sort_name!", created_at as "created_at!"
         FROM artists WHERE id = ?"#,
        id
    )
    .fetch_one(&db)
    .await?;
    Ok(artist)
}

#[tauri::command]
pub async fn get_artist_items(
    artist_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<ItemSummary>> {
    let db = state.pool().await;
    let rows = sqlx::query!(
        r#"SELECT i.id as "id!", i.title as "title!", i.format as "format!",
                  i.year, i.label, i.catalogue_number, i.cover_art_path,
                  i.date_added as "date_added!",
                  COALESCE((SELECT GROUP_CONCAT(a2.name, ', ')
                   FROM item_artists ia2 JOIN artists a2 ON a2.id = ia2.artist_id
                   WHERE ia2.item_id = i.id ORDER BY ia2.sort_order), '') as "artist_names!: String"
           FROM items i
           JOIN item_artists ia ON ia.item_id = i.id
           WHERE ia.artist_id = ?
           ORDER BY i.year DESC, i.title COLLATE NOCASE"#,
        artist_id
    )
    .fetch_all(&db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ItemSummary {
            id: r.id,
            title: r.title,
            format: r.format,
            year: r.year,
            label: r.label,
            catalogue_number: r.catalogue_number,
            cover_art_path: r.cover_art_path,
            date_added: r.date_added,
            artist_names: r.artist_names,
        })
        .collect())
}
