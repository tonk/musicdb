use tauri::State;

use crate::{error::Result, state::AppState};

#[tauri::command]
pub async fn export_csv(path: String, state: State<'_, AppState>) -> Result<()> {
    let db = state.pool().await;
    let rows = sqlx::query!(
        r#"SELECT i.id as "id!", i.title as "title!", i.format as "format!",
                  i.year, i.label, i.publisher, i.catalogue_number, i.condition,
                  i.notes, i.disc_id, i.source_category, i.musicbrainz_id,
                  i.date_added as "date_added!",
                  COALESCE((SELECT GROUP_CONCAT(a.name, '; ')
                             FROM item_artists ia JOIN artists a ON a.id=ia.artist_id
                             WHERE ia.item_id=i.id ORDER BY ia.sort_order), '') AS "artist_names!",
                  COALESCE((SELECT GROUP_CONCAT(g.name, '; ')
                             FROM item_genres ig JOIN genres g ON g.id=ig.genre_id
                             WHERE ig.item_id=i.id), '') AS "genre_names!"
           FROM items i ORDER BY i.title COLLATE NOCASE"#
    )
    .fetch_all(&db)
    .await?;

    let mut wtr = csv::Writer::from_path(&path)?;
    wtr.write_record([
        "id",
        "title",
        "artist",
        "format",
        "year",
        "label",
        "publisher",
        "catalogue_number",
        "condition",
        "genres",
        "notes",
        "disc_id",
        "source_category",
        "musicbrainz_id",
        "date_added",
    ])?;

    for r in rows {
        wtr.write_record([
            r.id.to_string(),
            r.title,
            r.artist_names,
            r.format,
            r.year.map(|y| y.to_string()).unwrap_or_default(),
            r.label.unwrap_or_default(),
            r.publisher.unwrap_or_default(),
            r.catalogue_number.unwrap_or_default(),
            r.condition.unwrap_or_default(),
            r.genre_names,
            r.notes.unwrap_or_default(),
            r.disc_id.unwrap_or_default(),
            r.source_category.unwrap_or_default(),
            r.musicbrainz_id.unwrap_or_default(),
            r.date_added,
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

#[tauri::command]
pub async fn export_json(path: String, state: State<'_, AppState>) -> Result<()> {
    let db = state.pool().await;
    let rows = sqlx::query!(
        r#"SELECT i.id as "id!", i.title as "title!", i.format as "format!",
                  i.year, i.label, i.publisher, i.catalogue_number, i.condition,
                  i.notes, i.disc_id, i.source_category, i.musicbrainz_id,
                  i.date_added as "date_added!", i.updated_at as "updated_at!"
           FROM items i ORDER BY i.title COLLATE NOCASE"#
    )
    .fetch_all(&db)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let artists = sqlx::query!(
            "SELECT a.name, ia.role FROM item_artists ia JOIN artists a ON a.id=ia.artist_id
             WHERE ia.item_id=? ORDER BY ia.sort_order",
            r.id
        )
        .fetch_all(&db)
        .await?
        .into_iter()
        .map(|a| serde_json::json!({"name": a.name, "role": a.role}))
        .collect::<Vec<_>>();

        let genres = sqlx::query_scalar!(
            "SELECT g.name FROM item_genres ig JOIN genres g ON g.id=ig.genre_id WHERE ig.item_id=?",
            r.id
        )
        .fetch_all(&db)
        .await?;

        out.push(serde_json::json!({
            "id": r.id,
            "title": r.title,
            "format": r.format,
            "year": r.year,
            "label": r.label,
            "publisher": r.publisher,
            "catalogue_number": r.catalogue_number,
            "condition": r.condition,
            "notes": r.notes,
            "disc_id": r.disc_id,
            "source_category": r.source_category,
            "musicbrainz_id": r.musicbrainz_id,
            "date_added": r.date_added,
            "updated_at": r.updated_at,
            "artists": artists,
            "genres": genres,
        }));
    }

    let json = serde_json::to_string_pretty(&out)
        .map_err(|e| crate::error::AppError::Parse(e.to_string()))?;
    tokio::fs::write(&path, json).await?;
    Ok(())
}
