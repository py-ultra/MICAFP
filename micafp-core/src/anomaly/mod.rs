//! Behavioral Anomaly Detection — MICAFP v10.0 Feature 21
//! Detects license sharing via usage pattern analysis. Never blocks — only logs.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageBaseline {
    pub avg_sessions_per_day: f32,
    pub avg_session_duration_min: f32,
    pub baseline_complete: bool,
    session_count_today: u32,
    total_days: u32,
}

impl Default for UsageBaseline {
    fn default() -> Self {
        Self {
            avg_sessions_per_day: 0.0,
            avg_session_duration_min: 0.0,
            baseline_complete: false,
            session_count_today: 0,
            total_days: 0,
        }
    }
}

impl UsageBaseline {
    pub fn record_session_start(&mut self) {
        self.session_count_today += 1;
    }

    pub fn record_day_end(&mut self) {
        let alpha = 0.1f32;
        self.avg_sessions_per_day = self.avg_sessions_per_day * (1.0 - alpha)
            + self.session_count_today as f32 * alpha;
        self.session_count_today = 0;
        self.total_days += 1;
        if self.total_days >= 7 { self.baseline_complete = true; }
    }

    pub fn check_anomaly(&self) -> Option<AnomalyEvent> {
        if !self.baseline_complete { return None; }
        if self.avg_sessions_per_day > 0.0
            && self.session_count_today as f32 > self.avg_sessions_per_day * 10.0
        {
            return Some(AnomalyEvent::ExcessiveSessions {
                today: self.session_count_today,
                baseline: self.avg_sessions_per_day,
            });
        }
        None
    }
}

#[derive(Debug, Clone)]
pub enum AnomalyEvent {
    ExcessiveSessions { today: u32, baseline: f32 },
    SessionTooLong    { duration_min: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_no_anomaly_before_baseline() {
        let b = UsageBaseline::default();
        assert!(b.check_anomaly().is_none());
    }
}
