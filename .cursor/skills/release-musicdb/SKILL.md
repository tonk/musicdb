---
name: release-musicdb
description: Bump MusicDB release versions, update changelog, commit, tag, and push. Use when the user asks to release a new version like "release 0.1.8" or requests version/tag publishing.
---

# Release MusicDB

## Purpose

Publish a new MusicDB release with consistent version metadata and git tags.

## Release checklist

When the user provides a target version (for example `0.1.8`):

1. Update version fields:
   - `package.json` -> `version`
   - `package-lock.json` -> root `version` and `packages[""].version`
   - `src-tauri/tauri.conf.json` -> `version`
   - `src-tauri/Cargo.toml` -> `package.version`
   - `src-tauri/Cargo.lock` -> `[[package]] name = "musicdb"` version
2. Add/update `CHANGELOG.md` entry for the new version.
3. Validate:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
4. Prepare git:
   - Stage only release-relevant files (never `node_modules`).
   - Commit with message: `release: v<version>`.
5. Tag and push:
   - Create annotated tag `v<version>`.
   - Push branch and tags.

## Guardrails

- Do not include `node_modules` changes in release commits.
- If the tag already exists, stop and ask user whether to replace it.
- If working tree includes unrelated changes, stage only the files required for the release.
- Keep changelog entry concise: Added / Changed / Fixed bullets based on actual shipped changes.
