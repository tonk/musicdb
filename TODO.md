# MusicDB — TODO

## Done
- [x] Fix compiler warnings
- [x] Allow wildcard search — easy part (`*` glob → SQL `LIKE`)
- [x] Show progress (bar + percentage) during import (audio and TXT)
- [x] Clearing the search box resets the results
- [x] Linux AppImage build target
- [x] GitHub Actions and Forgejo Actions CI/CD
- [x] Makefile with `appimage`, `deb`, `rpm`, `dev`, `clean` targets (auto-patches strip for Fedora/Arch)
- [x] Database backup with timestamped default filename
- [x] Dark mode button/form-control readability fixes

## Pending
- [ ] Allow wildcard search — hard part (full regular expression support)
- [ ] Set version number from current Git tag; show it in the status bar
- [ ] When importing from an audio file, the "BY FORMAT" is not ""Other"
      but the audio file format (`mp3`, `flac`, `ogg`, etc)
- [ ] In "Settings" add an option for the number of displayed items on
      a single page
