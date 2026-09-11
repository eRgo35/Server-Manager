#[test]
fn garbage_config_is_backed_up_and_reset() {
    let dir = tempfile::tempdir().unwrap();
    let paths = server_manager::state::Paths {
        dir: dir.path().into(),
        config: dir.path().join("config.toml"),
        known_hosts: dir.path().join("known_hosts"),
        log: dir.path().join("latest.log"),
    };
    std::fs::write(&paths.config, "== broken ==").unwrap();
    let loaded = server_manager::state::load_config(&paths);
    assert_eq!(loaded.cfg.schema_version, 2);
    assert!(loaded.notice.unwrap().contains("valid TOML"));
    assert!(std::fs::read_dir(dir.path()).unwrap().any(|e| e
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with("config.toml.bak-")));
}

#[test]
fn too_new_config_sets_read_only_reason() {
    let dir = tempfile::tempdir().unwrap();
    let paths = server_manager::state::Paths {
        dir: dir.path().into(),
        config: dir.path().join("config.toml"),
        known_hosts: dir.path().join("known_hosts"),
        log: dir.path().join("latest.log"),
    };
    std::fs::write(&paths.config, "schema_version = 99\n").unwrap();
    let loaded = server_manager::state::load_config(&paths);
    assert_eq!(loaded.cfg.schema_version, 2, "runs on defaults");
    let reason = loaded
        .read_only_reason
        .expect("TooNew must set read_only_reason");
    assert!(reason.contains("v99"));
    // The user's newer file must NOT have been overwritten.
    assert_eq!(
        std::fs::read_to_string(&paths.config).unwrap(),
        "schema_version = 99\n"
    );
}
