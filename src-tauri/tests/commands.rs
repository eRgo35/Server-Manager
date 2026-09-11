//! Task 18: `src-tauri` command tests over tauri's MockRuntime IPC.
//!
//! `mock_context(noop_assets())` avoids the real `tauri.conf.json`/frontend
//! assets; `AppState` is managed by hand on the mock app.

use serde_json::json;
use tauri::test::{mock_builder, mock_context, noop_assets};
use tauri::Manager;

use server_manager::commands;
use server_manager::state::{AppState, Paths};
use sm_core::{Config, Machine, MachineId, SecretMode};

type MockApp = tauri::App<tauri::test::MockRuntime>;
type MockWindow = tauri::WebviewWindow<tauri::test::MockRuntime>;

fn paths(dir: &std::path::Path) -> Paths {
    Paths {
        dir: dir.to_owned(),
        config: dir.join("config.toml"),
        known_hosts: dir.join("known_hosts"),
        log: dir.join("latest.log"),
    }
}

fn app(dir: &std::path::Path) -> MockApp {
    let state = std::sync::Arc::new(AppState::new(paths(dir), Config::default(), None));
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::set_active,
            commands::get_machine,
            commands::upsert_machine,
            commands::delete_machine,
            commands::save_settings,
            commands::wake,
            commands::refresh_now,
            commands::power,
            commands::trust_host,
            commands::provide_secret,
            commands::open_files,
            commands::sample_stats,
        ])
        .build(mock_context(noop_assets()))
        .expect("mock app builds");
    app.manage(state);
    app
}

fn window(app: &MockApp) -> MockWindow {
    tauri::WebviewWindowBuilder::new(app, "main", Default::default())
        .build()
        .expect("mock webview window")
}

/// Invokes `cmd` over the mock IPC and returns the payload as JSON.
fn ipc(
    webview: &MockWindow,
    cmd: &str,
    body: serde_json::Value,
) -> Result<serde_json::Value, serde_json::Value> {
    tauri::test::get_ipc_response(
        webview,
        tauri::webview::InvokeRequest {
            cmd: cmd.to_owned(),
            callback: tauri::ipc::CallbackFn(0),
            error: tauri::ipc::CallbackFn(1),
            // The local app origin (http://tauri.localhost is Windows/Android only).
            url: "tauri://localhost".parse().unwrap(),
            body: tauri::ipc::InvokeBody::Json(body),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_owned(),
        },
    )
    .map(|b| {
        b.deserialize::<serde_json::Value>()
            .expect("payload is JSON")
    })
}

fn test_machine() -> Machine {
    Machine {
        id: MachineId::from("nas"),
        name: "NAS".into(),
        mac: "AA:BB:CC:DD:EE:FF".into(),
        broadcast_addr: None,
        os_host: "192.168.1.10".into(),
        ssh_port: 22,
        ssh_user: "mike".into(),
        key_path: None,
        secret_mode: None,
        shutdown_cmd: None,
        reboot_cmd: None,
        stats_cmd: None,
        share_path: None,
        ssh_password: Some("hunter2".into()),
        key_passphrase: None,
        sudo_password: None,
    }
}

#[test]
fn get_state_returns_defaults_on_fresh_config() {
    let dir = tempfile::tempdir().unwrap();
    let app = app(dir.path());
    let w = window(&app);
    let res = ipc(&w, "get_state", json!({})).expect("get_state ok");
    let snap: server_manager::commands::AppSnapshot = serde_json::from_value(res).unwrap();
    assert_eq!(snap.settings.poll_base_secs, 5);
    assert!(matches!(
        snap.settings.default_secret_mode,
        SecretMode::Keyring
    ));
    assert!(snap.machines.is_empty());
    assert_eq!(snap.active, None);
    assert_eq!(snap.notice, None);
}

#[test]
fn upsert_then_get_machine_blanks_secrets() {
    let dir = tempfile::tempdir().unwrap();
    let app = app(dir.path());
    let w = window(&app);

    ipc(
        &w,
        "upsert_machine",
        json!({ "machine": serde_json::to_value(test_machine()).unwrap() }),
    )
    .expect("upsert_machine ok");

    let res = ipc(&w, "get_machine", json!({ "id": "nas" })).expect("get_machine ok");
    let got: Machine = serde_json::from_value(res).unwrap();
    assert_eq!(got.id, MachineId::from("nas"));
    assert_eq!(got.name, "NAS");
    assert_eq!(got.os_host, "192.168.1.10");
    assert_eq!(got.ssh_password, None, "secrets must be blanked");

    // The machine was persisted to config.toml as well.
    let text = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(text.contains("nas"));
}
