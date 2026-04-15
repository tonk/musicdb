# MusicDB User Guide

## Navigation

The sidebar provides access to all sections:

| Section | Description |
|---------|-------------|
| Collection | Browse and manage your full collection |
| Search | Full-text and wildcard search |
| Statistics | Counts by format, genre, and year |
| Import | Import from audio folders, CSV, or legacy TXT |
| Settings | Appearance, language, and database management |

---

## Collection

The collection view lists all items sorted by date added (newest first by default). Click any column header to sort by that field.

### Views
Switch between **List** view (table with sortable columns) and **Grid** view (cover art thumbnails) using the toggle in the toolbar.

### Filters
Click **Filters** to narrow results by format, condition, genre, or year range. Active filters are shown as chips; click the × on a chip or **Clear Filters** to remove them.

### Adding an item
Click **Add Item** and fill in the fields. Required fields are Title and Format. Artists and genres are looked up from existing entries and created on the fly if they don't exist yet.

### Editing and deleting
Open an item by clicking its row or card. Use the **Edit** button to modify fields. **Delete** moves the item to an undo buffer — a toast notification appears at the bottom of the screen with an **Undo** link that restores it.

---

## Search

The search box in the Search view queries titles, artists, labels, catalogue numbers, and notes simultaneously.

### Plain search
Type any word or phrase and press Enter. Results are ranked by relevance using SQLite FTS5.

### Wildcard search
Use `*` as a wildcard character:

| Query | Matches |
|-------|---------|
| `beat*` | Anything starting with "beat" (Beatles, Beatniks, …) |
| `*stein` | Anything ending with "stein" |
| `*rock*` | Anything containing "rock" |

Wildcards trigger a SQL `LIKE` search across title, label, catalogue number, and artist name.

### Clearing search
Clearing the search box immediately removes the results without needing to resubmit.

---

## Import

### Audio folder import
Select one or more folders. MusicDB recursively scans for audio files (MP3, FLAC, OGG, WAV, AIFF, M4A, …), reads the ID3 or Vorbis tags, and groups tracks by album. The format field is set automatically from the file extension (e.g. MP3, FLAC). Cover art embedded in the files or present as `cover.jpg` / `folder.jpg` in the folder is imported automatically.

A progress bar shows the current album being processed.

### CSV import
1. Click **Select File** and pick a `.csv` file.
2. MusicDB shows a preview of the first row and auto-maps column headers to fields where the names match.
3. Adjust the mapping using the dropdowns, then click **Import**.

Supported target fields: `title`, `artist`, `format`, `year`, `label`, `publisher`, `catalogue_number`, `condition`, `genre`, `notes`, `total_time`, `archive_number`.

### TXT import (legacy)
Imports from the CDN database export format (`.txt`). A progress bar tracks items processed.

---

## Export

In the Import view, scroll to the **Export** section:

- **Export CSV** — flat CSV with one item per row; artists and genres are semicolon-joined in single columns.
- **Export JSON** — full JSON array including all artists, genres, and per-track data.

Both exports include all items in the active database.

---

## Cover Art

On an item's detail view, click the cover art area (or the placeholder) to open the cover art picker:

- **Upload local file** — choose any JPEG, PNG, or WebP image.
- **Fetch from MusicBrainz** — if the item has a MusicBrainz ID set, MusicDB fetches the front cover from the [Cover Art Archive](https://coverartarchive.org) automatically.

---

## MusicBrainz Lookup

On an item's detail view, click **Look Up on MusicBrainz**. MusicDB searches MusicBrainz by title and artist and shows a list of matching releases. Selecting a release populates the MusicBrainz ID, label, catalogue number, year, and cover art.

---

## Statistics

The Statistics view shows:
- Total item and track counts
- Breakdown by format (CD, LP, etc.)
- Top genres by item count
- Items per release year

---

## Databases

MusicDB supports multiple independent databases — useful for keeping a vinyl collection separate from CDs, or a personal collection separate from a shared one.

### Managing databases
Go to **Settings → Databases**:
- **Add** a new database by typing a name and pressing Add.
- **Rename** an existing database inline.
- **Delete** a database (not possible while it is the active one).
- Switch the active database by clicking its name in the **sidebar** (below the navigation items).

### Backup
Go to **Settings → Active Database** and click **Backup Database**. A save dialog opens with a default filename containing the current timestamp (`musicdb_backup_YYYYmmddHHMMSS.sqlite`). The backup is a plain SQLite file — it can be opened directly with any SQLite browser.

### Move database
**Settings → Active Database → Move Database** copies the current database file to a new location. Restart the app after moving to use the new path.

---

## Settings

| Setting | Description |
|---------|-------------|
| Theme | Light, Dark, or follow the system setting |
| Language | English, Nederlands, Deutsch, Français, Español |
| Default View | List or Grid for the Collection view |
| Startup View | Which section opens when the app starts |
| Items per page | Number of items shown per page in the Collection view (25 / 50 / 100 / 200; default 50) |

---

## Keyboard shortcuts

| Key | Action |
|-----|--------|
| Click column header | Sort collection by that column (click again to reverse) |
| Enter (search box) | Run search |
| Escape (rename field) | Cancel rename |
| Enter (rename field) | Confirm rename |
