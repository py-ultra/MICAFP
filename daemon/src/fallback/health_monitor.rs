//! HealthMonitor — Five-Signal Blocking Detection Engine
//!
//! Runs as a background task and continuously measures tunnel health
//! using five independent signals. Emits BlockSignal events to the
//! FallbackEngine when blocking is detected.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, mpsc};
use tokio::time::interval;
use tracing::{debug, info, warn};

use super::{ActiveTunnel, FallbackConfig, TunnelStats};
use super::block_detector::{BlockSignal, BlockSignalType};

/// Sliding window throughput tracker.
struct ThroughputWindow {
    samples: std::collections::VecDeque<(Instant, u64)>,
    window: Duration,
}

impl ThroughputWindow {
    fn new(window_secs: u64) -> Self {
        Self {
            samples: std::collections::VecDeque::with_capacity(64),
            window: Duration::from_secs(window_secs),
        }
    }

    fn push(&mut self, bytes: u64) {
        let now = Instant::now();
        self.samples.push_back((now, bytes));
        // Evict old samples
        while self.samples.front()
            .map(|(t, _)| now.duration_since(*t) > self.window)
            .unwrap_or(false)
        {
            self.samples.pop_front();
        }
    }

    /// Bytes per second over the tracking window.
    fn bps(&self) -> u64 {
        if self.samples.len() < 2 { return 0; }
        let first = self.samples.front().unwrap();
        let last  = self.samples.back().unwrap();
        let elapsed = last.0.duration_since(first.0).as_secs_f64();
        if elapsed < 0.001 { return 0; }
        let total_bytes: u64 = self.samples.iter().map(|(_, b)| b).sum();
        (total_bytes as f64 / elapsed) as u64
    }
}

/// RST storm tracker.
struct RstTracker {
    timestamps: std::collections::VecDeque<Instant>,
    window: Duration,
    threshold: u32,
}

impl RstTracker {
    fn new(window_secs: u64, threshold: u32) -> Self {
        Self {
            timestamps: std::collections::VecDeque::with_capacity(32),
            window: Duration::from_secs(window_secs),
            threshold,
        }
    }

    fn record_rst(&mut self) {
        let now = Instant::now();
        self.timestamps.push_back(now);
        while self.timestamps.front()
            .map(|t| now.duration_since(*t) > self.window)
            .unwrap_or(false)
        {
            self.timestamps.pop_front();
        }
    }

    fn is_storm(&self) -> bool {
        self.timestamps.len() as u32 >= self.threshold
    }

    fn count(&self) -> usize { self.timestamps.len() }
}

/// TLS failure rate tracker (failures per minute).
struct TlsFailureTracker {
    failures: std::collections::VecDeque<Instant>,
    window: Duration,
}

impl TlsFailureTracker {
    fn new() -> Self {
        Self {
            failures: std::collections::VecDeque::with_capacity(32),
            window: Duration::from_secs(60),
        }
    }

    fn record_failure(&mut self) {
        let now = Instant::now();
        self.failures.push_back(now);
        while self.failures.front()
            .map(|t| now.duration_since(*t) > self.window)
            .unwrap_or(false)
        {
            self.failures.pop_front();
        }
    }

    fn failures_per_minute(&self) -> f32 {
        self.failures.len() as f32
    }
}

pub struct HealthMonitor {
    config: FallbackConfig,
}

impl HealthMonitor {
    pub fn new(config: FallbackConfig) -> Self {
        Self { config }
    }

    /// Main monitoring loop. Runs until tunnel is closed or fallback triggered.
    pub async fn run(
        &self,
        tunnel: Arc<RwLock<Option<Box<dyn ActiveTunnel>>>>,
        signal_tx: mpsc::Sender<BlockSignal>,
        config: FallbackConfig,
    ) {
        let mut throughput = ThroughputWindow::new(config.throughput_collapse_window_secs);
        let mut rst_tracker = RstTracker::new(config.rst_window_secs, config.rst_storm_threshold);
        let mut tls_tracker = TlsFailureTracker::new();
        let mut last_stats = TunnelStats::default();
        let mut last_keepalive_ok = Instant::now();

        // Signal 1+2: Throughput + Keepalive — check every 2 seconds
        let mut throughput_tick = interval(Duration::from_secs(2));
        // Signal 2: Keepalive — check every keepalive_interval_secs
        let mut keepalive_tick = interval(Duration::from_secs(config.keepalive_interval_secs));

        loop {
            tokio::select! {
                _ = throughput_tick.tick() => {
                    let guard = tunnel.read().await;
                    if let Some(t) = guard.as_ref() {
                        let stats = t.stats();
                        let new_bytes = stats.bytes_received.saturating_sub(last_stats.bytes_received);
                        throughput.push(new_bytes);
                        last_stats = stats;

                        let bps = throughput.bps();
                        debug!("HealthMonitor: throughput={} bps", bps);

                        // Signal 1: Throughput collapse
                        if bps < config.throughput_collapse_threshold_bps && new_bytes == 0 {
                            warn!("HealthMonitor: throughput collapse detected ({} bps)", bps);
                            let _ = signal_tx.send(BlockSignal::new(
                                BlockSignalType::ThroughputCollapse,
                                format!("throughput={}bps threshold={}bps",
                                    bps, config.throughput_collapse_threshold_bps)
                            )).await;
                        }

                        // Signal 4: TLS failures via tunnel error state
                        if !t.is_alive().await {
                            tls_tracker.record_failure();
                            let rate = tls_tracker.failures_per_minute();
                            if rate >= config.tls_failure_rate_threshold {
                                warn!("HealthMonitor: TLS failure spike ({:.1}/min)", rate);
                                let _ = signal_tx.send(BlockSignal::new(
                                    BlockSignalType::TlsFailureSpike,
                                    format!("failures_per_min={:.1} threshold={:.1}",
                                        rate, config.tls_failure_rate_threshold)
                                )).await;
                            }
                        }
                    }
                }

                _ = keepalive_tick.tick() => {
                    let guard = tunnel.read().await;
                    if let Some(t) = guard.as_ref() {
                        match tokio::time::timeout(
                            Duration::from_secs(config.keepalive_timeout_secs),
                            t.keepalive_ping()
                        ).await {
                            Ok(Ok(rtt)) => {
                                last_keepalive_ok = Instant::now();
                                debug!("HealthMonitor: keepalive OK rtt={:?}", rtt);
                            }
                            Ok(Err(e)) => {
                                warn!("HealthMonitor: keepalive error: {}", e);
                                rst_tracker.record_rst();
                                if rst_tracker.is_storm() {
                                    let _ = signal_tx.send(BlockSignal::new(
                                        BlockSignalType::RstStorm,
                                        format!("rst_count={} window={}s",
                                            rst_tracker.count(), config.rst_window_secs)
                                    )).await;
                                }
                            }
                            Err(_) => {
                                // Signal 2: Keepalive timeout
                                let elapsed = last_keepalive_ok.elapsed();
                                warn!("HealthMonitor: keepalive timeout after {:?}", elapsed);
                                let _ = signal_tx.send(BlockSignal::new(
                                    BlockSignalType::KeepaliveTimeout,
                                    format!("timeout={}s since_last_ok={:.0}s",
                                        config.keepalive_timeout_secs, elapsed.as_secs_f32())
                                )).await;
                            }
                        }
                    }
                }
            }
        }
    }
}
