# Server Manager v2 — Design

Status: **FINAL** — ready for implementation planning. All requirements
resolved over three Q&A rounds (`docs/DRAFT_01.MD` – `DRAFT_03.MD`,
kept as the decision record).

---

## 1. Context

Spiritual successor to the 2021 "Server Manager" (C# / WinForms). The
kept idea: from one small window, see whether my machines are up and
flip their power — wake them, shut them down.

v2 keeps that core and adds: SSH-driven reboot / shutdown / stats, a
"open the files on that box" shortcut, optional network-drive mapping,
and it runs on Linux, Windows and Android instead of just Windows.

Name stays **Server Manager**. License: **MIT**.

### What v2 is not

- Not a fleet/monitoring tool. Expected scale: **under 10 machines,
  often just one.**
- Not a remote-wake-over-the-internet tool. **LAN only** (a VPN that
  puts the client on the same L2 as the target counts as LAN; nothing
  in the design changes for that case).
- Not an SFTP client. File access = launching the OS file manager at a
  network path. No in-app file browser.
- Not a drive manager. "Map drive" hands off to the OS; the app does
  not track drive letters, credentials, or unmounting.

---

## 2. Stack

| Layer | Choice |
|---|---|
| Shell / packaging | **Tauri v2** (desktop + Android from one codebase) |
| Backend / core logic | **Rust** |
| Frontend | **Plain JS, no TypeScript** (JSDoc for type hints). No UI framework. Reusable components as ES6 classes extending `HTMLElement` (custom elements), one per file. Built with **Vite** (dev server + minified static production build). Visual target: clean, compact, panel-based — VSCode / Electron-app feel. |
| SSH | **russh** (pure-Rust, async, cross-compiles to Android cleanly; no libssh2/C dependency) |
| Async runtime | **tokio** |
| Config | **TOML** |

Trade-off accepted: Tauri means the UI is HTML/JS/CSS rather than
Rust, and there is a system-WebView dependency (WebView2 on Windows,
system WebView on Android, WebKitGTK on Linux). In return: real
cross-platform including Android, small binaries, and a clean split
between a testable Rust core and a thin view layer.

The "GUI components as separable units, one per file" intent (from the
original draft) is honored on the JS side as one custom element per
file, and on the Rust side as one module per concern.

---

## 3. Architecture

Cargo workspace. Clean-architecture layering; `core` has no I/O and no
framework types.

```
server-manager/
├── crates/
│   ├── core/        # domain: Machine, Status, Stats, AuthMode,
│   │                #   BackoffState, config schema + migration.
│   │                #   Pure. No tokio, no russh, no tauri.
│   ├── services/    # trait definitions the app depends on:
│   │                #   Waker, StatusProbe, SshRunner, StatsProbe,
│   │                #   SecretStore, HostKeyStore, FileOpener
│   └── infra/       # concrete impls: wol (UDP magic packet),
│                    #   tcp_probe, russh_runner, stats parsers,
│                    #   keyring, known_hosts (TOFU), os file-manager
│                    #   launcher / `net use`
├── src-tauri/       # Tauri app: commands, event pump, wiring,
│                    #   config load/save, logging
├── ui/              # frontend (Vite project)
│   ├── package.json
│   ├── vite.config.js
│   ├── index.html
│   └── src/
│       ├── main.js
│       ├── components/  # one custom-element class per file
│       ├── i18n/        # en.json, pl.json
│       └── styles/
├── tests/           # cross-crate + integration
├── docs/
├── justfile         # the "one build script" — see §10
└── Cargo.toml
```

**Data flow:** frontend calls Rust via Tauri `invoke` commands
(`wake`, `power`, `open_files`, `map_drive`, `get_state`,
`save_settings`, `upsert_machine`, …). Long-running things (status
polling, stats sampling) run in tokio tasks in the backend and push
results to the frontend as Tauri **events** (`status://update`,
`stats://update`); the UI is a pure render of the last event received.
No business logic in JS.

Where a command needs something from the user mid-flight, it returns a
**structured error code** the frontend recognises rather than a prose
string: `HOSTKEY_UNTRUSTED`, `HOSTKEY_CHANGED`,
`SECRET_REQUIRED:<kind>`. The frontend shows the matching prompt
(trust dialog / password field) and retries.

---

## 4. Data model & config

### 4.1 Location

| OS | Directory |
|---|---|
| Linux | `~/.config/server-manager/` |
| Windows | `%APPDATA%\server-manager\` |
| Android | app-private storage |

Files in that directory:

- `config.toml` — machines + settings
- `known_hosts` — app-managed SSH host-key fingerprints (TOFU)
- `latest.log` — single rolling log file, overwritten each run

### 4.2 Schema

```toml
schema_version = 2

[settings]
language          = "en"          # "en" | "pl"
theme             = "system"      # "system" | "light" | "dark"
stats_display     = "graph"       # "graph" | "numbers"
poll_base_secs    = 5             # see §6.2 for backoff
default_secret_mode = "keyring"   # "plaintext" | "keyring" | "prompt"
mount_protocol    = "smb"         # "smb" | "sshfs"

[[machine]]
id            = "nas"             # stable key, used by the selector
name          = "Home NAS"        # display name
mac           = "AA:BB:CC:DD:EE:FF"
broadcast_addr = "255.255.255.255"  # optional, this is the default
os_host       = "192.168.1.10"   # ip or hostname
ssh_port      = 22
ssh_user      = "mike"
key_path      = "~/.ssh/id_ed25519" # optional; else agent / autodiscovery / password
secret_mode   = "keyring"         # optional, overrides settings default
shutdown_cmd  = "shutdown -h now" # optional override (default shown)
reboot_cmd    = "shutdown -r now" # optional override (default shown)
stats_cmd     = ""                # optional; required for non-Linux stats
share_path    = "\\\\192.168.1.10\\media"  # optional, for Open Files / Map Drive
```

- **One `[[machine]]` = one physical box.** The 2021 two-address model
  (POWER_IP vs OS_IP) is dropped. There is no `bmc_host`. The UI shows
  one machine at a time, chosen from a dropdown (Q4a).
- Secrets themselves are **not** in this schema unless `secret_mode =
  "plaintext"` (then an `ssh_password` / `key_passphrase` /
  `sudo_password` key may appear on the machine). Otherwise they live
  in the keyring or are prompted.

### 4.3 Malformed / old config (Q4c)

On load:
- Unparseable TOML → back up to `config.toml.bak-<timestamp>`, start
  from defaults, show a non-blocking error in the UI.
- `schema_version` older than current → back up, run the migration
  chain (`v2→v3→…`), write the upgraded file, tell the user what
  happened. **No v1→v2 migration** — v1 users are assumed nonexistent.
- `schema_version` newer than the app supports → refuse to write,
  read-only mode, tell the user to upgrade the app.

---

## 5. Wake-on-LAN

- Magic packet: 6×`0xFF` + 16×target MAC, sent UDP to
  `broadcast_addr` on ports **9 and 7**.
- Default `broadcast_addr` is `255.255.255.255` (limited broadcast).
  Per-machine override for a directed broadcast if the user needs it.
  Rationale: user testing showed unicast-to-host did not wake reliably;
  broadcast did.
- No SecureOn password. No IPv6.
- LAN/VPN only — documented, not enforced.

---

## 6. Status detection

### 6.1 Mechanism

**TCP connect to `os_host:ssh_port`** with a short timeout (2 s
default). Reachable ⇒ "ONLINE", else "OFFLINE". No ICMP. All probing
is async on tokio tasks — the UI thread is never blocked (the 2021
app's 10 s ping hang must not recur).

### 6.2 Smart auto-refresh

Per machine:
- Poll every `poll_base_secs` (default 5, configurable) while the last
  result was ONLINE or the machine was just acted on.
- On consecutive failures, exponential backoff ×2 up to a **5 min
  cap**. Reset to base on the first success.
- A greyed-out (offline) machine keeps polling, just at the backed-off
  interval.
- Manual "refresh now" button bypasses the backoff once.

Backoff constants: base 5 s (from `poll_base_secs`), factor ×2, cap
300 s.

---

## 7. SSH

### 7.1 Auth

Supported methods: **password**, **key file**, **key file +
passphrase**, **ssh-agent** (desktop only — no agent on Android).

Secret storage is a per-machine mode with a global default
(`default_secret_mode`):

| Mode | Behavior |
|---|---|
| `plaintext` | secret written into `config.toml` |
| `keyring` | OS keyring — Secret Service (Linux), Credential Manager (Windows), Keystore (Android) |
| `prompt` | never stored; asked once per app session, kept in memory only |

When `keyring` is chosen, the same store holds the SSH password *or*
the key passphrase *or* the sudo password as needed — this is the
"cache the passphrase on Windows and Android" ask.

### 7.2 Host key verification

App-managed **TOFU**: on first connect, show the fingerprint and ask
to trust; store it in `known_hosts`; on a later mismatch, block the
connection and warn loudly. Mirrors normal `ssh` first-connect
behavior. The system `~/.ssh/known_hosts` is not used (Android has
none).

### 7.3 Power

- Shutdown runs `shutdown_cmd` (default `shutdown -h now`), reboot runs
  `reboot_cmd` (default `shutdown -r now`).
- Try without `sudo` first. If it fails on permissions **and** a sudo
  password is configured for that machine, retry via `sudo -S`.
  Otherwise surface the permission error and tell the user to either
  set up passwordless sudo or add a sudo password in settings.
- Non-Linux / BSD boxes are handled through the per-machine command
  override (e.g. `shutdown /s /t 0` for Windows).

### 7.4 Stats

Metrics: **CPU %, memory, disk, uptime.**

Gathering: **one batched SSH exec** per sample, parsed in `infra`.
Read `/proc` directly in a single command — no assumptions about which
userland tools exist. Sections are delimited by `---SM-*` sentinel
lines so the parser is unambiguous:

```
echo '---SM-UPTIME'; cat /proc/uptime;
echo '---SM-MEM';    cat /proc/meminfo;
echo '---SM-DISK';   df -B1 --output=size,used / | tail -1;
echo '---SM-CPU1';   head -1 /proc/stat;
sleep 0.2;
echo '---SM-CPU2';   head -1 /proc/stat
```

(two `/proc/stat` reads 200 ms apart for a CPU delta). The exact string
lives in code as `sm_core::stats::PROC_STATS_CMD`.

Non-Linux machines: stats are shown as "unavailable" unless
`stats_cmd` is set and returns a documented `key=value` format.

Display: `stats_display` setting — plain numbers, or small sparklines
with in-memory history (history is session-only, not persisted).

---

## 8. File access & drive mapping — desktop only

Excluded on Android.

- **Open Files** button → launches the OS file manager at the network
  path:
  - Windows: `explorer \\<os_host>` (or `share_path` if set)
  - Linux: `xdg-open smb://<os_host>`
- **Map Drive** (Windows only) → open the Windows **"Map network
  drive" wizard** and copy `share_path` to the clipboard, with a toast
  telling the user to paste it (Windows offers no supported way to
  pre-fill that wizard). Windows is a later milestone (§12).
- `mount_protocol` setting: `smb` (default) or `sshfs`. `sshfs` mode is
  documented as requiring sshfs / WinFSP already installed; the app
  does not install it.
- The OS owns credentials, drive letters, and unmounting.

---

## 9. UI

### 9.1 Layout

Keep the original's shape: a **small window** with controls stacked
**top to bottom**:

1. Machine selector (dropdown)
2. Status line — `ONLINE` green / `OFFLINE` red
3. Wake / Shutdown / Reboot
4. Open Files / Map Drive (hidden on Android)
5. Stats area (numbers or sparklines)
6. Settings — **slide-out panel** toggled by a chevron control
   (spiritual successor to the original `>` / `<` button, done nicely).

Desktop window opens at **360×280**, minimum **320×240**, resizable
above that. Android: full screen, same vertical order, Open Files /
Map Drive omitted.

### 9.2 Theming

Light + dark, following the OS by default (`theme = "system"`),
override in settings.

### 9.3 i18n

**English + Polish.** Runtime JSON dictionaries in `ui/src/i18n/`,
language switch in settings. Rust surfaces errors as stable
codes/enums; the frontend translates them.

### 9.4 Logging

Single `latest.log` in the config directory, overwritten each run.
Verbosity via an env var or a hidden setting (default: info).

---

## 10. Build & packaging

### 10.1 Task runner

A **`justfile`** is the "one simple build script":

- `just test` — all unit tests
- `just test-integration` — Docker-based SSH tests (separate, see §11)
- `just build-linux` — AppImage + `.rpm` + `PKGBUILD`
- `just build-android` — signed APK (needs Android SDK/NDK)
- `just build-windows` — Windows `.exe` (later; see below)

### 10.2 Cross-building from Linux

- **Linux + Android**: native on the Linux host — these are the
  first-class targets, built and tested locally with no second machine.
  Android uses the Tauri Android tooling + NDK.
- **Windows**: **deferred, low priority.** Cross-compiling a Tauri
  Windows build from Linux is not well supported, so when it lands it
  will be produced by a **GitHub Actions Windows runner**. No local
  `just build-windows` is required in the meantime. This does not block
  Linux/Android work.

### 10.3 Artifacts & distribution

- **Linux**: AppImage (portable, Fedora + Arch), `.rpm` (Fedora), and
  an Arch `PKGBUILD` — all from the start.
- **Windows** (later): portable `.exe` only — no installer.
  Supported: **Windows 10 and 11 only**, WebView2 assumed present
  (evergreen on 11, delivered via Windows Update on 10); the runtime
  requirement is documented, not bundled.
- **Android**: **sideload APK from GitHub Releases** only (no Play
  Store), signed with a release keystore held in CI secrets. Obtainium
  is a fine install path for users.
- **No auto-update.** User re-downloads.
- Releases built by a GitHub Actions matrix.

---

## 11. Testing

- **Rust unit tests** cover the parts that carry the risk: WOL packet
  bytes, config parse + migration chain, `/proc` stats parsers, backoff
  state machine, TOFU accept/mismatch logic, command-override
  resolution.
- **SSH integration tests** run against a Docker `sshd` container.
  This is a **separate CI job / separate `just` target** with its own
  README section, not part of `just test`.
- **WOL** is asserted at the byte level (mock the UDP socket, check the
  packet); actually waking hardware is out of scope for CI.
- **GUI**: a **manual visual checklist per release**, kept in
  `docs/`. Frontend logic is deliberately thin; a JS test runner is
  added only if a real helper (e.g. a formatter) appears.

---

## 12. Milestones

Delivered **sequentially** — each milestone fully working before the
next begins.

- **M1 — Linux core.** Workspace + crates, config load/save/migrate,
  TCP status probe + smart backoff, WOL, russh connect + TOFU, power
  (shutdown/reboot with sudo fallback), `/proc` stats, the Tauri shell,
  the full UI (machine selector, status, actions, stats, slide-out
  settings), EN+PL, theming, logging. AppImage + `.rpm` + `PKGBUILD`.
  Manual visual checklist. Docker SSH integration tests.
- **M2 — Android.** Same codebase; drop Open Files / Map Drive; verify
  keyring→Keystore, WOL broadcast on Wi-Fi, layout at phone size.
  Signed APK via CI, GitHub Releases.
- **M3 — Windows.** CI Windows runner, portable `.exe`, Open Files +
  Map Drive wizard, keyring→Credential Manager. Win 10/11.

The implementation plan that follows this spec covers **M1 only**;
M2 and M3 get their own plans when M1 is done.
