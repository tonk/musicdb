use tauri::State;

use crate::{
    error::Result,
    models::{artist::Artist, item::ItemSummary},
    state::AppState,
};

// SQLite NOCASE is not accent-insensitive, so fold common diacritics for
// predictable ordering/search behavior (e.g. Á -> A).
const FOLDED_SORT_NAME_SQL: &str = "LOWER(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(sort_name, 'Á','A'),'á','a'),'À','A'),'à','a'),'Â','A'),'â','a'),'Ä','A'),'ä','a'),'Ã','A'),'ã','a'),'Å','A'),'å','a'),'Æ','AE'),'æ','ae'),'Ç','C'),'ç','c'),'Ð','D'),'ð','d'),'É','E'),'é','e'),'È','E'),'è','e'),'Ê','E'),'ê','e'),'Ë','E'),'ë','e'),'Í','I'),'í','i'),'Ì','I'),'ì','i'),'Î','I'),'î','i'),'Ï','I'),'ï','i'),'Ó','O'),'ó','o'),'Ö','O'),'ö','o'))";

#[tauri::command]
pub async fn list_artists(state: State<'_, AppState>) -> Result<Vec<Artist>> {
    let db = state.pool().await;
    let query = format!(
        r#"SELECT id, name, sort_name, created_at
           FROM artists
           ORDER BY {FOLDED_SORT_NAME_SQL}, sort_name COLLATE NOCASE"#
    );
    let rows = sqlx::query(&query)
    .fetch_all(&db)
    .await?;

    use sqlx::Row;
    Ok(rows
        .into_iter()
        .map(|r| Artist {
            id: r.get("id"),
            name: r.get("name"),
            sort_name: r.get("sort_name"),
            created_at: r.get("created_at"),
        })
        .collect())
}

#[tauri::command]
pub async fn autocomplete_artists(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<Artist>> {
    let db = state.pool().await;
    let pattern = format!("%{}%", query);
    let query = format!(
        r#"SELECT id, name, sort_name, created_at
           FROM artists
           WHERE name LIKE ?1 OR sort_name LIKE ?2
           ORDER BY {FOLDED_SORT_NAME_SQL}, sort_name COLLATE NOCASE
           LIMIT 20"#
    );
    let rows = sqlx::query(&query)
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(&db)
    .await?;

    use sqlx::Row;
    Ok(rows
        .into_iter()
        .map(|r| Artist {
            id: r.get("id"),
            name: r.get("name"),
            sort_name: r.get("sort_name"),
            created_at: r.get("created_at"),
        })
        .collect())
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
