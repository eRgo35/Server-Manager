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
Detailed per-platform instructions land with the M1 implementation.

```
just test              # unit tests
just test-integration  # Docker-based SSH tests
just build-linux       # AppImage + .rpm + PKGBUILD
just build-android     # signed APK (Android SDK/NDK required)
```

The `v1` line lives on the `master` branch and the `old-*` archive
branches.
