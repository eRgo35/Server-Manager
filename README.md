# Server Manager

Spiritual successor to the 2021 Server Manager (C# / WinForms). From one
small window: see whether your machines are up, and flip their power —
Wake-on-LAN to turn them on, SSH to shut down / reboot / read basic
stats. Runs on Linux, Android, and (later) Windows.

**Status:** v2 in design. See
[`docs/superpowers/specs/2026-09-10-server-manager-v2-design.md`](docs/superpowers/specs/2026-09-10-server-manager-v2-design.md)
for the full design, and `docs/DRAFT_01.MD`–`DRAFT_03.MD` for the
decision record.

Stack: Rust + Tauri v2, plain-JS frontend (Vite), `russh`. MIT licensed.

## Building

Build tasks run through [`just`](https://github.com/casey/just).

```
just test              # unit tests
just test-integration  # Docker-based SSH tests
just build-linux       # AppImage + .rpm + PKGBUILD
just build-android     # signed APK (Android SDK/NDK required)
```

### Linux

`just build-linux` runs two steps:

1. `cargo tauri build --bundles appimage,rpm` — release build plus
   bundling (needs `rpmbuild`; Tauri downloads the AppImage tooling on
   first run). Artifacts land in `src-tauri/target/release/bundle/`
   (`appimage/*.AppImage`, `rpm/*.rpm`).
2. `packaging/build-pkgbuild.sh` — renders `packaging/out/PKGBUILD`
   from the template with the current version and copies in the release
   binary, desktop file and icon. On an Arch system with `makepkg` it
   also builds the package; elsewhere it just emits the `PKGBUILD`.

Android (APK) is deferred to M2; Windows (portable `.exe` via CI) to
M3. There is no auto-update — users re-download.

The `v1` line lives on the `master` branch and the `old-*` archive
branches.
