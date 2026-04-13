use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use image::ImageFormat;
use lofty::{
    picture::PictureType,
    prelude::{AudioFile, TaggedFileExt},
    probe::Probe,
    tag::{Accessor, ItemKey},
};
use tauri::{AppHandle, Emitter, Manager, State};
use walkdir::WalkDir;

use crate::{
    error::{AppError, Result},
    models::import::{
        AudioImportSummary, CsvColumnMapping, CsvPreview, ImportAlbum, ImportSummary, TxtRecord,
    },
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

            let total_secs: i64 = tracks.iter()
                .filter_map(|t| parse_duration(&t.tijd))
                .sum();
            let total_time = if total_secs > 0 {
                Some(format_duration_secs(total_secs))
            } else {
                None
            };

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
                total_time,
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

fn format_duration_secs(total: i64) -> String {
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
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
                                  disc_id, source_category, total_time)
               VALUES (?, 'CD', ?, ?, ?, ?, ?, ?, ?) RETURNING id as "id!""#,
            album.title,
            album.year,
            album.label,
            album.publisher,
            album.catalogue_number,
            album.disc_id,
            album.category,
            album.total_time,
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
        let mut total_time: Option<String> = None;
        let mut archive_number: Option<String> = None;

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
                Some("total_time") => total_time = Some(val.to_owned()),
                Some("archive_number") => archive_number = Some(val.to_owned()),
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
                                   condition, notes, total_time, archive_number)
                 VALUES(?,?,?,?,?,?,?,?,?,?) RETURNING id as "id!""#,
                title,
                format,
                year,
                label,
                publisher,
                catalogue_number,
                condition,
                notes,
                total_time,
                archive_number,
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

// ─── Audio Import ─────────────────────────────────────────────────────────────

const AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "flac", "ogg", "wav", "m4a", "aac", "opus", "wv", "ape", "aiff", "aif", "mpc", "spx",
    "wma",
];

fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| AUDIO_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Look for cover/folder image files in the given directory (case-insensitive).
/// Preference order: cover.jpg › cover.png › folder.jpg › folder.png
fn find_folder_image(dir: &Path) -> Option<Vec<u8>> {
    const CANDIDATES: &[&str] = &[
        "cover.jpg",    "cover.jpeg",    "cover.png",
        "folder.jpg",   "folder.jpeg",   "folder.png",
        "front.jpg",    "front.jpeg",    "front.png",
        "album.jpg",    "album.jpeg",    "album.png",
        "albumart.jpg", "albumart.jpeg", "albumart.png",
    ];

    let entries = std::fs::read_dir(dir).ok()?;

    let mut best_path: Option<PathBuf> = None;
    let mut best_rank = usize::MAX;

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let lower = file_name.to_string_lossy().to_lowercase();
        if let Some(rank) = CANDIDATES.iter().position(|&c| c == lower) {
            if rank < best_rank {
                best_rank = rank;
                best_path = Some(entry.path());
            }
        }
    }

    best_path.and_then(|p| std::fs::read(&p).ok())
}

struct AudioTrackInfo {
    title: String,
    track_number: String,
    disc_id: String,
    duration_secs: Option<i64>,
    artist: Option<String>,
}

struct ParsedAudioAlbum {
    title: String,
    artist: Option<String>,
    year: Option<i64>,
    genre: Option<String>,
    tracks: Vec<AudioTrackInfo>,
    cover_bytes: Option<Vec<u8>>,
}

/// Synchronously read tags from all audio files in one directory.
fn parse_audio_dir(dir: &Path, mut files: Vec<PathBuf>) -> ParsedAudioAlbum {
    files.sort();

    let mut tracks: Vec<AudioTrackInfo> = Vec::new();
    let mut album_title: Option<String> = None;
    let mut album_artist: Option<String> = None;
    let mut year: Option<i64> = None;
    let mut genre: Option<String> = None;
    let mut embedded_cover: Option<Vec<u8>> = None;

    for path in &files {
        let fallback_title = || {
            path.file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        };

        let Ok(tagged_file) = Probe::open(path).and_then(|p| p.read()) else {
            tracks.push(AudioTrackInfo {
                title: fallback_title(),
                track_number: format!("{:02}", tracks.len() + 1),
                disc_id: "1".to_string(),
                duration_secs: None,
                artist: None,
            });
            continue;
        };

        let duration_secs = Some(tagged_file.properties().duration().as_secs() as i64);

        let Some(tag) = tagged_file.primary_tag().or_else(|| tagged_file.first_tag()) else {
            tracks.push(AudioTrackInfo {
                title: fallback_title(),
                track_number: format!("{:02}", tracks.len() + 1),
                disc_id: "1".to_string(),
                duration_secs,
                artist: None,
            });
            continue;
        };

        // Collect album-level fields from the first file that has them
        if album_title.is_none() {
            album_title = tag.album().map(|s| s.to_string());
        }
        if album_artist.is_none() {
            album_artist = tag
                .get_string(&ItemKey::AlbumArtist)
                .map(|s| s.to_string())
                .or_else(|| tag.artist().map(|s| s.to_string()));
        }
        if year.is_none() {
            year = tag.year().map(|y| y as i64);
        }
        if genre.is_none() {
            genre = tag.genre().map(|s| s.to_string());
        }
        if embedded_cover.is_none() {
            embedded_cover = tag
                .pictures()
                .iter()
                .find(|p| {
                    matches!(p.pic_type(), PictureType::CoverFront | PictureType::Other)
                })
                .map(|p| p.data().to_vec());
        }

        let title = tag
            .title()
            .map(|s| s.to_string())
            .unwrap_or_else(fallback_title);
        let track_number = tag
            .track()
            .map(|n| format!("{:02}", n))
            .unwrap_or_else(|| format!("{:02}", tracks.len() + 1));
        let disc_id = tag
            .disk()
            .map(|d| d.to_string())
            .unwrap_or_else(|| "1".to_string());
        let artist = tag.artist().map(|s| s.to_string());

        tracks.push(AudioTrackInfo {
            title,
            track_number,
            disc_id,
            duration_secs,
            artist,
        });
    }

    // Sort by disc then track number so order matches tags
    tracks.sort_by(|a, b| {
        a.disc_id
            .cmp(&b.disc_id)
            .then_with(|| a.track_number.cmp(&b.track_number))
    });

    let title = album_title.unwrap_or_else(|| {
        dir.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });

    // Folder image takes priority; fall back to embedded cover art
    let cover_bytes = find_folder_image(dir).or(embedded_cover);

    ParsedAudioAlbum {
        title,
        artist: album_artist,
        year,
        genre,
        tracks,
        cover_bytes,
    }
}

/// Synchronously walk `folder` and group audio files by their parent directory.
fn scan_audio_folder(folder: &Path) -> Vec<(PathBuf, Vec<PathBuf>)> {
    let mut dir_groups: BTreeMap<PathBuf, Vec<PathBuf>> = BTreeMap::new();
    for entry in WalkDir::new(folder).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && is_audio_file(entry.path()) {
            let dir = entry.path().parent().unwrap_or(folder).to_path_buf();
            dir_groups
                .entry(dir)
                .or_default()
                .push(entry.path().to_path_buf());
        }
    }
    dir_groups.into_iter().collect()
}

async fn upsert_audio_album(
    db: &sqlx::SqlitePool,
    app: &AppHandle,
    album: &ParsedAudioAlbum,
) -> Result<i64> {
    if album.tracks.is_empty() {
        return Err(AppError::Parse("no audio tracks found".to_string()));
    }

    let total_secs: i64 = album.tracks.iter().filter_map(|t| t.duration_secs).sum();
    let total_time = if total_secs > 0 {
        Some(format_duration_secs(total_secs))
    } else {
        None
    };

    let disc_id = crate::commands::items::next_disc_id(db).await?;

    let mut tx = db.begin().await?;

    let item_id: i64 = sqlx::query_scalar!(
        r#"INSERT INTO items(title, format, year, total_time, disc_id)
           VALUES (?, 'Other', ?, ?, ?) RETURNING id as "id!""#,
        album.title,
        album.year,
        total_time,
        disc_id,
    )
    .fetch_one(&mut *tx)
    .await?;

    // Upsert album artist and link to item
    if let Some(artist_name) = &album.artist {
        let artist_id: i64 = sqlx::query_scalar!(
            r#"INSERT INTO artists(name, sort_name) VALUES(?,?)
               ON CONFLICT(sort_name) DO UPDATE SET name=excluded.name
               RETURNING id as "id!""#,
            artist_name,
            artist_name,
        )
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query!(
            "INSERT OR IGNORE INTO item_artists(item_id, artist_id, role, sort_order)
             VALUES(?,?,'artist',0)",
            item_id,
            artist_id,
        )
        .execute(&mut *tx)
        .await?;
    }

    // Upsert genre and link to item
    if let Some(genre_name) = &album.genre {
        let genre_id: i64 = sqlx::query_scalar!(
            r#"INSERT INTO genres(name) VALUES(?)
               ON CONFLICT(name) DO UPDATE SET name=excluded.name
               RETURNING id as "id!""#,
            genre_name,
        )
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query!(
            "INSERT OR IGNORE INTO item_genres(item_id, genre_id) VALUES(?,?)",
            item_id,
            genre_id,
        )
        .execute(&mut *tx)
        .await?;
    }

    // Insert tracks
    for (ord, track) in album.tracks.iter().enumerate() {
        let sort_order = ord as i64;

        let track_id: i64 = sqlx::query_scalar!(
            r#"INSERT INTO tracks(item_id, disc_id, track_number, title, duration_secs, sort_order)
               VALUES(?,?,?,?,?,?) RETURNING id as "id!""#,
            item_id,
            track.disc_id,
            track.track_number,
            track.title,
            track.duration_secs,
            sort_order,
        )
        .fetch_one(&mut *tx)
        .await?;

        if let Some(artist_name) = &track.artist {
            let artist_id: i64 = sqlx::query_scalar!(
                r#"INSERT INTO artists(name, sort_name) VALUES(?,?)
                   ON CONFLICT(sort_name) DO UPDATE SET name=excluded.name
                   RETURNING id as "id!""#,
                artist_name,
                artist_name,
            )
            .fetch_one(&mut *tx)
            .await?;

            sqlx::query!(
                "INSERT OR IGNORE INTO track_artists(track_id, artist_id, role)
                 VALUES(?,?,'artist')",
                track_id,
                artist_id,
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;

    // Save cover art after commit (failures are non-fatal — album is already imported)
    if let Some(bytes) = &album.cover_bytes {
        if let Ok(img) = image::load_from_memory(bytes) {
            if let Ok(data_dir) = app.path().app_data_dir() {
                let covers_dir = data_dir.join("covers");
                let _ = tokio::fs::create_dir_all(&covers_dir).await;
                let filename = format!("{}.jpg", item_id);
                let filepath = covers_dir.join(&filename);
                let thumb = img.resize(500, 500, image::imageops::FilterType::Lanczos3);
                if thumb.save_with_format(&filepath, ImageFormat::Jpeg).is_ok() {
                    let abs_path = filepath.to_string_lossy().to_string();
                    let _ = sqlx::query!(
                        "UPDATE items SET cover_art_path=?, updated_at=datetime('now') WHERE id=?",
                        abs_path,
                        item_id,
                    )
                    .execute(db)
                    .await;
                }
            }
        }
    }

    Ok(item_id)
}

#[tauri::command]
pub async fn import_audio_folder(
    folders: Vec<String>,
    state: State<'_, AppState>,
    window: tauri::Window,
    app: AppHandle,
) -> Result<AudioImportSummary> {
    let folder_paths: Vec<PathBuf> = folders.into_iter().map(PathBuf::from).collect();

    // Phase 1: walk every selected directory and collect audio file paths (blocking I/O)
    let groups = tokio::task::spawn_blocking(move || {
        let mut all: Vec<(PathBuf, Vec<PathBuf>)> = Vec::new();
        for root in &folder_paths {
            all.extend(scan_audio_folder(root));
        }
        all
    })
    .await
    .map_err(|e| AppError::Parse(e.to_string()))?;

    let total_albums = groups.len();
    let total_files: usize = groups.iter().map(|(_, files)| files.len()).sum();

    // Phase 2: parse tags for every directory group (blocking CPU + I/O)
    let albums: Vec<ParsedAudioAlbum> = tokio::task::spawn_blocking(move || {
        groups
            .into_iter()
            .map(|(dir, files)| parse_audio_dir(&dir, files))
            .collect()
    })
    .await
    .map_err(|e| AppError::Parse(e.to_string()))?;

    let mut imported = 0usize;
    let mut skipped = 0usize;

    // Phase 3: upsert each parsed album into the database
    for (i, album) in albums.iter().enumerate() {
        let _ = window.emit(
            "import-progress",
            serde_json::json!({
                "done": i,
                "total": total_albums,
                "current": &album.title,
            }),
        );

        match upsert_audio_album(&state.db, &app, album).await {
            Ok(_) => imported += 1,
            Err(e) => {
                eprintln!("Audio import skip \"{}\": {e}", album.title);
                skipped += 1;
            }
        }
    }

    let _ = window.emit(
        "import-progress",
        serde_json::json!({ "done": total_albums, "total": total_albums, "current": "" }),
    );

    Ok(AudioImportSummary {
        total_files,
        total_albums,
        imported,
        skipped,
    })
}
