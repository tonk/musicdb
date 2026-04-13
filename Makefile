.PHONY: all dev build appimage deb rpm clean help

LINUXDEPLOY := $(HOME)/.cache/tauri/linuxdeploy-x86_64.AppImage

# ─── Default ──────────────────────────────────────────────────────────────────
all: build

# ─── Development ──────────────────────────────────────────────────────────────
dev:
	npm run tauri dev

# ─── Production builds ────────────────────────────────────────────────────────
build:
	APPIMAGE_EXTRACT_AND_RUN=1 npm run tauri build

# AppImage build with automatic strip compatibility fix.
#
# Fedora 38+ (and other modern distros) ship binutils that produce ELF files
# with SHT_RELR (type 0x13) relocation sections.  The strip binary bundled
# inside linuxdeploy's AppImage is old and fails on these files.
#
# Tauri downloads linuxdeploy once to ~/.cache/tauri/linuxdeploy-x86_64.AppImage.
# When run with APPIMAGE_EXTRACT_AND_RUN=1 (or --appimage-extract-and-run) the
# AppImage runtime extracts its payload to /tmp/appimage_extracted_<MD5>/ where
# <MD5> is the md5sum of the AppImage file.  The bundled strip lives in that
# temp dir and is used in preference to the system strip.
#
# Fix: a background polling loop detects when the temp dir's strip appears and
# immediately replaces it with the system strip (/usr/bin/strip).  The loop
# runs for the duration of the Tauri build and exits cleanly afterward.
appimage:
	@LINUXDEPLOY="$(LINUXDEPLOY)"; \
	if [ ! -f "$$LINUXDEPLOY" ]; then \
	    APPIMAGE_EXTRACT_AND_RUN=1 npm run tauri build -- --bundles appimage; \
	    exit $$?; \
	fi; \
	HASH=$$(md5sum "$$LINUXDEPLOY" | awk '{print $$1}'); \
	STRIP_PATH="/tmp/appimage_extracted_$${HASH}/usr/bin/strip"; \
	( while true; do \
	    if [ -f "$$STRIP_PATH" ]; then \
	        cp /usr/bin/strip "$$STRIP_PATH" 2>/dev/null; \
	        break; \
	    fi; \
	    sleep 0.05; \
	  done ) & \
	PATCHER_PID=$$!; \
	APPIMAGE_EXTRACT_AND_RUN=1 npm run tauri build -- --bundles appimage; \
	STATUS=$$?; \
	kill $$PATCHER_PID 2>/dev/null; \
	exit $$STATUS

deb:
	npm run tauri build -- --bundles deb

rpm:
	npm run tauri build -- --bundles rpm

# ─── Housekeeping ─────────────────────────────────────────────────────────────
clean:
	cargo clean --manifest-path src-tauri/Cargo.toml
	rm -rf dist

help:
	@echo "Targets:"
	@echo "  dev        Start development server with hot-reload"
	@echo "  build      Build all configured bundles (deb, rpm, appimage)"
	@echo "  appimage   Build Linux AppImage only (auto-fixes strip on Fedora/Arch)"
	@echo "  deb        Build Debian package only"
	@echo "  rpm        Build RPM package only"
	@echo "  clean      Remove Rust build artefacts and frontend dist"
