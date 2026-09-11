pub mod commands;
pub mod logging;
pub mod poller;
pub mod state;

use std::sync::Arc;

use tauri::Manager;

/// Runs the desktop app: loads the config, wires the infra-backed
/// `AppState`, registers every invoke command (Task 18) and starts the
/// background status/stats poller (Task 19).
pub fn run() {
    let paths = state::resolve_paths();
    logging::init_logging(&paths);
    let (cfg, notice) = state::load_config(&paths);
    let app_state = Arc::new(state::AppState::new(paths, cfg, notice));

    tauri::Builder::default()
        .manage(app_state.clone())
        .setup(|app| {
            poller::spawn(app.handle().clone(), app.state::<Arc<state::AppState>>().inner().clone());
            Ok(())
        })
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