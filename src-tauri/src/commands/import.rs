use std::collections::{BTreeMap, HashSet};

use tauri::{Emitter, State};

use crate::{
    error::{AppError, Result},
    models::import::{CsvColumnMapping, CsvPreview, ImportAlbum, ImportSummary, TxtRecord},
    state::AppState,
};

// ─── TXT Parser ───────────────────────────────────────────────────────────────

fn parse_field_line(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    if !line.starts_with('[') {
        return None;
    }
    let close = line.find(']')?;
    let key = &line[1..close];
    if key == "Einde Record" {
        return None;
    }
    let rest = &line[close + 1..];
    let eq = rest.find('=')?;
    let val = rest[eq + 1..].trim();
    Some((key, val))
}

fn parse_single_record(chunk: &str) -> Option<TxtRecord> {
    let mut rec = TxtRecord::default();
    let mut has_nummer = false;

    for line in chunk.lines() {
        if let Some((key, val)) = parse_field_line(line) {
            match key.trim() {
                "Nummer" => {
                    rec.nummer = val.to_owned();
                    has_nummer = true;
                    if let Some(dot) = val.rfind('.') {
                        rec.disc_id = val[..dot].to_owned();
                        rec.track_num = val[dot + 1..].to_owned();
                    } else {
                        rec.disc_id = val.to_owned();
                        rec.track_num = "01".to_owned();
                    }
                }
                "Titel" => rec.titel = val.to_owned(),
                "Artiest" => rec.artiest = val.to_owned(),
                "Label" => rec.label = val.to_owned(),
                "Uitgever" => rec.uitgever = val.to_owned(),
                "Tijd" => rec.tijd = val.to_owned(),
                "Category" => rec.category = val.to_owned(),
                "Band" => {} // ignored — always "N"
                "Catalogus" => {
                    let s = val.trim().to_owned();
                    if !s.is_empty() {
                        rec.catalogus = Some(s);
                    }
                }
                "Drager" => {
                    let s = val.trim().to_owned();
                    if !s.is_empty() {
                        rec.drager = Some(s);
                    }
                }
                "Jaar" => rec.jaar = val.trim().parse().ok(),
                "Arrangeur" => {
                    let s = val.trim().to_owned();
                    if !s.is_empty() {
                        rec.arrangeur = Some(s);
                    }
                }
                "Componist1" => {
                    let s = val.trim().to_owned();
                    if !s.is_empty() {
                        rec.componist1 = Some(s);
                    }
                }
                "Versie" => {
                    let s = val.trim().to_owned();
                    if !s.is_empty() {
                        rec.versie = Some(s);
                    }
                }
                _ => {}
            }
        }
    }

    if has_nummer { Some(rec) } else { None }
}

pub fn parse_txt_records(content: &str) -> Vec<TxtRecord> {
    content
        .split("[Einde Record]")
        .filter_map(parse_single_record)
        .collect()
}

pub fn group_into_albums(mut records: Vec<TxtRecord>) -> Vec<ImportAlbum> {
    // Group by catalogue number (if present) else by disc_id
    let mut map: BTreeMap<String, Vec<TxtRecord>> = BTreeMap::new();
    for rec in records.drain(..) {
        let key = match &rec.catalogus {
            Some(c) if !c.trim().is_empty() => c.trim().to_owned(),
            _ => rec.disc_id.clone(),
        };
        map.entry(key).or_default().push(rec);
    }

    map.into_values()
        .map(|mut tracks| {
            // Sort by disc_id, then numeric track_num
            tracks.sort_by(|a, b| {
                a.disc_id.cmp(&b.disc_id).then_with(|| {
                    let na: u32 = a.track_num.parse().unwrap_or(0);
                    let nb: u32 = b.track_num.parse().unwrap_or(0);
                    na.cmp(&nb)
                })
            });

            let first = &tracks[0];

            // Collect all unique artist sort_names across the album
            let mut seen: HashSet<String> = HashSet::new();
            let mut artist_sort_names: Vec<String> = Vec::new();
            for t in &tracks {
                for name in [
                    t.artiest.as_str(),
                    t.componist1.as_deref().unwrap_or(""),
                    t.arrangeur.as_deref().unwrap_or(""),
                ] {
                    let n = name.trim().to_owned();
                    if !n.is_empty() && seen.insert(n.clone()) {
                        artist_sort_names.push(n);
                    }
                }
            }

            ImportAlbum {
                disc_id: first.disc_id.clone(),
                title: first
                    .drager
                    .clone()
                    .unwrap_or_else(|| first.disc_id.clone()),
                catalogue_number: first.catalogus.clone(),
                label: first.label.clone(),
                publisher: first.uitgever.clone(),
                year: first.jaar,
                category: first.category.clone(),
                artist_sort_names,
                tracks,
            }
        })
        .collect()
}

fn sort_name_to_display(sort_name: &str) -> String {
    let parts: Vec<&str> = sort_name.splitn(2, ',').collect();
    match parts.as_slice() {
        [last, first] => {
            let first = first.trim();
            let last = last.trim();
            if first.is_empty() {
                last.to_owned()
            } else {
                format!("{} {}", capitalize(first), capitalize(last))
            }
        }
        _ => capitalize(sort_name),
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().to_string() + c.as_str(),
    }
}

fn parse_duration(s: &str) -> Option<i64> {
    // "005:35" → 335
    let parts: Vec<&str> = s.trim().splitn(2, ':').collect();
    match parts.as_slice() {
        [m, sec] => {
            let mins: i64 = m.trim().parse().ok()?;
            let secs: i64 = sec.trim().parse().ok()?;
            Some(mins * 60 + secs)
        }
        _ => None,
    }
}

fn track_credits(t: &TxtRecord) -> Vec<(String, &'static str)> {
    let mut credits = Vec::new();
    let artiest = t.artiest.trim().to_owned();
    if !artiest.is_empty() {
        credits.push((artiest, "artist"));
    }
    if let Some(c) = &t.componist1 {
        let c = c.trim().to_owned();
        if !c.is_empty() {
            credits.push((c, "composer"));
        }
    }
    if let Some(a) = &t.arrangeur {
        let a = a.trim().to_owned();
        if !a.is_empty() {
            credits.push((a, "arranger"));
        }
    }
    credits
}

// ─── DB Upsert ────────────────────────────────────────────────────────────────

async fn upsert_album(db: &sqlx::SqlitePool, album: &ImportAlbum) -> Result<i64> {
    let mut tx = db.begin().await?;

    // Upsert item — conflict on (catalogue_number, disc_id) when catalogue is set,
    // else just insert (disc_id alone has no unique constraint).
    // Find existing item by catalogue_number+disc_id (when present) or disc_id alone
    let existing_id: Option<i64> = if let Some(cat) = &album.catalogue_number {
        sqlx::query_scalar!(
            r#"SELECT id as "id!" FROM items
               WHERE catalogue_number = ? AND disc_id = ?"#,
            cat,
            album.disc_id
        )
        .fetch_optional(&mut *tx)
        .await?
    } else {
        sqlx::query_scalar!(
            r#"SELECT id as "id!" FROM items
               WHERE disc_id = ? AND catalogue_number IS NULL"#,
            album.disc_id
        )
        .fetch_optional(&mut *tx)
        .await?
    };

    let item_id: i64 = if let Some(id) = existing_id {
        sqlx::query!(
            "UPDATE items SET updated_at=datetime('now') WHERE id=?",
            id
        )
        .execute(&mut *tx)
        .await?;
        id
    } else {
        sqlx::query_scalar!(
            r#"INSERT INTO items(title, format, year, label, publisher, catalogue_number,
                                  disc_id, source_category)
               VALUES (?, 'CD', ?, ?, ?, ?, ?, ?) RETURNING id as "id!""#,
            album.title,
            album.year,
            album.label,
            album.publisher,
            album.catalogue_number,
            album.disc_id,
            album.category,
        )
        .fetch_one(&mut *tx)
        .await?
    };

    // Upsert artists and link to item
    for sort_name in &album.artist_sort_names {
        let display = sort_name_to_display(sort_name);
        let artist_id: i64 = sqlx::query_scalar!(
            r#"INSERT INTO artists(name, sort_name) VALUES(?,?)
             ON CONFLICT(sort_name) DO UPDATE SET name=excluded.name
             RETURNING id as "id!""#,
            display,
            sort_name
        )
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query!(
            "INSERT OR IGNORE INTO item_artists(item_id, artist_id, role, sort_order)
             VALUES(?,?,'artist',0)",
            item_id,
            artist_id
        )
        .execute(&mut *tx)
        .await?;
    }

    // Delete old tracks and re-insert (idempotent)
    sqlx::query!("DELETE FROM tracks WHERE item_id = ?", item_id)
        .execute(&mut *tx)
        .await?;

    for (ord, t) in album.tracks.iter().enumerate() {
        let duration_secs = parse_duration(&t.tijd);
        let sort_order = ord as i64;

        let track_id: i64 = sqlx::query_scalar!(
            r#"INSERT INTO tracks(item_id, disc_id, track_number, title,
                                duration_secs, version, sort_order)
             VALUES(?,?,?,?,?,?,?) RETURNING id as "id!""#,
            item_id,
            t.disc_id,
            t.track_num,
            t.titel,
            duration_secs,
            t.versie,
            sort_order,
        )
        .fetch_one(&mut *tx)
        .await?;

        for (sort_name, role) in track_credits(t) {
            let display = sort_name_to_display(&sort_name);
            let artist_id: i64 = sqlx::query_scalar!(
                r#"INSERT INTO artists(name, sort_name) VALUES(?,?)
                 ON CONFLICT(sort_name) DO UPDATE SET name=excluded.name
                 RETURNING id as "id!""#,
                display,
                sort_name
            )
            .fetch_one(&mut *tx)
            .await?;

            sqlx::query!(
                "INSERT OR IGNORE INTO track_artists(track_id, artist_id, role)
                 VALUES(?,?,?)",
                track_id,
                artist_id,
                role,
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(item_id)
}

// ─── Commands ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn import_txt_file(
    path: String,
    state: State<'_, AppState>,
    window: tauri::Window,
) -> Result<ImportSummary> {
    let content = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| AppError::Parse(format!("Cannot read file: {e}")))?;

    let records = parse_txt_records(&content);
    let albums = group_into_albums(records);
    let total = albums.len();
    let mut imported = 0usize;
    let mut skipped = 0usize;

    for (i, album) in albums.iter().enumerate() {
        if i % 10 == 0 {
            let _ = window.emit(
                "import-progress",
                serde_json::json!({
                    "done": i,
                    "total": total,
                    "current": &album.title,
                }),
            );
        }

        match upsert_album(&state.db, album).await {
            Ok(_) => imported += 1,
            Err(e) => {
                eprintln!("Import skip {}: {e}", album.disc_id);
                skipped += 1;
            }
        }
    }

    // Emit final progress
    let _ = window.emit(
        "import-progress",
        serde_json::json!({"done": total, "total": total, "current": ""}),
    );

    Ok(ImportSummary {
        total,
        imported,
        skipped,
    })
}

#[tauri::command]
pub async fn preview_csv(path: String) -> Result<CsvPreview> {
    let mut rdr = csv::Reader::from_path(&path)?;
    let headers = rdr
        .headers()
        .map_err(|e| AppError::Parse(e.to_string()))?
        .iter()
        .map(|s| s.to_owned())
        .collect();

    let rows: Vec<Vec<String>> = rdr
        .records()
        .take(5)
        .filter_map(|r| r.ok())
        .map(|r| r.iter().map(|s| s.to_owned()).collect())
        .collect();

    Ok(CsvPreview { headers, rows })
}

#[tauri::command]
pub async fn import_csv(
    path: String,
    mapping: Vec<CsvColumnMapping>,
    state: State<'_, AppState>,
) -> Result<ImportSummary> {
    let mut rdr = csv::Reader::from_path(&path)?;
    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| AppError::Parse(e.to_string()))?
        .iter()
        .map(|s| s.to_owned())
        .collect();

    // Build column index → field name map
    let col_map: std::collections::HashMap<usize, &str> = mapping
        .iter()
        .filter_map(|m| {
            headers
                .iter()
                .position(|h| h == &m.csv_column)
                .map(|idx| (idx, m.item_field.as_str()))
        })
        .collect();

    let mut total = 0usize;
    let mut imported = 0usize;
    let mut skipped = 0usize;

    for result in rdr.records() {
        let record = result.map_err(|e| AppError::Parse(e.to_string()))?;
        total += 1;

        let mut title = String::new();
        let mut format = "Other".to_string();
        let mut year: Option<i64> = None;
        let mut label: Option<String> = None;
        let mut publisher: Option<String> = None;
        let mut catalogue_number: Option<String> = None;
        let mut condition: Option<String> = None;
        let mut notes: Option<String> = None;
        let mut artist_name: Option<String> = None;

        for (idx, field) in record.iter().enumerate() {
            let val = field.trim();
            if val.is_empty() {
                continue;
            }
            match col_map.get(&idx).copied() {
                Some("title") => title = val.to_owned(),
                Some("format") => format = val.to_owned(),
                Some("year") => year = val.parse().ok(),
                Some("label") => label = Some(val.to_owned()),
                Some("publisher") => publisher = Some(val.to_owned()),
                Some("catalogue_number") => catalogue_number = Some(val.to_owned()),
                Some("condition") => condition = Some(val.to_owned()),
                Some("notes") => notes = Some(val.to_owned()),
                Some("artist") => artist_name = Some(val.to_owned()),
                _ => {}
            }
        }

        if title.is_empty() {
            skipped += 1;
            continue;
        }

        let result: Result<()> = async {
            let item_id: i64 = sqlx::query_scalar!(
                r#"INSERT INTO items(title, format, year, label, publisher, catalogue_number,
                                   condition, notes)
                 VALUES(?,?,?,?,?,?,?,?) RETURNING id as "id!""#,
                title,
                format,
                year,
                label,
                publisher,
                catalogue_number,
                condition,
                notes,
            )
            .fetch_one(&state.db)
            .await?;

            if let Some(name) = artist_name {
                let sort_name = name.clone();
                let artist_id: i64 = sqlx::query_scalar!(
                    r#"INSERT INTO artists(name, sort_name) VALUES(?,?)
                     ON CONFLICT(sort_name) DO UPDATE SET name=excluded.name
                     RETURNING id as "id!""#,
                    name,
                    sort_name,
                )
                .fetch_one(&state.db)
                .await?;
                sqlx::query!(
                    "INSERT OR IGNORE INTO item_artists(item_id, artist_id, role, sort_order)
                     VALUES(?,?,'artist',0)",
                    item_id,
                    artist_id,
                )
                .execute(&state.db)
                .await?;
            }
            Ok(())
        }
        .await;

        match result {
            Ok(_) => imported += 1,
            Err(e) => {
                eprintln!("CSV import skip row {total}: {e}");
                skipped += 1;
            }
        }
    }

    Ok(ImportSummary {
        total,
        imported,
        skipped,
    })
}
