//! Fails the build if sm-core grows a forbidden dependency.
#[test]
fn no_forbidden_deps() {
    let manifest = include_str!("../Cargo.toml");
    for bad in ["tokio", "russh", "tauri", "reqwest", "keyring"] {
        assert!(!manifest.contains(bad), "sm-core must not depend on {bad}");
    }
}
