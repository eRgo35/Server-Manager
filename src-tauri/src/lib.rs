pub mod commands;
pub mod logging;
pub mod state;

use std::sync::Arc;

/// Runs the desktop app: loads the config, wires the infra-backed
/// `AppState`, and registers every invoke command (Task 18). Poller wiring
/// (poll loop, backoff) lands in Task 19.
pub fn run() {
    let paths = state::resolve_paths();
    logging::init_logging(&paths);
    let (cfg, notice) = state::load_config(&paths);
    let app_state = Arc::new(state::AppState::new(paths, cfg, notice));

    tauri::Builder::default()
        .manage(app_state)
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}