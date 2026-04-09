use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Artist {
    pub id: i64,
    pub name: String,
    pub sort_name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistWithRole {
    pub id: i64,
    pub name: String,
    pub sort_name: String,
    pub role: String,
    pub sort_order: i64,
}
