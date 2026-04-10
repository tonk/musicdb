ALTER TABLE items ADD COLUMN total_time     TEXT;
ALTER TABLE items ADD COLUMN archive_number TEXT;

CREATE INDEX idx_items_archive ON items(archive_number);
