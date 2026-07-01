//! Adaptive Channel Learning — Feature 1 (MICAFP v8.0)
//!
//! Tracks per-channel success rates and sorts channels by performance
//! so the best-performing channels are always tried first.

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Debug, Clone, Serialize, Deserialize, Zeroize)]
pub struct ChannelStats {
    pub channel_id:            u8,
    pub success_count:         u64,
    pub fail_count:            u64,
    pub avg_latency_ms:        f32,
    pub last_success:          u64,  // NTP timestamp
    pub consecutive_failures:  u8,
}

impl Default for ChannelStats {
    fn default() -> Self {
        Self {
            channel_id: 0,
            success_count: 0,
            fail_count: 0,
            avg_latency_ms: 0.0,
            last_success: 0,
            consecutive_failures: 0,
        }
    }
}

impl ChannelStats {
    pub fn new(channel_id: u8) -> Self {
        Self { channel_id, ..Default::default() }
    }

    pub fn success_rate(&self) -> f32 {
        let total = self.success_count + self.fail_count;
        if total == 0 { return 0.5; } // unknown → neutral
        self.success_count as f32 / total as f32
    }

    pub fn detection_risk(&self) -> f32 {
        let total = (self.success_count + self.fail_count).max(1);
        self.fail_count as f32 / total as f32
    }

    pub fn record_success(&mut self, latency_ms: u32, ntp_now: u64) {
        self.success_count += 1;
        self.consecutive_failures = 0;
        self.last_success = ntp_now;
        // Exponential moving average (α = 0.2)
        self.avg_latency_ms = self.avg_latency_ms * 0.8
            + latency_ms as f32 * 0.2;
    }

    pub fn record_failure(&mut self) {
        self.fail_count += 1;
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
    }

    /// Channel is deprioritised (but never permanently disabled) after
    /// 10 consecutive failures.
    pub fn is_deprioritized(&self) -> bool {
        self.consecutive_failures >= 10
    }

    /// Sort channel IDs by success rate descending.
    /// Deprioritised channels are moved to the end.
    pub fn sorted_by_performance(all: &[ChannelStats]) -> Vec<u8> {
        let mut indexed: Vec<(u8, f32, bool)> = all.iter()
            .map(|s| (s.channel_id, s.success_rate(), s.is_deprioritized()))
            .collect();
        indexed.sort_by(|a, b| {
            // Deprioritised always last
            match (a.2, b.2) {
                (true,  false) => std::cmp::Ordering::Greater,
                (false, true)  => std::cmp::Ordering::Less,
                _ => b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal),
            }
        });
        indexed.iter().map(|(id, _, _)| *id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_rate_empty() {
        let s = ChannelStats::new(1);
        assert!((s.success_rate() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_record_success_updates_rate() {
        let mut s = ChannelStats::new(1);
        s.record_success(100, 0);
        assert!((s.success_rate() - 1.0).abs() < 0.001);
        assert_eq!(s.consecutive_failures, 0);
    }

    #[test]
    fn test_deprioritized_after_10_failures() {
        let mut s = ChannelStats::new(1);
        for _ in 0..10 { s.record_failure(); }
        assert!(s.is_deprioritized());
    }

    #[test]
    fn test_deprioritized_reset_on_success() {
        let mut s = ChannelStats::new(1);
        for _ in 0..10 { s.record_failure(); }
        s.record_success(50, 0);
        assert!(!s.is_deprioritized());
    }

    #[test]
    fn test_sorted_by_performance() {
        let stats = vec![
            { let mut s = ChannelStats::new(1); s.record_failure(); s },
            { let mut s = ChannelStats::new(2); s.record_success(50, 0); s },
            ChannelStats::new(3),
        ];
        let order = ChannelStats::sorted_by_performance(&stats);
        assert_eq!(order[0], 2); // best success rate first
    }
}
