use serde::{Deserialize, Serialize};

use super::artist::ArtistWithRole;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TrackRow {
    pub id: i64,
    pub item_id: i64,
    pub disc_id: String,
    pub track_number: String,
    pub title: String,
    pub duration_secs: Option<i64>,
    pub version: Option<String>,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: i64,
    pub item_id: i64,
    pub disc_id: String,
    pub track_number: String,
    pub title: String,
    pub duration_secs: Option<i64>,
    pub version: Option<String>,
    pub sort_order: i64,
    pub artists: Vec<ArtistWithRole>,
}
