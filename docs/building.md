# Building MusicDB

## Prerequisites

### Rust

Install via [rustup](https://rustup.rs) on all platforms.

**Linux / macOS:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Windows:** download and run `rustup-init.exe` from [rustup.rs](https://rustup.rs). rustup will prompt you to install the MSVC toolchain automatically.

### Node.js

Version 20 or later. Install via [nvm](https://github.com/nvm-sh/nvm) (Linux/macOS), [nvm-windows](https://github.com/coreybutler/nvm-windows), or directly from [nodejs.org](https://nodejs.org).

### Platform-specific system libraries

#### Windows

Install [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and select the **"Desktop development with C++"** workload (alternatively install Visual Studio 2022 with that workload).

WebView2 runtime is pre-installed on Windows 11. On Windows 10 download the [Evergreen Bootstrapper](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) and run it.

No further system libraries are needed — the MSVC toolchain and WebView2 cover all runtime requirements.

#### macOS

Install the Xcode Command Line Tools (provides `clang`, `make`, and other build essentials):
```bash
xcode-select --install
```

No additional system libraries are needed.

#### Debian / Ubuntu
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

#### Fedora
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

## Quick start

```bash
git clone <repo-url> musicdb
cd musicdb
npm ci             # install exact Node.js deps from package-lock.json
make dev           # hot-reload development server (Linux/macOS)
```

On **Windows** there is no Makefile; use npm directly:
```powershell
npm ci
npm run tauri dev
```

If you already ran `npm install` before and hit frontend dependency resolution errors, reset and reinstall:
```bash
rm -rf node_modules
npm ci
```

## Note: AppImage on Fedora / modern distros

Tauri downloads `linuxdeploy` as an AppImage to assemble the final AppImage bundle. Two issues can arise on modern systems:

1. **FUSE** — Running the linuxdeploy helper requires FUSE, which is not always available (containers, Fedora/newer-kernel setups). The Makefile and CI workflows set `APPIMAGE_EXTRACT_AND_RUN=1` to work around this — linuxdeploy extracts itself to a temp directory instead of mounting via FUSE.

2. **Old bundled `strip`** — Fedora 38+ and Arch produce ELF files with `SHT_RELR` (type 0x13) relocation sections. The `strip` binary bundled inside linuxdeploy's extraction payload is too old to handle these. `make appimage` automatically detects when linuxdeploy extracts its payload and replaces the bundled `strip` with `/usr/bin/strip` on-the-fly — no manual steps needed.

## Building packages

### Linux

```bash
rm -rf node_modules && npm ci   # recommended after clone or lockfile updates
make              # build all configured targets (AppImage, .deb, .rpm)
make appimage     # AppImage only
make deb          # .deb only
make rpm          # .rpm only
```

Output files land in `src-tauri/target/release/bundle/`.

You can also invoke Tauri directly:
```bash
npm run tauri build                         # all targets from tauri.conf.json
npm run tauri build -- --bundles appimage   # single target override
```

### Common troubleshooting

- `Error [ERR_MODULE_NOT_FOUND]: Cannot find package 'esbuild'` or `'rollup'` during `vite build`
  - Cause: stale or partially-installed frontend dependencies.
  - Fix:
    ```bash
    rm -rf node_modules
    npm ci
    ```
- `beforeBuildCommand npm run build failed` in `tauri build`
  - This is usually a frontend build issue; run `npm run build` directly first to see the root error, then retry `make appimage`.

### Windows

```powershell
npm run tauri build
```

Produces an `.msi` installer and a standalone `.exe` in `src-tauri\target\release\bundle\`.

### macOS

```bash
npm run tauri build
```

Produces a `.dmg` disk image and a `.app` bundle in `src-tauri/target/release/bundle/`.

## Makefile reference

| Target | Description |
|--------|-------------|
| `make` / `make build` | Build all bundles defined in `tauri.conf.json` |
| `make appimage` | Build Linux AppImage (auto-fixes strip on Fedora/Arch) |
| `make deb` | Build Debian package only |
| `make rpm` | Build RPM package only |
| `make dev` | Start development server with hot-reload |
| `make clean` | Remove Rust build artefacts and frontend `dist/` |
| `make help` | Print target summary |

## CI/CD

Two equivalent workflows are provided:

| File | Platform |
|------|---------|
| `.github/workflows/build.yml` | GitHub Actions |
| `.forgejo/workflows/build.yml` | Forgejo Actions |

Both workflows:
- Run on every push to `main` and on pull requests
- Build the AppImage and upload it as a workflow artefact
- On a `v*` tag push, publish a release with the AppImage attached

### Creating a release

```bash
git tag v1.0.0
git push origin v1.0.0
```

The CI workflow will build the AppImage and attach it to the release automatically.

**Forgejo only:** add a personal access token with `write:repository` scope as a repository secret named `FORGEJO_TOKEN`.

## Project structure

```
musicdb/
├── src/                    # Vue 3 frontend
│   ├── views/              # Page-level components
│   ├── components/         # Shared UI components
│   ├── stores/             # Pinia state stores
│   ├── i18n/               # Translation files (en, nl, de, fr, es)
│   └── assets/styles/      # Global CSS + theme variables
├── src-tauri/              # Rust / Tauri backend
│   ├── src/
│   │   ├── commands/       # Tauri command handlers
│   │   ├── models/         # Data model structs
│   │   ├── db.rs           # SQLite pool initialisation
│   │   ├── state.rs        # App state
│   │   └── lib.rs          # Entry point + command registration
│   ├── migrations/         # SQLite schema migrations
│   └── tauri.conf.json     # Tauri configuration
├── .github/workflows/      # GitHub Actions
├── .forgejo/workflows/     # Forgejo Actions
├── Makefile
└── TODO.md
```
