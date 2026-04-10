use serde::{Deserialize, Serialize};

use super::{artist::ArtistWithRole, genre::Genre, track::Track};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemRow {
    pub id: i64,
    pub title: String,
    pub format: String,
    pub year: Option<i64>,
    pub label: Option<String>,
    pub publisher: Option<String>,
    pub catalogue_number: Option<String>,
    pub condition: Option<String>,
    pub notes: Option<String>,
    pub cover_art_path: Option<String>,
    pub disc_id: Option<String>,
    pub source_category: Option<String>,
    pub musicbrainz_id: Option<String>,
    pub total_time: Option<String>,
    pub archive_number: Option<String>,
    pub date_added: String,
    pub updated_at: String,
}

/// Lightweight summary for list/grid views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemSummary {
    pub id: i64,
    pub title: String,
    pub format: String,
    pub year: Option<i64>,
    pub label: Option<String>,
    pub catalogue_number: Option<String>,
    pub cover_art_path: Option<String>,
    pub date_added: String,
    /// Comma-joined artist names for display
    pub artist_names: String,
}

/// Full item with all relations — used for detail view and undo buffer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemWithArtists {
    pub id: i64,
    pub title: String,
    pub format: String,
    pub year: Option<i64>,
    pub label: Option<String>,
    pub publisher: Option<String>,
    pub catalogue_number: Option<String>,
    pub condition: Option<String>,
    pub notes: Option<String>,
    pub cover_art_path: Option<String>,
    pub disc_id: Option<String>,
    pub source_category: Option<String>,
    pub musicbrainz_id: Option<String>,
    pub total_time: Option<String>,
    pub archive_number: Option<String>,
    pub date_added: String,
    pub updated_at: String,
    pub artists: Vec<ArtistWithRole>,
    pub genres: Vec<Genre>,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Deserialize)]
pub struct CreateItemInput {
    pub title: String,
    pub format: String,
    pub year: Option<i64>,
    pub label: Option<String>,
    pub publisher: Option<String>,
    pub catalogue_number: Option<String>,
    pub condition: Option<String>,
    pub notes: Option<String>,
    pub musicbrainz_id: Option<String>,
    pub total_time: Option<String>,
    pub archive_number: Option<String>,
    pub artist_ids: Vec<ArtistRoleInput>,
    pub genre_ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ArtistRoleInput {
    pub artist_id: i64,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateItemInput {
    pub title: Option<String>,
    pub format: Option<String>,
    pub year: Option<i64>,
    pub label: Option<String>,
    pub publisher: Option<String>,
    pub catalogue_number: Option<String>,
    pub condition: Option<String>,
    pub notes: Option<String>,
    pub musicbrainz_id: Option<String>,
    pub total_time: Option<String>,
    pub archive_number: Option<String>,
    pub artist_ids: Option<Vec<ArtistRoleInput>>,
    pub genre_ids: Option<Vec<i64>>,
}

#[derive(Debug, Deserialize)]
pub struct ListItemsParams {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub sort_field: Option<String>,
    pub sort_dir: Option<String>,
    pub format: Option<String>,
    pub condition: Option<String>,
    pub year_from: Option<i64>,
    pub year_to: Option<i64>,
    pub genre_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ItemsPage {
    pub items: Vec<ItemSummary>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Serialize)]
pub struct Statistics {
    pub total_items: i64,
    pub total_tracks: i64,
    pub by_format: Vec<CountEntry>,
    pub by_genre: Vec<CountEntry>,
    pub by_year: Vec<CountEntry>,
}

#[derive(Debug, Serialize)]
pub struct CountEntry {
    pub label: String,
    pub count: i64,
}
