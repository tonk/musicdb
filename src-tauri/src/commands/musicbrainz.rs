use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{error::Result, state::AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct MbRelease {
    pub id: String,
    pub title: String,
    pub date: Option<String>,
    pub label: Option<String>,
    pub catalogue: Option<String>,
    pub genres: Vec<String>,
    pub artist: String,
}

#[derive(Debug, Deserialize)]
struct MbSearchResponse {
    releases: Vec<MbReleaseRaw>,
}

#[derive(Debug, Deserialize)]
struct MbReleaseRaw {
    id: String,
    title: String,
    date: Option<String>,
    #[serde(rename = "artist-credit")]
    artist_credit: Option<Vec<ArtistCredit>>,
    #[serde(rename = "label-info")]
    label_info: Option<Vec<LabelInfo>>,
    genres: Option<Vec<MbGenre>>,
    tags: Option<Vec<MbTag>>,
}

#[derive(Debug, Deserialize)]
struct ArtistCredit {
    artist: ArtistRaw,
}

#[derive(Debug, Deserialize)]
struct ArtistRaw {
    name: String,
}

#[derive(Debug, Deserialize)]
struct LabelInfo {
    label: Option<LabelRaw>,
    #[serde(rename = "catalog-number")]
    catalog_number: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LabelRaw {
    name: String,
}

#[derive(Debug, Deserialize)]
struct MbGenre {
    name: String,
}

#[derive(Debug, Deserialize)]
struct MbTag {
    name: String,
    count: i64,
}

#[tauri::command]
pub async fn lookup_release(
    title: String,
    artist: String,
    _state: State<'_, AppState>,
) -> Result<Vec<MbRelease>> {
    // MusicBrainz requires a descriptive User-Agent
    let client = reqwest::Client::builder()
        .user_agent("MusicDB/0.1.0 (https://github.com/musicdb)")
        .build()?;

    let query = format!("release:{} AND artist:{}", title, artist);
    let url = format!(
        "https://musicbrainz.org/ws/2/release?query={}&fmt=json&limit=10&inc=genres+tags+label-info+artist-credits",
        urlencoding::encode(&query)
    );

    // Respect MusicBrainz 1 req/sec rate limit
    tokio::time::sleep(tokio::time::Duration::from_millis(1100)).await;

    let resp: MbSearchResponse = client.get(&url).send().await?.json().await?;

    Ok(resp
        .releases
        .into_iter()
        .map(|r| {
            let artist = r
                .artist_credit
                .as_deref()
                .and_then(|ac| ac.first())
                .map(|ac| ac.artist.name.clone())
                .unwrap_or_default();

            let (label, catalogue) = r
                .label_info
                .as_deref()
                .and_then(|li| li.first())
                .map(|li| {
                    (
                        li.label.as_ref().map(|l| l.name.clone()),
                        li.catalog_number.clone(),
                    )
                })
                .unwrap_or((None, None));

            let genres = r
                .genres
                .unwrap_or_default()
                .into_iter()
                .map(|g| g.name)
                .chain(
                    r.tags
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|t| t.count > 0)
                        .map(|t| t.name),
                )
                .take(10)
                .collect();

            MbRelease {
                id: r.id,
                title: r.title,
                date: r.date,
                label,
                catalogue,
                genres,
                artist,
            }
        })
        .collect())
}
