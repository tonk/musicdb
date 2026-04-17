---
name: musicdb-release
description: Handles the release process for MusicDB. Use when the user wants to cut a new release, update the changelog, and tag the version in git.
---

# MusicDB Release Workflow

This skill automates the multi-step process of releasing a new version of MusicDB, ensuring consistency across the changelog, documentation, and version tags.

## Workflow

Follow these steps in order when a release is requested:

### 1. Gather Changes
Analyze the commits since the last release tag to determine the scope of changes.
- Command: `git log --oneline $(git describe --tags --abbrev=0)..HEAD`
- Categorize changes into: **Added**, **Fixed**, **Changed**.

### 2. Update Version Numbers
Update the version in both the frontend and backend configurations.
- `package.json`: Update `"version"`
- `src-tauri/tauri.conf.json`: Update `"version"`
- `src-tauri/Cargo.toml`: Update `[package] version`

### 3. Update CHANGELOG.md
If `CHANGELOG.md` does not exist, create it. Add a new section at the top:
```markdown
## v{version} — {YYYY-MM-DD}

### Added
- [Feature description]

### Fixed
- [Bug fix description]

### Changed
- [Change description]
```

### 4. Update README.md and Documentation
- Update the **Features** list in `README.md` if new capabilities were added.
- Review and update `docs/*.md` if any changes affect building or user guidance.

### 5. Commit and Tag
Stage all changed files and create a release commit and tag.
- Commit message format: `chore: release v{version}`
- Tag format: `v{version}`

### 6. Push
Push the changes and the new tag to the remote repository.
- Command: `git push && git push --tags`

## Example Usage
"Release version 0.1.2 with the new MusicBrainz lookup and performance fixes."
