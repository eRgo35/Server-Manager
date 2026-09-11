# Integration tests

Docker-backed SSH integration tests for `sm-infra`. Run with:

```
just test-integration
```

The recipe:

1. Starts the sshd container from `docker-compose.yml` (linuxserver
   `openssh-server`, user `tester`, password `testpass`).
2. Waits for port **2222** on 127.0.0.1 (the image generates its host keys on
   first start, so this can take a few seconds).
3. Runs `cargo test -p sm-infra --test ssh_it -- --ignored` (the two tests in
   that file are `#[ignore]`-gated; they exercise `RusshRunner` TOFU and
   `SshStatsProbe` against a real server).
4. Tears the container down with volumes (`down -v`).

Notes:

- **Docker is required.** The recipe fails fast if the daemon is not running.
- Fixed port **2222**: nothing else may be listening on it while the tests run.
- These tests never run in plain `just test` (`cargo test --workspace` only
  compiles them, it does not execute them — they are ignored).