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

# docker-backed SSH tests, opt-in (needs docker; never part of `just test`)
test-integration:
    docker compose -f tests/integration/docker-compose.yml up -d
    @port_up=0; for i in $(seq 1 30); do if (exec 3<>/dev/tcp/127.0.0.1/2222) 2>/dev/null; then exec 3>&- 3<&-; port_up=1; break; fi; sleep 1; done; [ "$port_up" = 1 ] || { echo "sshd on 127.0.0.1:2222 did not come up"; exit 1; }
    -cargo test --package sm-infra --test ssh_it -- --ignored --test-threads 1
    docker compose -f tests/integration/docker-compose.yml down -v

build-linux:
    cargo tauri build --bundles appimage,rpm
    bash packaging/build-pkgbuild.sh

build-android:
    cargo tauri android build --apk
