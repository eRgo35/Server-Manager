//! Background status/stats poller for the active machine (Task 19, spec §6.2).
//!
//! One tokio task polls the active machine's TCP reachability, emits
//! `status://update` each tick and, on every 6th consecutive online tick,
//! `stats://update`. Offline polls back off ×2 per consecutive failure
//! (cap 300 s); `set_active` / `refresh_now` wake the loop immediately via
//! `AppState::notify` and reset the backoff.

use std::sync::Arc;
use std::time::Duration;

use serde_json::json;
use sm_core::{BackoffState, MachineId};
use sm_services::{StatsProbe, StatusProbe};
use tauri::{AppHandle, Emitter};

use crate::commands::err_string;
use crate::state::AppState;

/// One-shot probe timeout, same as `commands::refresh_now`.
const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

/// Sample stats on every 6th tick while online (spec: stats are cheap to
/// skip; status is what drives the UI colour).
const STATS_EVERY: u64 = 6;

/// What one poll tick should do and how long to wait before the next one.
#[derive(Debug, PartialEq)]
pub struct TickPlan {
    pub emit_status: bool,
    pub sample_stats: bool,
    pub sleep_secs: u64,
}

/// Pure tick decision: online resets the backoff and sleeps at the base
/// rate (sampling stats on every [`STATS_EVERY`]th tick); offline records a
/// failure so the sleep follows the backoff ramp.
fn plan_tick(online: bool, tick: u64, backoff: &mut BackoffState) -> TickPlan {
    if online {
        backoff.on_success();
    } else {
        backoff.on_failure();
    }
    TickPlan {
        emit_status: true,
        sample_stats: online && tick % STATS_EVERY == 0,
        sleep_secs: backoff.current_delay_secs(),
    }
}

/// Starts the poll loop on the Tauri async runtime.
pub fn spawn(app: AppHandle, state: Arc<AppState>) {
    tauri::async_runtime::spawn(poll_loop(app, state));
}

async fn poll_loop(app: AppHandle, state: Arc<AppState>) {
    let mut tick: u64 = 0;
    loop {
        // Resolve the active machine; if none, idle at the base interval
        // (still wakeable by set_active).
        let active: Option<(MachineId, sm_core::Machine)> = {
            let id = state.active.read().unwrap().clone();
            id.and_then(|id| {
                let machine = state.cfg.read().unwrap().machine(&id).cloned();
                machine.map(|m| (id, m))
            })
        };
        let Some((id, machine)) = active else {
            let base = base_secs(&state);
            wait_or_notify(&state, base).await;
            continue;
        };

        let online = state
            .probe
            .is_up(&machine.os_host, machine.ssh_port, PROBE_TIMEOUT)
            .await;
        let plan = plan_tick(online, tick, &mut state.backoff.write().unwrap());

        let _ = app.emit(
            "status://update",
            json!({
                "id": id.0,
                "status": if online { "online" } else { "offline" },
                "next_poll_secs": plan.sleep_secs,
            }),
        );

        if plan.sample_stats {
            let payload = match state.stats.sample(&machine).await {
                Ok(stats) => json!({ "id": id.0, "stats": stats }),
                Err(e) => json!({ "id": id.0, "error": err_string(e) }),
            };
            let _ = app.emit("stats://update", payload);
        }

        tick = tick.wrapping_add(1);
        wait_or_notify(&state, plan.sleep_secs).await;
    }
}

fn base_secs(state: &AppState) -> u64 {
    state.cfg.read().unwrap().settings.poll_base_secs.max(1)
}

/// Sleeps `secs`, or wakes early when a command signals a re-poll.
async fn wait_or_notify(state: &AppState, secs: u64) {
    tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(secs)) => {}
        _ = state.notify.notified() => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_sleep_follows_backoff_growth() {
        let mut b = BackoffState::new(5);
        let p = plan_tick(false, 0, &mut b);
        assert_eq!(
            p,
            TickPlan {
                emit_status: true,
                sample_stats: false,
                sleep_secs: 10
            }
        );
        assert_eq!(plan_tick(false, 1, &mut b).sleep_secs, 20);
        assert_eq!(plan_tick(false, 2, &mut b).sleep_secs, 40);
    }

    #[test]
    fn online_sleeps_at_base_rate() {
        let mut b = BackoffState::new(5);
        for t in 0..6u64 {
            let p = plan_tick(true, t, &mut b);
            assert_eq!(p.sleep_secs, 5);
            assert!(p.emit_status);
            assert_eq!(p.sample_stats, t % 6 == 0);
        }
    }

    #[test]
    fn online_every_sixth_tick_samples_stats() {
        let mut b = BackoffState::new(5);
        assert!(plan_tick(true, 0, &mut b).sample_stats);
        assert!(!plan_tick(true, 5, &mut b).sample_stats);
        assert!(plan_tick(true, 6, &mut b).sample_stats);
    }

    #[test]
    fn success_resets_backoff() {
        let mut b = BackoffState::new(5);
        plan_tick(false, 0, &mut b);
        plan_tick(false, 1, &mut b);
        assert_eq!(plan_tick(true, 2, &mut b).sleep_secs, 5);
    }
}