# DRAFT_01 — Requirements Q&A

Working scratch file. Michał fills in answers under each **A:** line.
When done, tell Claude — he reads this, folds it into
`specs/2026-09-10-server-manager-v2-design.md`, and this file stays as
the record. Leave a question blank / write "later" to defer it.

---

## Part 0 — Already decided (from the spec draft)

- **Purpose / kept idea:** spiritual successor to the 2021 Server Manager.
  Watch whether my machines are up, and flip their power (wake / shut
  down) from one place.
- **Feature list (v2):**
  - Wake-on-LAN
  - SSH: basic file exploration ("opens explorer") / power management /
    simple stats
  - Cross-platform
  - Option to mount the drive (Windows OSes)
- **Interface:** GUI — desktop + mobile.
- **Target OSes:** Windows, Linux, Android.
- **Language:** Rust. One framework for all OSes, light framework.
- **Distribution:** single exe file. Config in user home
  (`~/.config` / `%APPDATA%`), per-user. Host definitions in clear TOML params.
- **Architecture intent:** Clean Architecture, core business logic with
  minimal dependencies, framework-independent. GUI components as
  separable units, one per file. Clear folder layout. One simple build
  script.
- **Testing intent:** all functions/structs + GUI testable in code, plus
  visual checks.

---

## Part 1 — Blockers (decide these first)

### Q1. GUI framework choice
Options and their trade-offs:
- **egui/eframe** — truly light, single binary, Android via winit.
  Immediate-mode: no retained "component objects", won't look like a
  native desktop app.
- **Slint** — declarative, "component per file" fits, Android support,
  resembles original best. Licensing: GPL or royalty-free-with-conditions
  / commercial — constrains project license.
- **Dioxus** — good model, mobile support young, heavier.
- **Tauri** — mobile works, but UI is HTML/JS (not Rust, not light).

Which framework, and do you accept its main trade-off?

**A:** I would go with Tauri and plain JS here.

### Q2. Android's real scope
Is Android a **full peer** (every feature) or a **remote-control subset**?
Note: on Android there is no "explorer" to open, no drive mounting, WOL
only works on the same Wi-Fi as the target, ICMP ping is restricted.

- [ ] Full peer — must find Android equivalents for every feature
- [x] Subset — Android does: status view, WOL (LAN only), SSH power, SSH stats. No file explorer, no mount.
- [ ] Other:

**A:** Subset mentioned 

### Q3. Build script scope
What must the "one simple build script" actually produce, and on which
host OS is it run? Is Android allowed to be a separate command
(`cargo apk` / xbuild)?

**A:** Yes, I want to have a README with build instructions for windows linux and android just i wanna have an option to build all those execs on linux without the need to setup 3 machines for testing.

---

## Part 2 — Core data model & mechanisms

### Q4. Host model (the TOML schema)
The 2021 app had TWO addresses per host: `POWER_IP` (BMC/IPMI-style
management interface) and `OS_IP` (the operating system).

- Q4a. Keep the two-address concept, or one host = one machine?
  **A:** I would add an option (drop down kinda) that allows you to select the machine to manage so config has a key for machine id, name and conn params. 
- Q4b. Confirm the per-host fields. Proposed:
  `name`, `mac` (WOL), `broadcast_addr`, `os_host` (ip/hostname),
  `ssh_port`, `ssh_user`, `ssh_auth` (see Q7), `shutdown_cmd` (override),
  `mount` (see Q10), `bmc_host` (optional).
  Add / remove / rename any?
  **A:** why bmc_host? all others im good with them. 
- Q4c. What happens when the TOML is malformed or from an older version
  (refuse & show error / migrate / back up & regenerate)?
  **A:** backup and show option to migrate. Definitely option for v2 up cuz no one is gonna migrate from v1 that's for sure.

### Q5. "Online" detection
The 2021 app used ICMP ping. Unprivileged ICMP is inconsistent across
Windows / Linux / Android.
- Q5a. Definition of "up": ICMP reply / TCP connect to SSH port / either?
  **A:** I don't need ICMP that much I just need a stable way to know the host is up whichever way is the simplest and fastest (C# lib had a problem when once it tried to ping for 10s till conn dropped which blocked the ui thread lol)
- Q5b. Accept "TCP-connect to port 22" as the portable default?
  **A:** yeah
- Q5c. Poll interval (seconds)? Configurable per host?
  **A:** should be configurable also with the autorefresh (smart) with some kind of the incremental backoff to not trigger spam

### Q6. Wake-on-LAN details
- Q6a. LAN-only (same L2 segment), or must it wake hosts on other
  networks too (needs an always-on relay/jump host)?
  **A:** yeah tool is LAN only by design (or works in VPNs if machine is connected - nothing changes architecture-wise here)
- Q6b. Magic packet target: L2 broadcast / directed broadcast / unicast?
  **A:** i found problem trying to reach server mac and ip directly had to send the packet to broadcast ip idk why
- Q6c. SecureOn password support needed? IPv6?
  **A:** No secureon pass needed no ipv6 needed

### Q7. SSH authentication
- Q7a. Which auth methods: key file / SSH agent / password / encrypted
  key with passphrase? (pick all that apply)
  **A:** plain password, key file and ssh agent (linux only im sure) also it would be cool if using ssh key to all to cache the passphrase on windows and android
- Q7b. Where are secrets stored:
  - [x] Plaintext in the TOML
  - [x] OS keyring (Secret Service / Win Credential Manager / Android Keystore)
  - [x] Never stored — prompt every time / rely on agent
  - [ ] Other:
  **A:** all three options just configurable from the settings
- Q7c. Host key verification: use `~/.ssh/known_hosts` / app-managed TOFU
  (trust on first use, warn on change) / no verification (not recommended)?
  **A:** yeah app should ask on first connection then just trust (same as classic ssh connection)

### Q8. Power management over SSH
- Q8a. Assume passwordless `sudo` for shutdown/reboot, or prompt for a
  sudo password, or require the SSH user to already have the right?
  **A:** shutdown should be passwordless if it requires sudo inform and ask for password only if set in settings. 
- Q8b. Need reboot in addition to shutdown?
  **A:** yeah
- Q8c. Non-Linux servers in scope (Windows `shutdown /s`, BSD)? Or Linux
  servers only?
  **A:** that's why I wanted to add the custom command override

### Q9. "Simple stats"
- Q9a. Which metrics: CPU %, memory, disk usage, uptime, load average,
  something else?
  **A:** cpu mem disk uptime
- Q9b. Gather by running commands (`uptime`, `free`, `df`) or reading
  `/proc`? (commands = more portable)
  **A:** whatever is simpler, help me decide
- Q9c. Display as plain numbers or small graphs/history?
  **A:** small graphs with history or numbers (choose in settings)

### Q10. File exploration & drive mounting
- Q10a. "(Windows OSes)" — is the CLIENT Windows, or is the SERVER
  Windows? The 2021 app opened `\\<ip>\` (Windows server + SMB shares).
  **A:** YES plus add option shortcut to map a drive with these params
- Q10b. Protocol: SMB/CIFS (needs Samba on the remote) or SSHFS/WinFSP
  (works with just SSH)?
  **A:** SMB by default switch to sshfs in settings
- Q10c. "opens explorer" = launch the OS file manager at a path, OR an
  in-app SFTP file browser? (or both, per platform)
  **A:** No SFTP all opens of file manager at a path
- Q10d. Mount lifecycle: pick drive letter (Windows)? store share
  credentials where? auto-unmount on exit? persist the mount?
  **A:**none of these. This handles the OS manually for us its out of scope

---

## Part 3 — Architecture & quality

### Q11. Crate / layer split
Proposed layers (Clean Architecture in Rust idiom):
- `core` — domain types (`Host`, `Status`, `Stats`), zero I/O
- `services` — traits: `Waker`, `StatusProbe`, `SshSession`, `Mounter`
- `infra` — concrete impls (wol, ping/tcp, russh, smb/sshfs)
- `ui` — the GUI, one module/file per screen/component
- `app` — wiring, config load/save

Accept this? Change any names/boundaries? One crate with modules, or a
Cargo workspace with multiple crates?

**A:** Accept

### Q12. Concurrency model
Polling many hosts + async SSH needs an async runtime (likely `tokio`)
running alongside the GUI event loop, with a channel to push updates to
the UI. Accept `tokio` as a dependency? Or prefer `smol` / thread pool?

**A:** Accepting tokio

### Q13. GUI testing — what does "visual checks" mean concretely?
- [ ] Snapshot/screenshot tests (e.g. `egui_kittest`, Slint test API)
- [ ] Headless widget-logic tests only
- [x] Manual visual checklist per release
- [ ] Combination:

**A:** Manual visual checklist per release

### Q14. Integration testing SSH / WOL
OK to spin up a container (Docker) with an SSH server for integration
tests in CI? WOL is hard to assert — mock the socket send and just
verify the packet bytes?

**A:** docker acceptible but should be a separate flow from other tests with clean docs

### Q15. Non-functional bounds
- Q15a. Expected number of hosts (rough): under 10 / tens / hundreds?
  **A:** under 10 (prob often just one)
- Q15b. Behavior when a configured host never responds — grey it out and
  keep polling, or back off?
  **A:** polling backed off (grayed out)
- Q15c. Dark mode / theming needed?
  **A:** of course
- Q15d. Internationalisation, or English only (like the original)?
  **A:** english and polish
- Q15e. Logging — to a file? verbosity control?
  **A:** only latest.log in same config path as config file

---

## Part 4 — Packaging & meta

### Q16. Distribution & signing
- Q16a. Code signing for Windows (avoids SmartScreen) — in scope now,
  later, or never?
  **A:** later
- Q16b. Android APK signing / distribution — Play Store, or sideload APK
  from GitHub releases?
  **A:** it's gonna be only sideloaded from Github releases
- Q16c. Auto-update mechanism — in scope, or "user re-downloads"?
  **A:** user redownloads for now (or uses app like obtainium)

### Q17. License
Orphan branch has no LICENSE yet. `master` had one. Which license for
v2? (Note: interacts with Q1 — Slint especially.)

**A:** MIT

### Q18. Name
Keep "Server Manager"? (There is well-known unrelated software by that
name.) New name?

**A:** Server Manager

### Q19. "Resemblance to the original" — how literal?
The 2021 UI: fixed ~289×222 window, ONLINE/OFFLINE status labels with
green/red colour, power on/off buttons, a slide-out (`>` / `<`) settings
panel with the two IP fields and a Save button.

Which of these must survive in v2, and which are just "the vibe"?

**A:** The overall layout and windows size, so the app has simillar layout of the buttons top to bottom

---

## Part 5 — Anything Michał wants to add

**Notes:**
