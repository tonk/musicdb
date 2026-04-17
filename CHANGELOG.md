# Changelog

## v0.1.6 — 2026-04-17

### Changed
- Major dependency updates: Vite 8.0, TypeScript 6.0, Vue-i18n 11.3, Vue-router 5.0, and @vitejs/plugin-vue 6.0.
- Updated Rust dependencies (tokio, uuid, rand, etc.) to their latest compatible versions.

## v0.1.5 — 2026-04-15

### Added
- **Items per page** — Settings now includes an "Items per page" selector (25 / 50 / 100 / 200). The choice is persisted in the database and takes effect immediately.

### Fixed
- Audio folder import now stores the actual file format (MP3, FLAC, OGG, WAV, …) instead of "Other", so the Statistics → By Format breakdown reflects the real media type.
- Cargo edition downgraded from `2026` to `2024` to match the Rust/Cargo toolchain version shipped on Fedora 43.

## v0.1.4 — 2025-03-15

### Added
- Dark mode readability fixes for buttons and form controls.
- Database backup with a timestamped default filename.

### Fixed
- Various compiler warnings resolved.

## v0.1.3 — 2025-02-20

### Added
- Linux AppImage build target.
- GitHub Actions and Forgejo Actions CI/CD pipelines.
- Makefile with `appimage`, `deb`, `rpm`, `dev`, and `clean` targets (auto-patches strip for Fedora/Arch).

## v0.1.2 — 2025-01-30

### Added
- Wildcard search — `*` glob is translated to SQL `LIKE`.
- Progress bar with percentage during import (audio and TXT).
- Clearing the search box resets the results list.
