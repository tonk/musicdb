-- ─── ARTISTS ─────────────────────────────────────────────────────────────────

CREATE TABLE artists (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT    NOT NULL,
    sort_name  TEXT    NOT NULL,
    created_at TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE UNIQUE INDEX uq_artists_sort_name ON artists(sort_name);
CREATE INDEX idx_artists_name ON artists(name COLLATE NOCASE);

-- ─── GENRES ──────────────────────────────────────────────────────────────────

CREATE TABLE genres (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL
);
CREATE UNIQUE INDEX uq_genres_name ON genres(name COLLATE NOCASE);

-- ─── COLLECTION ITEMS ────────────────────────────────────────────────────────

CREATE TABLE items (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    title            TEXT    NOT NULL,
    format           TEXT    NOT NULL DEFAULT 'CD'
                             CHECK(format IN ('CD','LP','EP','Single','Cassette','Music Book','Sheet Music','Other')),
    year             INTEGER,
    label            TEXT,
    publisher        TEXT,
    catalogue_number TEXT,
    condition        TEXT    CHECK(condition IN ('Mint','Near Mint','VG+','VG','Good','Fair','Poor') OR condition IS NULL),
    notes            TEXT,
    cover_art_path   TEXT,
    disc_id          TEXT,
    source_category  TEXT,
    musicbrainz_id   TEXT,
    date_added       TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at       TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE UNIQUE INDEX uq_items_cat_disc ON items(catalogue_number, disc_id)
    WHERE catalogue_number IS NOT NULL;
CREATE INDEX idx_items_title     ON items(title COLLATE NOCASE);
CREATE INDEX idx_items_year      ON items(year);
CREATE INDEX idx_items_format    ON items(format);
CREATE INDEX idx_items_catalogue ON items(catalogue_number);
CREATE INDEX idx_items_date      ON items(date_added);

-- ─── TRACKS ──────────────────────────────────────────────────────────────────

CREATE TABLE tracks (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id       INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    disc_id       TEXT    NOT NULL,
    track_number  TEXT    NOT NULL,
    title         TEXT    NOT NULL,
    duration_secs INTEGER,
    version       TEXT,
    sort_order    INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_tracks_item_id ON tracks(item_id);

-- ─── TRACK CREDITS ───────────────────────────────────────────────────────────

CREATE TABLE track_artists (
    track_id  INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    artist_id INTEGER NOT NULL REFERENCES artists(id) ON DELETE RESTRICT,
    role      TEXT    NOT NULL DEFAULT 'artist'
);
CREATE UNIQUE INDEX uq_track_artists ON track_artists(track_id, artist_id, role);
CREATE INDEX idx_track_artists_artist ON track_artists(artist_id);

-- ─── ITEM RELATIONSHIPS ──────────────────────────────────────────────────────

CREATE TABLE item_artists (
    item_id    INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    artist_id  INTEGER NOT NULL REFERENCES artists(id) ON DELETE RESTRICT,
    role       TEXT    NOT NULL DEFAULT 'artist',
    sort_order INTEGER NOT NULL DEFAULT 0
);
CREATE UNIQUE INDEX uq_item_artists ON item_artists(item_id, artist_id, role);
CREATE INDEX idx_item_artists_artist ON item_artists(artist_id);

CREATE TABLE item_genres (
    item_id  INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    genre_id INTEGER NOT NULL REFERENCES genres(id) ON DELETE RESTRICT
);
CREATE UNIQUE INDEX uq_item_genres ON item_genres(item_id, genre_id);
CREATE INDEX idx_item_genres_genre ON item_genres(genre_id);

-- ─── SETTINGS ────────────────────────────────────────────────────────────────

CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT
);
INSERT INTO settings VALUES
    ('theme',         'system'),
    ('default_view',  'list'),
    ('date_format',   'YYYY-MM-DD'),
    ('language',      'en'),
    ('startup_view',  'collection'),
    ('window_x',      NULL),
    ('window_y',      NULL),
    ('window_width',  '1200'),
    ('window_height', '800');

-- ─── FULL-TEXT SEARCH ────────────────────────────────────────────────────────

CREATE VIRTUAL TABLE items_fts USING fts5(
    title,
    artist_names,
    label,
    publisher,
    catalogue_number,
    notes,
    content='items',
    content_rowid='id'
);

CREATE TRIGGER items_ai AFTER INSERT ON items BEGIN
    INSERT INTO items_fts(rowid, title, artist_names, label, publisher, catalogue_number, notes)
    VALUES (new.id, new.title, '', new.label, new.publisher, new.catalogue_number, new.notes);
END;

CREATE TRIGGER items_ad AFTER DELETE ON items BEGIN
    INSERT INTO items_fts(items_fts, rowid, title, artist_names, label, publisher, catalogue_number, notes)
    VALUES ('delete', old.id, old.title, '', old.label, old.publisher, old.catalogue_number, old.notes);
END;

CREATE TRIGGER items_au AFTER UPDATE ON items BEGIN
    INSERT INTO items_fts(items_fts, rowid, title, artist_names, label, publisher, catalogue_number, notes)
    VALUES ('delete', old.id, old.title, '', old.label, old.publisher, old.catalogue_number, old.notes);
    INSERT INTO items_fts(rowid, title, artist_names, label, publisher, catalogue_number, notes)
    VALUES (new.id, new.title, '', new.label, new.publisher, new.catalogue_number, new.notes);
END;
