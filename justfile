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

# docker-backed SSH tests, opt-in
test-integration:
    docker compose -f tests/integration/docker-compose.yml up -d --wait
    -cargo test --package sm-infra --test ssh_it -- --ignored --test-threads 1
    docker compose -f tests/integration/docker-compose.yml down

build-linux:
    cargo tauri build --bundles appimage,rpm
    bash packaging/build-pkgbuild.sh

build-android:
    cargo tauri android build --apk
