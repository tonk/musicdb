.PHONY: all dev build deb rpm install setup clean check

# Tauri v2 system dependencies (Fedora/RHEL)
TAURI_DEPS = webkit2gtk4.1-devel \
             openssl-devel \
             libappindicator-gtk3-devel \
             librsvg2-devel \
             patchelf \
             curl \
             wget \
             file \
             gcc \
             gcc-c++ \
             nodejs \
             npm

all: build

setup:
	sudo dnf install -y $(TAURI_DEPS)

install:
	npm install

dev: install
	npm run tauri dev

build: install
	npm run tauri build -- --no-bundle

deb: install
	npm run tauri build -- --bundles deb

rpm: install
	npm run tauri build -- --bundles rpm

check:
	npm run build
	cd src-tauri && cargo check

clean:
	rm -rf dist node_modules
	cd src-tauri && cargo clean
