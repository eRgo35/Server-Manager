default:
    @just --list

# install the extra tooling this repo needs
setup:
    cargo install just tauri-cli --locked || true
    rustup component add rustfmt clippy

fmt:
    cargo fmt --all

lint:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings

test:
    cargo test --workspace

# tauri dev shell; DMABUF renderer disabled — WebKitGTK crashes the GTK loop
# on Wayland ("Error 71 dispatching to Wayland display") without this
dev:
    WEBKIT_DISABLE_DMABUF_RENDERER=1 cargo tauri dev

# plain cargo run of the tauri binary (no devUrl); same Wayland workaround.
# The binary embeds ui/dist at compile time, so refresh the dist first —
# otherwise a stale frontend ships in the window.
run:
    cd ui && npm run build
    WEBKIT_DISABLE_DMABUF_RENDERER=1 cargo run

# docker-backed SSH tests, opt-in (needs docker; never part of `just test`)
test-integration:
    docker compose -f tests/integration/docker-compose.yml up -d --wait
    # Wait for the SSH banner, not just the TCP port: the port forwards
    # before sshd is ready to speak SSH, so an early connect would ECONNRESET.
    @for i in $(seq 1 60); do if ssh-keyscan -T 1 -p 2222 127.0.0.1 >/dev/null 2>&1; then break; fi; sleep 1; done
    @[ -n "$(ssh-keyscan -T 1 -p 2222 127.0.0.1 2>/dev/null)" ] || { echo "sshd never spoke SSH on 127.0.0.1:2222"; docker compose -f tests/integration/docker-compose.yml down -v; exit 1; }
    @set -e; trap 'docker compose -f tests/integration/docker-compose.yml down -v' EXIT; cargo test --package sm-infra --test ssh_it -- --ignored --test-threads 1

# NO_STRIP: the linuxdeploy AppImage Tauri pins bundles binutils too old to
# strip Fedora's .relr.dyn sections, which fails the whole AppImage bundling
build-linux:
    NO_STRIP=1 cargo tauri build --bundles appimage,rpm
    bash packaging/build-pkgbuild.sh

build-android:
    cargo tauri android build --apk
