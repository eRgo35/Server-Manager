pub mod logging;
pub mod state;

/// Runs the desktop app. Wiring of the Tauri builder and commands lands in
/// Tasks 18/19; nothing may reach this in M1 tests, hence the explicit abort.
pub fn run() {
    unimplemented!("tauri runtime wiring lands in Task 18");
}