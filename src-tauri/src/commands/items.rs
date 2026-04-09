use tauri::State;

use crate::{
    error::{AppError, Result},
    models::{
        artist::ArtistWithRole,
        genre::Genre,
        item::{
            ArtistRoleInput, CountEntry, CreateItemInput, ItemSummary, ItemWithArtists, ItemsPage,
            ListItemsParams, Statistics, UpdateItemInput,
        },
        track::Track,
    },
    state::{AppState, UndoEntry},
};

// ─── Helpers ──────────────────────────────────────────────────────────────────

async fn fetch_item_with_artists(id: i64, db: &sqlx::SqlitePool) -> Result<ItemWithArtists> {
    let row = sqlx::query!(
        r#"SELECT id as "id!", title as "title!", format as "format!", year,
                  label, publisher, catalogue_number, condition, notes,
                  cover_art_path, disc_id, source_category, musicbrainz_id,
                  date_added as "date_added!", updated_at as "updated_at!"
           FROM items WHERE id = ?"#,
        id
    )
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Item {id}")))?;

    let artists = fetch_item_artists(id, db).await?;
    let genres = fetch_item_genres(id, db).await?;
    let tracks = fetch_item_tracks(id, db).await?;

    Ok(ItemWithArtists {
        id: row.id,
        title: row.title,
        format: row.format,
        year: row.year,
        label: row.label,
        publisher: row.publisher,
        catalogue_number: row.catalogue_number,
        condition: row.condition,
        notes: row.notes,
        cover_art_path: row.cover_art_path,
        disc_id: row.disc_id,
        source_category: row.source_category,
        musicbrainz_id: row.musicbrainz_id,
        date_added: row.date_added,
        updated_at: row.updated_at,
        artists,
        genres,
        tracks,
    })
}

async fn fetch_item_artists(item_id: i64, db: &sqlx::SqlitePool) -> Result<Vec<ArtistWithRole>> {
    let rows = sqlx::query!(
        r#"SELECT a.id as "id!", a.name as "name!", a.sort_name as "sort_name!",
                  ia.role as "role!", ia.sort_order as "sort_order!"
           FROM item_artists ia JOIN artists a ON a.id = ia.artist_id
           WHERE ia.item_id = ? ORDER BY ia.sort_order"#,
        item_id
    )
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ArtistWithRole {
            id: r.id,
            name: r.name,
            sort_name: r.sort_name,
            role: r.role,
            sort_order: r.sort_order,
        })
        .collect())
}

async fn fetch_item_genres(item_id: i64, db: &sqlx::SqlitePool) -> Result<Vec<Genre>> {
    let rows = sqlx::query!(
        r#"SELECT g.id as "id!", g.name as "name!"
           FROM item_genres ig JOIN genres g ON g.id = ig.genre_id
           WHERE ig.item_id = ? ORDER BY g.name"#,
        item_id
    )
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| Genre { id: r.id, name: r.name })
        .collect())
}

async fn fetch_item_tracks(item_id: i64, db: &sqlx::SqlitePool) -> Result<Vec<Track>> {
    let track_rows = sqlx::query!(
        r#"SELECT id as "id!", item_id as "item_id!", disc_id as "disc_id!",
                  track_number as "track_number!", title as "title!",
                  duration_secs, version, sort_order as "sort_order!"
           FROM tracks WHERE item_id = ? ORDER BY sort_order, disc_id, track_number"#,
        item_id
    )
    .fetch_all(db)
    .await?;

    let mut tracks = Vec::with_capacity(track_rows.len());
    for tr in track_rows {
        let artists = sqlx::query!(
            r#"SELECT a.id as "id!", a.name as "name!", a.sort_name as "sort_name!",
                      ta.role as "role!"
               FROM track_artists ta JOIN artists a ON a.id = ta.artist_id
               WHERE ta.track_id = ?"#,
            tr.id
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|r| ArtistWithRole {
            id: r.id,
            name: r.name,
            sort_name: r.sort_name,
            role: r.role,
            sort_order: 0,
        })
        .collect();

        tracks.push(Track {
            id: tr.id,
            item_id: tr.item_id,
            disc_id: tr.disc_id,
            track_number: tr.track_number,
            title: tr.title,
            duration_secs: tr.duration_secs,
            version: tr.version,
            sort_order: tr.sort_order,
            artists,
        });
    }
    Ok(tracks)
}

async fn sync_item_artists(
    item_id: i64,
    artist_ids: &[ArtistRoleInput],
    db: &sqlx::SqlitePool,
) -> Result<()> {
    sqlx::query!("DELETE FROM item_artists WHERE item_id = ?", item_id)
        .execute(db)
        .await?;
    for (i, ar) in artist_ids.iter().enumerate() {
        let ord = i as i64;
        sqlx::query!(
            "INSERT OR IGNORE INTO item_artists(item_id, artist_id, role, sort_order) VALUES(?,?,?,?)",
            item_id,
            ar.artist_id,
            ar.role,
            ord
        )
        .execute(db)
        .await?;
    }
    refresh_fts_artists(item_id, db).await
}

async fn sync_item_genres(
    item_id: i64,
    genre_ids: &[i64],
    db: &sqlx::SqlitePool,
) -> Result<()> {
    sqlx::query!("DELETE FROM item_genres WHERE item_id = ?", item_id)
        .execute(db)
        .await?;
    for gid in genre_ids {
        sqlx::query!(
            "INSERT OR IGNORE INTO item_genres(item_id, genre_id) VALUES(?,?)",
            item_id,
            gid
        )
        .execute(db)
        .await?;
    }
    Ok(())
}

pub async fn refresh_fts_artists(item_id: i64, db: &sqlx::SqlitePool) -> Result<()> {
    let names: Option<String> = sqlx::query_scalar!(
        r#"SELECT GROUP_CONCAT(a.name, ' ') as "names: String"
           FROM item_artists ia JOIN artists a ON a.id = ia.artist_id
           WHERE ia.item_id = ? ORDER BY ia.sort_order"#,
        item_id
    )
    .fetch_one(db)
    .await?;

    let names = names.unwrap_or_default();
    sqlx::query!(
        "INSERT INTO items_fts(items_fts, rowid, title, artist_names, label, publisher, catalogue_number, notes)
         SELECT 'delete', id, title, ?, label, publisher, catalogue_number, notes FROM items WHERE id = ?",
        names,
        item_id
    )
    .execute(db)
    .await?;
    sqlx::query!(
        "INSERT INTO items_fts(rowid, title, artist_names, label, publisher, catalogue_number, notes)
         SELECT id, title, ?, label, publisher, catalogue_number, notes FROM items WHERE id = ?",
        names,
        item_id
    )
    .execute(db)
    .await?;
    Ok(())
}

// ─── Commands ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_item(id: i64, state: State<'_, AppState>) -> Result<ItemWithArtists> {
    fetch_item_with_artists(id, &state.db).await
}

#[tauri::command]
pub async fn create_item(
    input: CreateItemInput,
    state: State<'_, AppState>,
) -> Result<ItemWithArtists> {
    let id: i64 = sqlx::query_scalar!(
        r#"INSERT INTO items(title, format, year, label, publisher, catalogue_number,
                           condition, notes, musicbrainz_id)
         VALUES(?,?,?,?,?,?,?,?,?) RETURNING id as "id!""#,
        input.title,
        input.format,
        input.year,
        input.label,
        input.publisher,
        input.catalogue_number,
        input.condition,
        input.notes,
        input.musicbrainz_id,
    )
    .fetch_one(&state.db)
    .await?;

    sync_item_artists(id, &input.artist_ids, &state.db).await?;
    sync_item_genres(id, &input.genre_ids, &state.db).await?;

    fetch_item_with_artists(id, &state.db).await
}

#[tauri::command]
pub async fn update_item(
    id: i64,
    input: UpdateItemInput,
    state: State<'_, AppState>,
) -> Result<ItemWithArtists> {
    let row = sqlx::query!(
        r#"SELECT id as "id!", title as "title!", format as "format!", year,
                  label, publisher, catalogue_number, condition, notes,
                  cover_art_path, disc_id, source_category, musicbrainz_id,
                  date_added as "date_added!", updated_at as "updated_at!"
           FROM items WHERE id = ?"#,
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Item {id}")))?;

    let title = input.title.unwrap_or(row.title);
    let format = input.format.unwrap_or(row.format);
    let year = input.year.or(row.year);
    let label = input.label.or(row.label);
    let publisher = input.publisher.or(row.publisher);
    let catalogue_number = input.catalogue_number.or(row.catalogue_number);
    let condition = input.condition.or(row.condition);
    let notes = input.notes.or(row.notes);
    let musicbrainz_id = input.musicbrainz_id.or(row.musicbrainz_id);

    sqlx::query!(
        "UPDATE items SET title=?, format=?, year=?, label=?, publisher=?,
                          catalogue_number=?, condition=?, notes=?, musicbrainz_id=?,
                          updated_at=datetime('now')
         WHERE id=?",
        title,
        format,
        year,
        label,
        publisher,
        catalogue_number,
        condition,
        notes,
        musicbrainz_id,
        id,
    )
    .execute(&state.db)
    .await?;

    if let Some(artist_ids) = input.artist_ids {
        sync_item_artists(id, &artist_ids, &state.db).await?;
    }
    if let Some(genre_ids) = input.genre_ids {
        sync_item_genres(id, &genre_ids, &state.db).await?;
    }

    fetch_item_with_artists(id, &state.db).await
}

#[tauri::command]
pub async fn delete_item(id: i64, state: State<'_, AppState>) -> Result<()> {
    let item = fetch_item_with_artists(id, &state.db).await?;
    {
        let mut buf = state.undo_buffer.lock().unwrap();
        *buf = Some(UndoEntry { item });
    }
    sqlx::query!("DELETE FROM items WHERE id = ?", id)
        .execute(&state.db)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn undo_delete(state: State<'_, AppState>) -> Result<Option<ItemWithArtists>> {
    let entry = {
        let mut buf = state.undo_buffer.lock().unwrap();
        buf.take()
    };

    if let Some(e) = entry {
        let item = e.item;
        let id: i64 = sqlx::query_scalar!(
            r#"INSERT INTO items(title, format, year, label, publisher, catalogue_number,
                               condition, notes, cover_art_path, disc_id, source_category,
                               musicbrainz_id, date_added, updated_at)
             VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?) RETURNING id as "id!""#,
            item.title,
            item.format,
            item.year,
            item.label,
            item.publisher,
            item.catalogue_number,
            item.condition,
            item.notes,
            item.cover_art_path,
            item.disc_id,
            item.source_category,
            item.musicbrainz_id,
            item.date_added,
            item.updated_at,
        )
        .fetch_one(&state.db)
        .await?;

        let artist_inputs: Vec<ArtistRoleInput> = item
            .artists
            .iter()
            .map(|a| ArtistRoleInput {
                artist_id: a.id,
                role: a.role.clone(),
            })
            .collect();
        sync_item_artists(id, &artist_inputs, &state.db).await?;

        let genre_ids: Vec<i64> = item.genres.iter().map(|g| g.id).collect();
        sync_item_genres(id, &genre_ids, &state.db).await?;

        Ok(Some(fetch_item_with_artists(id, &state.db).await?))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn list_items(
    params: ListItemsParams,
    state: State<'_, AppState>,
) -> Result<ItemsPage> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(50).clamp(1, 500);
    let offset = (page - 1) * page_size;

    let sort_col = match params.sort_field.as_deref() {
        Some("title") => "i.title COLLATE NOCASE",
        Some("year") => "i.year",
        Some("format") => "i.format",
        Some("label") => "i.label COLLATE NOCASE",
        Some("catalogue_number") => "i.catalogue_number COLLATE NOCASE",
        _ => "i.date_added",
    };
    let sort_dir = if params.sort_dir.as_deref() == Some("asc") { "ASC" } else { "DESC" };

    let mut conditions = vec!["1=1".to_string()];
    if let Some(f) = &params.format {
        conditions.push(format!("i.format = '{}'", f.replace('\'', "''")));
    }
    if let Some(c) = &params.condition {
        conditions.push(format!("i.condition = '{}'", c.replace('\'', "''")));
    }
    if let Some(y) = params.year_from {
        conditions.push(format!("i.year >= {y}"));
    }
    if let Some(y) = params.year_to {
        conditions.push(format!("i.year <= {y}"));
    }
    if let Some(gid) = params.genre_id {
        conditions.push(format!(
            "EXISTS(SELECT 1 FROM item_genres ig WHERE ig.item_id=i.id AND ig.genre_id={gid})"
        ));
    }
    let where_clause = conditions.join(" AND ");

    let query_str = format!(
        r#"SELECT i.id, i.title, i.format, i.year, i.label, i.catalogue_number,
                  i.cover_art_path, i.date_added,
                  COALESCE((SELECT GROUP_CONCAT(a.name, ', ')
                             FROM item_artists ia JOIN artists a ON a.id=ia.artist_id
                             WHERE ia.item_id=i.id ORDER BY ia.sort_order), '') AS artist_names
           FROM items i
           WHERE {where_clause}
           ORDER BY {sort_col} {sort_dir}
           LIMIT {page_size} OFFSET {offset}"#
    );
    let count_str = format!("SELECT COUNT(*) FROM items i WHERE {where_clause}");

    let total: i64 = sqlx::query_scalar(&count_str)
        .fetch_one(&state.db)
        .await?;

    let rows = sqlx::query(&query_str).fetch_all(&state.db).await?;

    use sqlx::Row;
    let items = rows
        .into_iter()
        .map(|r| ItemSummary {
            id: r.get("id"),
            title: r.get("title"),
            format: r.get("format"),
            year: r.get("year"),
            label: r.get("label"),
            catalogue_number: r.get("catalogue_number"),
            cover_art_path: r.get("cover_art_path"),
            date_added: r.get("date_added"),
            artist_names: r.get::<Option<String>, _>("artist_names").unwrap_or_default(),
        })
        .collect();

    Ok(ItemsPage { items, total, page, page_size })
}

#[tauri::command]
pub async fn search_items(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<ItemSummary>> {
    let fts_query = format!("{}*", query.replace('"', ""));
    let rows = sqlx::query!(
        r#"SELECT i.id as "id!", i.title as "title!", i.format as "format!",
                  i.year, i.label, i.catalogue_number, i.cover_art_path,
                  i.date_added as "date_added!",
                  COALESCE((SELECT GROUP_CONCAT(a.name, ', ')
                             FROM item_artists ia JOIN artists a ON a.id=ia.artist_id
                             WHERE ia.item_id=i.id ORDER BY ia.sort_order), '') AS "artist_names!: String"
           FROM items_fts f
           JOIN items i ON i.id = f.rowid
           WHERE items_fts MATCH ?
           ORDER BY rank
           LIMIT 100"#,
        fts_query
    )
    .fetch_all(&state.db)
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

#[tauri::command]
pub async fn get_statistics(state: State<'_, AppState>) -> Result<Statistics> {
    let total_items: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM items")
        .fetch_one(&state.db)
        .await?;

    let by_format_rows = sqlx::query(
        "SELECT format, COUNT(*) as cnt FROM items GROUP BY format ORDER BY 2 DESC"
    )
    .fetch_all(&state.db)
    .await?;
    use sqlx::Row;
    let by_format: Vec<CountEntry> = by_format_rows
        .into_iter()
        .map(|r| CountEntry {
            label: r.get::<Option<String>, _>("format").unwrap_or_default(),
            count: r.get::<i64, _>("cnt"),
        })
        .collect();

    let by_genre_rows = sqlx::query(
        "SELECT g.name, COUNT(ig.item_id) as cnt
         FROM genres g JOIN item_genres ig ON ig.genre_id=g.id
         GROUP BY g.id ORDER BY 2 DESC LIMIT 20"
    )
    .fetch_all(&state.db)
    .await?;
    let by_genre: Vec<CountEntry> = by_genre_rows
        .into_iter()
        .map(|r| CountEntry {
            label: r.get::<Option<String>, _>("name").unwrap_or_default(),
            count: r.get::<i64, _>("cnt"),
        })
        .collect();

    let by_year_rows = sqlx::query(
        "SELECT CAST(year AS TEXT) as yr, COUNT(*) as cnt
         FROM items WHERE year IS NOT NULL GROUP BY year ORDER BY year DESC LIMIT 50"
    )
    .fetch_all(&state.db)
    .await?;
    let by_year: Vec<CountEntry> = by_year_rows
        .into_iter()
        .map(|r| CountEntry {
            label: r.get::<Option<String>, _>("yr").unwrap_or_default(),
            count: r.get::<i64, _>("cnt"),
        })
        .collect();

    Ok(Statistics { total_items, by_format, by_genre, by_year })
}
