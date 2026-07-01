// Failover Engine — executes rapid transport switching on health degradation.

use std::time::{Duration, Instant};
use tracing::{info, warn};

pub struct FailoverEngine {
    pub timeout: Duration,
    pub attempts: u32,
    pub last_failover: Option<Instant>,
    pub cooldown: Duration,
}

impl FailoverEngine {
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            attempts: 0,
            last_failover: None,
            cooldown: Duration::from_secs(5),
        }
    }

    /// Returns true if a failover is allowed (respects cooldown).
    pub fn can_failover(&self) -> bool {
        match self.last_failover {
            None => true,
            Some(t) => t.elapsed() > self.cooldown,
        }
    }

    /// Record a failover event and update cooldown.
    pub fn record_failover(&mut self, from: &str, to: &str) {
        self.attempts += 1;
        self.last_failover = Some(Instant::now());
        info!(from, to, attempt = self.attempts, "Transport failover executed");
    }
}

// ── TASK-02: Publish failover events to Flutter layer ────────────────────────

impl FailoverEngine {
    /// Record a failover with FRB event publication. Replaces the original
    /// `record_failover` for all callers that have latency data available.
    pub fn record_failover_with_latency(
        &mut self,
        from: &str,
        to: &str,
        latency_ms: u64,
    ) {
        self.attempts += 1;
        self.last_failover = Some(Instant::now());
        tracing::info!(
            from,
            to,
            attempt = self.attempts,
            latency_ms,
            "Transport failover executed"
        );

        // Push to Flutter — zero user interaction required.
        crate::orchestrator::publish(
            crate::frb_api::ShieldEvent::TransportChanged {
                from: from.to_owned(),
                to: to.to_owned(),
                reason: format!(
                    "Failover attempt #{} (AI-triggered)",
                    self.attempts
                ),
                failover_latency_ms: latency_ms,
            },
        );
    }
}
