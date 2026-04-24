<div align="center">
  <img src="src-tauri/icons/128x128.png" alt="MusicDB logo" width="128" />

# MusicDB

A desktop application for managing a personal music collection. Built with [Tauri 2](https://tauri.app), [Vue 3](https://vuejs.org), and [SQLite](https://sqlite.org).

</div>

## Features

- **Collection management** — add, edit, and delete albums, singles, cassettes, vinyl, sheet music, and more
- **Full-text search** — instant search across titles, artists, labels, catalogue numbers, and notes; supports `*` wildcards
- **Import** — scan audio folders (MP3, FLAC, OGG, WAV, …; reads ID3/Vorbis tags and detects format automatically), import CSV with column mapping, import legacy TXT (CDN format)
- **Export** — export your full collection to CSV or JSON
- **Cover art** — attach local images or fetch automatically from the MusicBrainz Cover Art Archive
- **MusicBrainz lookup** — search and populate metadata from MusicBrainz
- **Multiple databases** — maintain separate databases (e.g. vinyl vs. CD) and switch between them instantly
- **Database backup** — one-click backup with a timestamped filename
- **Statistics** — breakdown by format, genre, and year
- **Configurable page size** — choose how many items are shown per page (25 / 50 / 100 / 200)
- **Themes** — light, dark, and system-follow modes
- **Localisation** — English, Dutch, German, French, Spanish

## Installation

Download the latest release for your platform from the [Releases](../../releases) page:

| Package | Format |
|---------|--------|
| `MusicDB_x.y.z_amd64.AppImage` | Linux (portable, no install needed) |
| `musicdb_x.y.z_amd64.deb` | Debian / Ubuntu |
| `musicdb-x.y.z-1.x86_64.rpm` | Fedora / openSUSE |
| `MusicDB_x.y.z_x64-setup.exe` | Microsoft Windows |
| `MusicDB_x.y.z_universal.dmg` | Apple macOS |

**AppImage** — make it executable and run:
```bash
chmod +x MusicDB_*.AppImage
./MusicDB_*.AppImage
```

## Building from source

### Prerequisites

#### Rust
Install via [rustup](https://rustup.rs) on all platforms.

On **Linux / macOS**:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

On **Windows**: download and run `rustup-init.exe` from [rustup.rs](https://rustup.rs).

#### Node.js
Version 20 or later. Install via [nvm](https://github.com/nvm-sh/nvm) (Linux/macOS), [nvm-windows](https://github.com/coreybutler/nvm-windows), or directly from [nodejs.org](https://nodejs.org).

#### Platform-specific system dependencies

**Windows**

Install [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (select the "Desktop development with C++" workload), or Visual Studio 2022 with that workload included.

WebView2 is pre-installed on Windows 11. On Windows 10 download the [Evergreen Bootstrapper](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).

**macOS**
```bash
xcode-select --install
```

**Debian / Ubuntu**
```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl wget file \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf \
  squashfs-tools
```

**Fedora**
```bash
sudo dnf install -y \
  webkit2gtk4.1-devel \
  openssl-devel \
  curl wget file \
  libappindicator-gtk3-devel \
  librsvg2-devel \
  patchelf \
  squashfs-tools
```

### Install Node.js package dependencies

```bash
npm install
```

If you are setting up from scratch or after pulling dependency changes, prefer a clean install:
```bash
rm -rf node_modules
npm ci
```

This project uses Vite 8, which expects certain optional peer dependencies (for example `esbuild` and `rollup`) to be present. Using `npm ci` from `package-lock.json` ensures they are installed consistently.

### Development server

```bash
npm run tauri dev
# or
make dev
```

### Production build

**Linux**
```bash
rm -rf node_modules && npm ci   # recommended once after fresh clone / lockfile updates
make              # AppImage + .deb + .rpm
make appimage     # AppImage only
make deb          # Debian package only
make rpm          # RPM package only
```

If `npm run build` fails with `ERR_MODULE_NOT_FOUND` (for example missing `esbuild` or `rollup`), reinstall frontend dependencies with:
```bash
rm -rf node_modules
npm ci
```

**Windows / macOS**
```bash
npm run tauri build
```

Output files are placed in `src-tauri/target/release/bundle/`.

For advanced build options, CI setup, and project structure see [docs/building.md](docs/building.md).

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
