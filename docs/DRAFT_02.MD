# DRAFT_02 — Round 2 Q&A

Round 1 (DRAFT_01.MD) is folded into
`specs/2026-09-10-server-manager-v2-design.md`. These are the questions
those answers opened up, plus a few defaults to confirm. Same drill:
fill in **A:**, say "done".

Most are confirmations — "yes" is a fine answer.

---

## Blockers

### Q1. Windows builds from Linux
Cross-compiling a **Tauri Windows installer** from Linux is not really
supported. Realistic options:
- **CI (GitHub Actions Windows runner)** builds the Windows release
  artifact; locally you get an unpackaged `.exe` via `cargo-xwin` just
  for smoke-testing.
- You keep a Windows VM for packaging.
- Ship Windows as "best effort", built only in CI.

Is the **CI-produces-Windows** path acceptable, or is a real
`just build-windows` that runs entirely on your Linux box a hard
requirement? (If hard, it constrains packaging choices.)

**A:** okay it's not that hard requirement i could use a CI but let's make it delayed not important first hand

### Q2. Artifact formats + WebView2
- Linux: AppImage **and** `.deb`? Or AppImage only?
  **A:** should work on fedora and arch 
- Windows: portable `.exe` for now, NSIS installer later — OK?
  **A:** exe only
- WebView2 (Windows): assume it's present (true on Win 10/11), or ship
  the installer that bootstraps it if missing?
  **A:** i wanna support win10 and win11 only

### Q3. Frontend build step
Plan is **vanilla ES modules + Web Components, no bundler, no build
step** for `ui/`. Alternative is allowing a light bundler (esbuild or
vite) for minification / dev server.
- [ ] No build step, pure static files
- [ ] Light bundler allowed
- [ ] Other:

**A:** ui is build of reusable components. clean ui simillar to vscode and other electron programs. no frameworks here but vite for minification is allowd. Vanilla js components are js classes that construct.

---

## Confirmations

### Q4. Secret storage granularity
Per-machine `secret_mode` (`plaintext` / `keyring` / `prompt`) with a
global default in `[settings]`. Keyring also holds the key passphrase
and sudo password when "remember" is picked. OK?

**A:** OK

### Q5. Stats gathering
One batched SSH command reading `/proc` (uptime, meminfo, `df -B1 /`,
two `/proc/stat` samples for CPU%). Non-Linux machines show
"unavailable" unless a per-machine `stats_cmd` is set that returns a
documented `key=value` format. OK?

**A:** Ok

### Q6. "Map Drive" behavior (Windows)
Which do you want the button to do?
- [ ] Run `net use` against the configured share, fire-and-forget
- [x] Open the Windows "Map network drive" wizard pre-filled
- [ ] Just open Explorer at `\\host` (drop the separate Map Drive button)
- [ ] Other:

**A:** Open wizard yeah 

### Q7. Window behavior + settings
- Desktop window: fixed size (like the original), or small but
  resizable with a minimum size?
  **A:** original size but resizable with minimum size
- Settings: slide-out panel like the original `>` / `<` button, or a
  separate full view?
  **A:** slide out panel but better chevrons haha

### Q8. Machine view
Confirm: the main window manages **one machine at a time**, selected
from a dropdown (like the original). Not a list showing all machines'
status at once. Correct?

**A:** correct

### Q9. Backoff constants
Auto-refresh defaults: poll every **5 s** when reachable, on failure
back off **×2** up to a **5 min** cap, reset on success. Good, or
different numbers?

**A:** good

### Q10. Task runner
`just` (a small, widely-used command runner) as the dev build script,
wrapping cargo / tauri / android commands. New dev-only dependency —
acceptable? (Alternative: a plain `build.sh` + `cargo xtask`.)

**A:** yay! let's use it

---

## Part 5 — Anything else

**Notes:**
