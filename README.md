<div align="center">
  <img src="src-tauri/icons/128x128.png" alt="MusicDB logo" width="128" />

# MusicDB

A desktop application for managing a personal music collection. Built with [Tauri 2](https://tauri.app), [Vue 3](https://vuejs.org), and [SQLite](https://sqlite.org).

</div>

## Features

- **Collection management** — add, edit, and delete albums, singles, cassettes, vinyl, sheet music, and more
- **Full-text search** — instant search across titles, artists, labels, catalogue numbers, and notes; supports `*` wildcards
- **Import** — scan audio folders (MP3, FLAC, OGG, WAV; reads ID3/Vorbis tags), import CSV with column mapping, import legacy TXT (CDN format)
- **Export** — export your full collection to CSV or JSON
- **Cover art** — attach local images or fetch automatically from the MusicBrainz Cover Art Archive
- **MusicBrainz lookup** — search and populate metadata from MusicBrainz
- **Multiple databases** — maintain separate databases (e.g. vinyl vs. CD) and switch between them instantly
- **Database backup** — one-click backup with a timestamped filename
- **Statistics** — breakdown by format, genre, and year
- **Themes** — light, dark, and system-follow modes
- **Localisation** — English, Dutch, German, French, Spanish

## Installation

Download the latest release for your platform from the [Releases](../../releases) page:

| Package | Format |
|---------|--------|
| `MusicDB_x.y.z_amd64.AppImage` | Linux (portable, no install needed) |
| `musicdb_x.y.z_amd64.deb` | Debian / Ubuntu |
| `musicdb-x.y.z-1.x86_64.rpm` | Fedora / openSUSE |

**AppImage** — make it executable and run:
```bash
chmod +x MusicDB_*.AppImage
./MusicDB_*.AppImage
```

## Building from source

See [docs/building.md](docs/building.md).

## Usage

See [docs/user-guide.md](docs/user-guide.md).

## Tech stack

| Layer | Technology |
|-------|-----------|
| Frontend | Vue 3 + TypeScript + Vite |
| Backend | Rust (Tauri 2) |
| Database | SQLite via sqlx |
| Search | SQLite FTS5 |
| Packaging | AppImage, .deb, .rpm |
| CI/CD | GitHub Actions, Forgejo Actions |

## Licence

MIT
