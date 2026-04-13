use serde::{Deserialize, Serialize};

/// One raw record parsed from the .txt file
#[derive(Debug, Default, Clone)]
pub struct TxtRecord {
    pub nummer: String,
    pub disc_id: String,
    pub track_num: String,
    pub titel: String,
    pub artiest: String,
    pub label: String,
    pub uitgever: String,
    pub tijd: String,
    pub category: String,
    pub catalogus: Option<String>,
    pub drager: Option<String>,
    pub jaar: Option<i64>,
    pub arrangeur: Option<String>,
    pub componist1: Option<String>,
    pub versie: Option<String>,
}

/// One album/carrier derived from grouping TxtRecords
#[derive(Debug, Clone)]
pub struct ImportAlbum {
    pub disc_id: String,
    pub title: String,
    pub catalogue_number: Option<String>,
    pub label: String,
    pub publisher: String,
    pub year: Option<i64>,
    pub category: String,
    pub total_time: Option<String>,
    pub tracks: Vec<TxtRecord>,
    /// Deduped sort_name strings for all artists across all tracks
    pub artist_sort_names: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ImportSummary {
    pub total: usize,
    pub imported: usize,
    pub skipped: usize,
}

/// CSV column mapping entry from the frontend wizard
#[derive(Debug, Deserialize)]
pub struct CsvColumnMapping {
    pub csv_column: String,
    pub item_field: String,
}

#[derive(Debug, Serialize)]
pub struct CsvPreview {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct AudioImportSummary {
    pub total_files: usize,
    pub total_albums: usize,
    pub imported: usize,
    pub skipped: usize,
}
