use sqlx::SqlitePool;
use std::sync::Mutex;

use crate::models::item::ItemWithArtists;

pub struct AppState {
    pub db: SqlitePool,
    pub undo_buffer: Mutex<Option<UndoEntry>>,
}

pub struct UndoEntry {
    pub item: ItemWithArtists,
}
