# Manual visual QA checklist

Walk this table before every release. Check the box when the row passes
on the target platform; note failures as issues.

Launch with `just dev` (or `just run`). On Wayland the DMABUF renderer is
disabled by the recipe — if you run `cargo run` bare on Wayland and the
window dies with `Error 71 (Protocol error) dispatching to Wayland
display`, that's the missing workaround, not an app bug.

| # | Check | Pass |
|---|-------|------|
| 1 | Window opens at 360×280; won't shrink below 320×240 | ☐ |
| 2 | Machine dropdown lists every machine in `config.toml` | ☐ |
| 3 | Status flips ONLINE/OFFLINE within `poll_base_secs` of the box going up/down | ☐ |
| 4 | Offline machine's poll interval visibly backs off (status timestamps stretch) | ☐ |
| 5 | Wake sends packets — `sudo tcpdump -ni any udp port 9` on the target host | ☐ |
| 6 | Shutdown works on a real Linux box | ☐ |
| 7 | Reboot works on a real Linux box | ☐ |
| 8 | Sudo-password path works when passwordless sudo is absent | ☐ |
| 9 | Host-key prompt appears on first connect; trust persists across app restart | ☐ |
| 10 | Host-key-changed warning blocks the connection | ☐ |
| 11 | Stats show sane CPU/mem/disk/uptime — both numbers and sparkline modes | ☐ |
| 12 | Open Files opens the file manager at `smb://<host>` | ☐ |
| 13 | Settings slide-out toggles (chevron) | ☐ |
| 14 | Language switch to PL translates the UI | ☐ |
| 15 | Dark and light themes both legible (incl. system-follow) | ☐ |
| 16 | Malformed `config.toml` → `.bak-<ts>` backup + notice + app still starts | ☐ |

Note for 11: stats need a working SSH connection — password (plaintext or
prompt) or key path; prompt mode asks the first time.