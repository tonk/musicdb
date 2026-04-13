use sqlx::SqlitePool;
use std::sync::Mutex;
use tokio::sync::RwLock;

use crate::models::item::ItemWithArtists;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct DatabaseEntry {
    pub name: String,
    pub path: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct DatabaseConfig {
    pub current: String,
    pub databases: Vec<DatabaseEntry>,
}

pub struct AppState {
    pub db: RwLock<SqlitePool>,
    pub db_config: Mutex<DatabaseConfig>,
    pub undo_buffer: Mutex<Option<UndoEntry>>,
}

impl AppState {
    /// Returns a cheap clone of the active pool (SqlitePool is Arc-wrapped internally).
    pub async fn pool(&self) -> SqlitePool {
        self.db.read().await.clone()
    }
}

pub struct UndoEntry {
    pub item: ItemWithArtists,
}
