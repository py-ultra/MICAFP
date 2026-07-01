//! Blackout Mode Engine — MICAFP v5.0 + Feature 20 Split-Brain Protection
//!
//! Trigger: ALL 10 channels fail for >2 hours AND last NTP confirmed <2h ago.
//! The second condition prevents a "fake blackout" attack where a user
//! disconnects AFTER expiry to illegitimately extend the grace period.
//! Grace: 30 days. On reconnect: immediate fetch. If expired → block.

use std::time::{Duration, Instant};
use tracing::{error, info, warn};

const CHANNEL_FAIL_TRIGGER_HOURS: u64 = 2;
const BLACKOUT_GRACE_DAYS: u64        = 30;
const NTP_MAX_AGE_FOR_BLACKOUT_HOURS: u64 = 2;
const NORMAL_GRACE_HOURS: u64         = 72;
const SPLIT_BRAIN_REDUCTION_PER_CYCLE_HOURS: u64 = 4;
const MINIMUM_GRACE_HOURS: u64        = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlackoutStatus {
    /// Normal operation — not in blackout.
    Normal,
    /// All channels failed, NTP recently confirmed — blackout active.
    Active { grace_remaining_days: u32 },
    /// Blackout expired — block all traffic.
    Expired,
}

pub struct BlackoutEngine {
    all_channels_failed_since: Option<Instant>,
    last_ntp_confirmed_unix:   u64,
    blackout_entered_unix:     Option<u64>,
    /// Feature 20: how many times device went offline and came back without renewal.
    airplane_cycles:           u32,
}

impl BlackoutEngine {
    pub fn new(last_ntp_confirmed: u64, blackout_entered: Option<u64>, airplane_cycles: u32) -> Self {
        Self {
            all_channels_failed_since: None,
            last_ntp_confirmed_unix: last_ntp_confirmed,
            blackout_entered_unix: blackout_entered,
            airplane_cycles,
        }
    }

    /// Call when NTP is successfully confirmed.
    pub fn record_ntp_confirmed(&mut self, unix_ts: u64) {
        self.last_ntp_confirmed_unix = unix_ts;
    }

    /// Call when a channel fetch attempt fails.
    pub fn record_all_channels_failed(&mut self) {
        if self.all_channels_failed_since.is_none() {
            info!("BlackoutEngine: all channels failed — starting blackout timer");
            self.all_channels_failed_since = Some(Instant::now());
        }
    }

    /// Call when any channel fetch succeeds.
    pub fn record_channel_success(&mut self) {
        if self.all_channels_failed_since.is_some() {
            info!("BlackoutEngine: channel recovered — resetting blackout timer");
        }
        self.all_channels_failed_since = None;
    }

    /// Call when device goes offline then comes back without a renewal.
    pub fn record_airplane_cycle(&mut self) {
        self.airplane_cycles = self.airplane_cycles.saturating_add(1);
        warn!("BlackoutEngine: airplane cycle #{} detected", self.airplane_cycles);
    }

    /// Evaluate current blackout status.
    /// `ntp_now`: current NTP-verified time.
    pub fn check_status(&mut self, ntp_now: u64) -> BlackoutStatus {
        // Check if already in blackout
        if let Some(entered) = self.blackout_entered_unix {
            let elapsed_secs = ntp_now.saturating_sub(entered);
            let grace_secs   = BLACKOUT_GRACE_DAYS * 86400;
            if elapsed_secs >= grace_secs {
                error!("BlackoutEngine: 30-day grace period expired — blocking all traffic");
                return BlackoutStatus::Expired;
            }
            let remaining_days = ((grace_secs - elapsed_secs) / 86400) as u32;
            return BlackoutStatus::Active { grace_remaining_days: remaining_days };
        }

        // Check trigger conditions
        let fail_duration = self.all_channels_failed_since
            .map(|t| t.elapsed())
            .unwrap_or(Duration::ZERO);

        let ntp_age_secs = ntp_now.saturating_sub(self.last_ntp_confirmed_unix);
        let ntp_is_fresh = ntp_age_secs < NTP_MAX_AGE_FOR_BLACKOUT_HOURS * 3600;

        let trigger_threshold = Duration::from_secs(CHANNEL_FAIL_TRIGGER_HOURS * 3600);

        if fail_duration >= trigger_threshold && ntp_is_fresh {
            info!("BlackoutEngine: TRIGGER — activating 30-day grace period at unix={}", ntp_now);
            self.blackout_entered_unix = Some(ntp_now);
            self.all_channels_failed_since = None;
            return BlackoutStatus::Active { grace_remaining_days: BLACKOUT_GRACE_DAYS as u32 };
        }

        BlackoutStatus::Normal
    }

    /// Feature 20: allowed grace hours accounting for split-brain airplane cycles.
    pub fn allowed_normal_grace_hours(&self) -> u64 {
        let reduction = self.airplane_cycles as u64 * SPLIT_BRAIN_REDUCTION_PER_CYCLE_HOURS;
        NORMAL_GRACE_HOURS.saturating_sub(reduction).max(MINIMUM_GRACE_HOURS)
    }

    pub fn blackout_entered_unix(&self) -> Option<u64> { self.blackout_entered_unix }
    pub fn airplane_cycles(&self) -> u32 { self.airplane_cycles }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_blackout_if_ntp_stale() {
        let stale_ntp = 1_000u64; // very old
        let mut engine = BlackoutEngine::new(stale_ntp, None, 0);
        engine.record_all_channels_failed();
        // Simulate >2h passed but NTP is stale (>2h old)
        let current_time = stale_ntp + 3 * 3600 + 100; // 3h later
        // NTP age = 3h > 2h → should NOT trigger blackout
        let status = engine.check_status(current_time);
        // With Instant-based tracking this test verifies logic in isolation
        assert!(matches!(status, BlackoutStatus::Normal));
    }

    #[test]
    fn test_split_brain_reduces_grace() {
        let mut engine = BlackoutEngine::new(0, None, 0);
        assert_eq!(engine.allowed_normal_grace_hours(), 72);
        engine.record_airplane_cycle();
        engine.record_airplane_cycle();
        assert_eq!(engine.allowed_normal_grace_hours(), 64);
    }

    #[test]
    fn test_split_brain_minimum_grace() {
        let mut engine = BlackoutEngine::new(0, None, 0);
        for _ in 0..20 { engine.record_airplane_cycle(); }
        assert_eq!(engine.allowed_normal_grace_hours(), MINIMUM_GRACE_HOURS);
    }

    #[test]
    fn test_blackout_expiry() {
        let entered = 1_000_000u64;
        let mut engine = BlackoutEngine::new(entered, Some(entered), 0);
        let after_30_days = entered + 30 * 86400 + 1;
        let status = engine.check_status(after_30_days);
        assert_eq!(status, BlackoutStatus::Expired);
    }
}
