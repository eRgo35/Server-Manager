#!/usr/bin/env bash
# Generate packaging/out/PKGBUILD (+ binary, desktop file, icon) after
# `cargo tauri build`, and run `makepkg -f` when makepkg is available.
set -euo pipefail
cd "$(dirname "$0")/.."

version=$(sed -n 's/.*"version": *"\([^"]*\)".*/\1/p' src-tauri/tauri.conf.json | head -1)
bin=target/release/server-manager

if [ ! -x "$bin" ]; then
    echo "error: release binary not found at $bin — run 'just build-linux' (or 'cargo tauri build') first" >&2
    exit 1
fi

out=packaging/out
mkdir -p "$out"
sed "s/@VERSION@/$version/" packaging/PKGBUILD.template > "$out/PKGBUILD"
cp "$bin" "$out/server-manager"
cp packaging/server-manager.desktop "$out/server-manager.desktop"
cp src-tauri/icons/icon.png "$out/server-manager.png"
echo "generated $out/PKGBUILD (version $version)"

if command -v makepkg >/dev/null 2>&1; then
    (cd "$out" && makepkg -f)
else
    echo "note: makepkg not installed — PKGBUILD was generated but the .pkg.tar.zst was not built"
fi