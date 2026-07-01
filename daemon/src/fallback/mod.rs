//! Seamless Protocol Fallback Engine
//!
//! Detects mid-session protocol blocking and switches to the next best
//! protocol **without** visible disconnection to the user. The TUN
//! interface stays up throughout; only the underlying tunnel is replaced.
//!
//! ## Architecture
//!
//! ```text
//!  ┌─────────────────────────────────────────────────────────┐
//!  │                  User Application                        │
//!  │              (sees uninterrupted traffic)                │
//!  └──────────────────────┬──────────────────────────────────┘
//!                         │ TUN device (stays UP always)
//!  ┌──────────────────────▼──────────────────────────────────┐
//!  │              FallbackEngine (this module)                │
//!  │  ┌─────────────┐   ┌──────────────┐  ┌──────────────┐  │
//!  │  │HealthMonitor│   │ProtocolQueue │  │HotSwapper    │  │
//!  │  │detects block│──▶│ranked list   │──▶│atomic switch │  │
//!  │  └─────────────┘   └──────────────┘  └──────────────┘  │
//!  └──────────────────────────────────────────────────────────┘
//!                         │ Active tunnel (swapped on block)
//!  ┌──────────────────────▼──────────────────────────────────┐
//!  │     Current Protocol (Reality / ShadowTLS / Hysteria2…) │
//!  └──────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Blocking Detection Methods
//!
//! The HealthMonitor uses **five independent signals** to detect blocking:
//!
//! 1. **Throughput collapse** — bytes/sec drops below threshold for N seconds
//! 2. **Round-trip timeout** — keepalive pings get no response within deadline
//! 3. **TCP RST storm** — DPI sends RST packets to kill the connection
//! 4. **TLS error rate spike** — sudden increase in TLS handshake failures
//! 5. **Active probe detection** — eBPF sees DPI probing the server IP
//!
//! Any two signals triggering together → immediate fallback initiated.
//! Single signal → grace period (configurable) before fallback.
//!
//! ## Seamless Switchover Process
//!
//! 1. Parallel pre-connect: while current tunnel works, silently establish
//!    the next protocol in background (pre-warm).
//! 2. On block detection: atomic pointer swap — new tunnel becomes active.
//! 3. Drain in-flight packets from old tunnel into new tunnel.
//! 4. Old tunnel connection closed gracefully.
//! 5. Total user-visible interruption: < 200ms in best case.
//!
//! ## ISP-Aware Fallback Chains (from isp-profiles.json v8.0.0)
//!
//! | ISP           | Chain                                           |
//! |---------------|-------------------------------------------------|
//! | irancell      | Reality-random → ShadowTLS-v3 → AmneziaWG → CDN|
//! | pars_online   | Reality-random → AmneziaWG → ShadowTLS → CDN   |
//! | mci           | Reality-chrome → AmneziaWG → ShadowTLS → CDN   |
//! | shatel        | Reality-firefox → AmneziaWG → NaiveProxy → CDN |
//! | rightel       | Hysteria2 → Reality → NaiveProxy → CDN          |
//! | asiatech      | Hysteria2 → Reality → NaiveProxy → CDN          |
//! | mokhaberat    | Reality → Psiphon → ArvanCloud-CDN               |
//! | NAIN (any)    | ArvanCloud-CDN (only reliable option)            |

pub mod block_detector;
pub mod health_monitor;
pub mod hot_swap;
pub mod prewarmer;
pub mod state_machine;

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock, watch};
use tokio::time::{interval, sleep, timeout};
use tracing::{error, info, warn, debug};

use crate::isp_detector::{DetectedIsp, protocol_selector::{self, Protocol, ProtocolSelection}};
use block_detector::{BlockSignal, BlockSignalType};
use health_monitor::HealthMonitor;
use hot_swap::HotSwapManager;
use prewarmer::ProtocolPrewarmer;
use state_machine::{FallbackState, FallbackStateMachine};

/// Fallback engine configuration.
#[derive(Debug, Clone)]
pub struct FallbackConfig {
    /// Throughput below this (bytes/sec) triggers a signal.
    pub throughput_collapse_threshold_bps: u64,
    /// Duration of low throughput before it counts as a signal (seconds).
    pub throughput_collapse_window_secs: u64,
    /// Keepalive interval (seconds).
    pub keepalive_interval_secs: u64,
    /// Keepalive timeout before considered dead (seconds).
    pub keepalive_timeout_secs: u64,
    /// Number of RST packets in window to trigger a signal.
    pub rst_storm_threshold: u32,
    /// RST detection window (seconds).
    pub rst_window_secs: u64,
    /// TLS failure rate (failures per minute) to trigger a signal.
    pub tls_failure_rate_threshold: f32,
    /// How many signals must fire to trigger immediate fallback.
    pub simultaneous_signals_for_immediate_fallback: usize,
    /// Grace period after single signal before fallback (seconds).
    pub single_signal_grace_period_secs: u64,
    /// How far ahead to pre-warm the next protocol.
    pub prewarm_enabled: bool,
    /// Timeout for pre-warming a new tunnel (seconds).
    pub prewarm_timeout_secs: u64,
    /// Maximum protocols to try before giving up and entering error state.
    pub max_fallback_attempts: usize,
    /// Minimum time before re-adding a failed protocol to the chain (seconds).
    pub protocol_cooldown_secs: u64,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            throughput_collapse_threshold_bps: 1024,  // 1 KB/s
            throughput_collapse_window_secs: 5,
            keepalive_interval_secs: 20,
            keepalive_timeout_secs: 8,
            rst_storm_threshold: 3,
            rst_window_secs: 10,
            tls_failure_rate_threshold: 5.0,
            simultaneous_signals_for_immediate_fallback: 2,
            single_signal_grace_period_secs: 12,
            prewarm_enabled: true,
            prewarm_timeout_secs: 10,
            max_fallback_attempts: 8,
            protocol_cooldown_secs: 300,
        }
    }
}

impl FallbackConfig {
    /// ISP-specific tuning: more aggressive for higher FAVA versions.
    pub fn for_isp(isp_id: &str) -> Self {
        let base = Self::default();
        match isp_id {
            // FAVA v4.x: blocks fast — react aggressively
            "irancell" | "pars_online" => Self {
                throughput_collapse_window_secs: 3,
                keepalive_interval_secs: 15,
                keepalive_timeout_secs: 5,
                rst_storm_threshold: 2,
                simultaneous_signals_for_immediate_fallback: 1, // single signal = immediate
                single_signal_grace_period_secs: 5,
                protocol_cooldown_secs: 600,
                ..base
            },
            // FAVA v3.x: moderate reaction
            "mci" | "shatel" | "fanava" => Self {
                throughput_collapse_window_secs: 5,
                keepalive_timeout_secs: 6,
                single_signal_grace_period_secs: 10,
                protocol_cooldown_secs: 300,
                ..base
            },
            // FAVA v2.x / Light: conservative reaction
            "rightel" | "asiatech" | "afranet" | "mobinnet" | "pishgaman" => Self {
                throughput_collapse_window_secs: 8,
                keepalive_timeout_secs: 10,
                single_signal_grace_period_secs: 20,
                protocol_cooldown_secs: 180,
                ..base
            },
            _ => base,
        }
    }
}

/// Active tunnel abstraction — any protocol implementing this can be hot-swapped.
#[async_trait::async_trait]
pub trait ActiveTunnel: Send + Sync {
    /// Protocol identifier.
    fn protocol(&self) -> Protocol;
    /// Send bytes through this tunnel.
    async fn send(&self, data: &[u8]) -> Result<usize, TunnelError>;
    /// Receive bytes from this tunnel.
    async fn recv(&self, buf: &mut [u8]) -> Result<usize, TunnelError>;
    /// Send a keepalive ping and await pong. Returns RTT on success.
    async fn keepalive_ping(&self) -> Result<Duration, TunnelError>;
    /// Check if the tunnel is still healthy without sending data.
    async fn is_alive(&self) -> bool;
    /// Gracefully close the tunnel.
    async fn close(self: Box<Self>);
    /// Current bytes sent/received statistics.
    fn stats(&self) -> TunnelStats;
}

#[derive(Debug, Default, Clone)]
pub struct TunnelStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub rtt_ms: Option<u32>,
    pub established_at: Option<std::time::Instant>,
}

#[derive(Debug, thiserror::Error)]
pub enum TunnelError {
    #[error("Connection reset by peer (possible DPI RST)")]
    Reset,
    #[error("TLS handshake failed: {0}")]
    TlsError(String),
    #[error("Connection timed out")]
    Timeout,
    #[error("Protocol blocked: throughput collapsed")]
    ThroughputCollapse,
    #[error("IO error: {0}")]
    Io(String),
    #[error("Connection closed")]
    Closed,
}

/// The main fallback engine — manages the entire fallback lifecycle.
pub struct FallbackEngine {
    config: FallbackConfig,
    isp: DetectedIsp,
    /// Ordered protocol chain for this ISP.
    protocol_chain: Vec<ProtocolSelection>,
    /// Index into protocol_chain of the currently active protocol.
    current_index: usize,
    /// The active tunnel (atomically swapped on fallback).
    active_tunnel: Arc<RwLock<Option<Box<dyn ActiveTunnel>>>>,
    /// Fallback state machine.
    state: Arc<Mutex<FallbackStateMachine>>,
    /// Health monitor task handle.
    health_monitor: Arc<HealthMonitor>,
    /// Protocol pre-warmer.
    prewarmer: Arc<ProtocolPrewarmer>,
    /// Hot-swap manager for atomic tunnel replacement.
    hot_swap: Arc<HotSwapManager>,
    /// Channel to signal that a fallback is needed.
    fallback_tx: tokio::sync::mpsc::Sender<BlockSignal>,
}

impl FallbackEngine {
    /// Create a new FallbackEngine for the detected ISP.
    pub fn new(isp: DetectedIsp) -> Self {
        let config = FallbackConfig::for_isp(&isp.id);
        let protocol_chain = protocol_selector::select_protocols(&isp);
        let (fallback_tx, fallback_rx) = tokio::sync::mpsc::channel(32);

        info!(
            "FallbackEngine created for ISP '{}' with {} protocols in chain",
            isp.id,
            protocol_chain.len()
        );
        for (i, p) in protocol_chain.iter().enumerate() {
            info!("  Chain[{}]: {:?} — {}", i, p.protocol, p.reason);
        }

        Self {
            config: config.clone(),
            isp,
            protocol_chain,
            current_index: 0,
            active_tunnel: Arc::new(RwLock::new(None)),
            state: Arc::new(Mutex::new(FallbackStateMachine::new())),
            health_monitor: Arc::new(HealthMonitor::new(config.clone())),
            prewarmer: Arc::new(ProtocolPrewarmer::new()),
            hot_swap: Arc::new(HotSwapManager::new()),
            fallback_tx,
        }
    }

    /// Start the engine: connect with first protocol, begin monitoring.
    pub async fn start(&mut self) -> Result<(), FallbackError> {
        info!("FallbackEngine starting with protocol chain for ISP '{}'", self.isp.id);

        // Connect with the primary (highest priority) protocol
        self.connect_protocol(0).await?;

        // Start background tasks
        self.spawn_health_monitor();
        self.spawn_fallback_listener();
        if self.config.prewarm_enabled {
            self.spawn_prewarmer();
        }

        info!("FallbackEngine active — monitoring {} signals", 5);
        Ok(())
    }

    /// Connect with the protocol at chain index `idx`.
    async fn connect_protocol(&mut self, idx: usize) -> Result<(), FallbackError> {
        if idx >= self.protocol_chain.len() {
            error!("All {} protocols exhausted — no fallback available", self.protocol_chain.len());
            return Err(FallbackError::AllProtocolsExhausted);
        }

        let selection = &self.protocol_chain[idx];
        info!("Connecting protocol[{}]: {:?}", idx, selection.protocol);

        let tunnel = self.hot_swap
            .establish_tunnel(&selection.protocol, &selection.config_hints)
            .await
            .map_err(|e| {
                warn!("Protocol {:?} failed to connect: {}", selection.protocol, e);
                FallbackError::ConnectFailed(format!("{:?}: {}", selection.protocol, e))
            })?;

        // Atomic swap of active tunnel
        {
            let mut active = self.active_tunnel.write().await;
            if let Some(old) = active.take() {
                debug!("Draining and closing old tunnel");
                old.close().await;
            }
            *active = Some(tunnel);
        }

        self.current_index = idx;
        self.state.lock().await.transition_to(FallbackState::Connected);
        info!("Protocol {:?} connected and active", selection.protocol);
        Ok(())
    }

    /// Spawn the health monitor task.
    fn spawn_health_monitor(&self) {
        let monitor = self.health_monitor.clone();
        let tunnel = self.active_tunnel.clone();
        let tx = self.fallback_tx.clone();
        let cfg = self.config.clone();

        tokio::spawn(async move {
            monitor.run(tunnel, tx, cfg).await;
        });
    }

    /// Spawn the fallback listener — reacts to block signals.
    fn spawn_fallback_listener(&self) {
        let active_tunnel = self.active_tunnel.clone();
        let state = self.state.clone();
        let hot_swap = self.hot_swap.clone();
        let chain = self.protocol_chain.clone();
        let config = self.config.clone();
        let isp_id = self.isp.id.clone();
        let mut current_idx = self.current_index;
        let (tx, mut rx) = tokio::sync::mpsc::channel::<BlockSignal>(32);

        tokio::spawn(async move {
            let mut pending_signals: Vec<BlockSignal> = Vec::new();

            loop {
                tokio::select! {
                    Some(signal) = rx.recv() => {
                        warn!(
                            "Block signal received: {:?} for ISP '{}' protocol {:?}",
                            signal.signal_type, isp_id,
                            chain.get(current_idx).map(|p| &p.protocol)
                        );

                        pending_signals.push(signal.clone());
                        let simultaneous = pending_signals.iter()
                            .filter(|s| s.timestamp.elapsed() < Duration::from_secs(30))
                            .count();

                        let immediate = simultaneous >= config.simultaneous_signals_for_immediate_fallback
                            || signal.severity >= BlockSignalSeverity::Critical;

                        if immediate {
                            info!("Immediate fallback triggered ({} concurrent signals)", simultaneous);
                            pending_signals.clear();
                            Self::execute_fallback_static(
                                &active_tunnel, &state, &hot_swap, &chain,
                                &mut current_idx, &config
                            ).await;
                        } else {
                            // Grace period — wait and see if more signals come
                            let grace = config.single_signal_grace_period_secs;
                            debug!("Single signal — entering {}-second grace period", grace);
                            tokio::time::sleep(Duration::from_secs(grace)).await;

                            // Check if signal still persists
                            if pending_signals.len() >= 1 {
                                info!("Signal persisted through grace period — executing fallback");
                                pending_signals.clear();
                                Self::execute_fallback_static(
                                    &active_tunnel, &state, &hot_swap, &chain,
                                    &mut current_idx, &config
                                ).await;
                            }
                        }
                    }
                    else => break,
                }
            }
        });
    }

    /// Execute a fallback: find next working protocol, hot-swap the tunnel.
    async fn execute_fallback_static(
        active_tunnel: &Arc<RwLock<Option<Box<dyn ActiveTunnel>>>>,
        state: &Arc<Mutex<FallbackStateMachine>>,
        hot_swap: &Arc<HotSwapManager>,
        chain: &[ProtocolSelection],
        current_idx: &mut usize,
        config: &FallbackConfig,
    ) {
        state.lock().await.transition_to(FallbackState::FallingBack);

        let start_idx = *current_idx + 1;
        let end_idx = start_idx + config.max_fallback_attempts;

        for try_idx in start_idx..end_idx.min(chain.len()) {
            let selection = &chain[try_idx % chain.len()];
            info!(
                "Fallback attempt {}/{}: trying {:?}",
                try_idx - start_idx + 1,
                config.max_fallback_attempts,
                selection.protocol
            );

            // Attempt to establish new tunnel within prewarm timeout
            let result = timeout(
                Duration::from_secs(config.prewarm_timeout_secs),
                hot_swap.establish_tunnel(&selection.protocol, &selection.config_hints),
            ).await;

            match result {
                Ok(Ok(new_tunnel)) => {
                    // ── ATOMIC HOT-SWAP ──────────────────────────────────
                    // The TUN device keeps running. Only the underlying
                    // tunnel connection is replaced here.
                    let mut active = active_tunnel.write().await;
                    if let Some(old) = active.take() {
                        old.close().await;
                    }
                    *active = Some(new_tunnel);
                    *current_idx = try_idx % chain.len();
                    // ─────────────────────────────────────────────────────

                    state.lock().await.transition_to(FallbackState::Connected);
                    info!(
                        "✓ Fallback succeeded → {:?} (index {})",
                        selection.protocol, current_idx
                    );
                    return;
                }
                Ok(Err(e)) => {
                    warn!("Protocol {:?} connect failed: {}", selection.protocol, e);
                    // Brief pause before trying next
                    sleep(Duration::from_millis(500)).await;
                }
                Err(_) => {
                    warn!("Protocol {:?} timed out after {}s",
                          selection.protocol, config.prewarm_timeout_secs);
                }
            }
        }

        error!("All fallback attempts exhausted — entering error state");
        state.lock().await.transition_to(FallbackState::AllProtocolsFailed);
    }

    /// Spawn the pre-warmer task that silently connects to the next protocol.
    fn spawn_prewarmer(&self) {
        let warmer = self.prewarmer.clone();
        let chain = self.protocol_chain.clone();
        let current = self.current_index;
        let timeout_secs = self.config.prewarm_timeout_secs;

        tokio::spawn(async move {
            let next_idx = (current + 1) % chain.len();
            if let Some(next) = chain.get(next_idx) {
                info!("Pre-warming next protocol: {:?}", next.protocol);
                warmer.prewarm(&next.protocol, &next.config_hints, timeout_secs).await;
            }
        });
    }

    /// Get current state for status reporting.
    pub async fn current_state(&self) -> FallbackState {
        self.state.lock().await.current()
    }

    /// Get the currently active protocol.
    pub async fn active_protocol(&self) -> Option<Protocol> {
        let tunnel = self.active_tunnel.read().await;
        tunnel.as_ref().map(|t| t.protocol())
    }

    /// Force an immediate manual fallback (for testing or user-triggered).
    pub async fn force_fallback(&mut self) -> Result<(), FallbackError> {
        warn!("Manual fallback forced by user/test");
        let next = self.current_index + 1;
        self.connect_protocol(next).await
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlockSignalSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, thiserror::Error)]
pub enum FallbackError {
    #[error("All protocols in chain exhausted — no working protocol found")]
    AllProtocolsExhausted,
    #[error("Initial connection failed: {0}")]
    ConnectFailed(String),
    #[error("Tunnel is in error state")]
    TunnelError,
}
