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
    let (cfg, msg) = server_manager::state::load_config(&paths);
    assert_eq!(cfg.schema_version, 2);
    assert!(msg.unwrap().contains("valid TOML"));
    assert!(std::fs::read_dir(dir.path()).unwrap()
        .any(|e| e.unwrap().file_name().to_string_lossy().starts_with("config.toml.bak-")));
}