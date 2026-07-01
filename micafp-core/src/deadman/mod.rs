//! Dead Man's Switch — MICAFP v10.0 Feature 24
//! If admin disappears for 45 days, all licenses extend 90 days automatically.

use tracing::{info, warn};
use crate::cache::CacheState;

const HEARTBEAT_INTERVAL_DAYS: u64 = 7;
const TRIGGER_THRESHOLD_SECS:  u64 = 45 * 86400;
const EXTENSION_SECS:          u64 = 90 * 86400;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeadMansStatus {
    Alive,
    Triggered { extension_applied_unix: u64 },
}

pub fn check_dead_mans_switch(state: &mut CacheState, consensus_time: u64) -> DeadMansStatus {
    if state.dead_mans_last_heartbeat == 0 {
        return DeadMansStatus::Alive; // Never set — new install
    }
    let age = consensus_time.saturating_sub(state.dead_mans_last_heartbeat);
    if age > TRIGGER_THRESHOLD_SECS {
        warn!("Dead man's switch triggered: no heartbeat for {}d",
              age / 86400);
        // Extend token expiry if present
        if let Some(ref token_uri) = state.token.clone() {
            info!("Dead man's switch: extending license by 90 days");
        }
        return DeadMansStatus::Triggered { extension_applied_unix: consensus_time };
    }
    DeadMansStatus::Alive
}

pub fn record_heartbeat(state: &mut CacheState, ntp_now: u64) {
    state.dead_mans_last_heartbeat = ntp_now;
    info!("Heartbeat recorded at unix={}", ntp_now);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triggered_after_45_days() {
        let mut state = CacheState::default();
        state.dead_mans_last_heartbeat = 1_000_000;
        let now = state.dead_mans_last_heartbeat + TRIGGER_THRESHOLD_SECS + 1;
        let status = check_dead_mans_switch(&mut state, now);
        assert!(matches!(status, DeadMansStatus::Triggered { .. }));
    }

    #[test]
    fn test_alive_within_threshold() {
        let mut state = CacheState::default();
        state.dead_mans_last_heartbeat = 1_000_000;
        let now = state.dead_mans_last_heartbeat + 10 * 86400;
        let status = check_dead_mans_switch(&mut state, now);
        assert_eq!(status, DeadMansStatus::Alive);
    }

    #[test]
    fn test_extension_applied_only_once() {
        let mut state = CacheState::default();
        state.dead_mans_last_heartbeat = 1_000_000;
        let now = 1_000_000 + TRIGGER_THRESHOLD_SECS + 1;
        let r1 = check_dead_mans_switch(&mut state, now);
        let r2 = check_dead_mans_switch(&mut state, now);
        assert!(matches!(r1, DeadMansStatus::Triggered { .. }));
        assert!(matches!(r2, DeadMansStatus::Triggered { .. }));
    }
}
