You are performing a WarmDesk release. The version to release is: $ARGUMENTS

Follow these steps in order:

## 1. Gather changes
Run `git log --oneline $(git describe --tags --abbrev=0)..HEAD` to see all commits since the last tag. Use these to write the release notes.

## 2. Update CHANGELOG.md
Add a new section at the top (below the `# Changelog` heading) in this format:

```
## v{version} — {today's date YYYY-MM-DD}

### Added
- ...

### Fixed
- ...

### Changed
- ...
```

Only include sections that have entries. Be specific and user-facing in the descriptions — same style as existing entries.

## 3. Update README.md
- If any new features were added, update the **Features** list to reflect them.
- No other sections need changing for a routine release.

## 4. Update documentation
- If any changes where made, that need an update on the documentation, update the documentation

## 5. Commit and tag
```bash
git add CHANGELOG.md README.md what.md
git commit -m "chore: release v{version} — CHANGELOG, README"
git tag -a v{version} -m "Release v{version}"
```

## 6. Push
```bash
git push && git push --tags
```

Report what was pushed when done.
