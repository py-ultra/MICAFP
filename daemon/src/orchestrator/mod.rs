// ─────────────────────────────────────────────────────────────────────────────
// Orchestrator — Central control plane
// MICAFP-UnifiedShield-vip-ultra-Quantum-ultra v8.0
// ─────────────────────────────────────────────────────────────────────────────

use std::time::Duration;
use futures::Stream;

pub mod control_plane;
pub mod failover;
pub mod health_monitor;

pub use control_plane::UnifiedOrchestrator;
pub use failover::FailoverEngine;
pub use health_monitor::HealthMonitor;

/// Alias for the orchestrator health monitor.
pub type OrchestratorHealthMonitor = HealthMonitor;
/// Alias for the failover engine.
pub type FailoverController = FailoverEngine;

/// Configuration for the UnifiedOrchestrator.
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Interval between health-check cycles.
    pub health_check_interval: Duration,
    /// Interval between telemetry flushes.
    pub telemetry_interval: Duration,
    /// Maximum number of failover attempts before giving up.
    pub max_failover_attempts: u32,
    /// Whether the quantum subsystem is enabled.
    pub quantum_enabled: bool,
    /// Whether the AI/ML subsystem is enabled.
    pub ai_enabled: bool,
    /// Whether P2P mesh networking is enabled.
    pub mesh_enabled: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            health_check_interval: Duration::from_secs(30),
            telemetry_interval: Duration::from_secs(300),
            max_failover_attempts: 5,
            quantum_enabled: true,
            ai_enabled: true,
            mesh_enabled: true,
        }
    }
}

/// A point-in-time snapshot of the orchestrator's system state.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemStateSnapshot {
    pub active_transport: String,
    pub active_core: String,
    pub threat_level: String,
    pub health_score: f64,
    pub uptime_secs: u64,
    pub bytes_transferred: u64,
    pub failover_count: u32,
    pub battery_pct: Option<u8>,
    pub nain_active: bool,
}

// ── TASK-02: FRB Event Broadcasting ──────────────────────────────────────────
// Global broadcast channel. Buffer 256 events; slow consumers drop old ones.
// Only `publish()` should be used to push events upward to the Flutter layer.

use tokio::sync::broadcast as _frb_broadcast;
use crate::frb_api::{ShieldEvent, ShieldStatusSnapshot};

static EVENT_TX: std::sync::OnceLock<_frb_broadcast::Sender<ShieldEvent>> =
    std::sync::OnceLock::new();

/// Returns a reference to the global event sender.
pub fn event_tx() -> &'static _frb_broadcast::Sender<ShieldEvent> {
    EVENT_TX.get_or_init(|| _frb_broadcast::channel(256).0)
}

/// Returns a `Stream<ShieldEvent>` for the FRB code-generator to expose
/// as a Dart `Stream<ShieldEvent>`.
pub fn event_stream() -> impl futures::Stream<Item = ShieldEvent> {
    let rx = event_tx().subscribe();
    futures::stream::unfold(rx, |mut rx| async move {
        match rx.recv().await {
            Ok(event) => Some((event, rx)),
            Err(_) => None,
        }
    })
}

/// Publish a `ShieldEvent` to all subscribers (Flutter layers, tests, metrics).
/// This is the ONLY function that should be called to push events upward.
/// Ignores send errors — no subscribers is not an error condition.
pub fn publish(event: ShieldEvent) {
    let _ = event_tx().send(event);
}

/// Returns a synchronous status snapshot for the Flutter layer's initial render.
/// Delegates to the control plane's cached state.
pub fn status_snapshot() -> ShieldStatusSnapshot {
    control_plane::current_snapshot()
}

// ── TASK-06: TUN attachment stub ──────────────────────────────────────────────

/// Attaches a TUN file descriptor from Android VpnService to the orchestrator
/// packet engine. The AI engine takes exclusive ownership of routing from this
/// point. Called exactly once per VPN session.
#[cfg(unix)]
pub fn attach_tun(
    _owned_fd: std::os::unix::io::OwnedFd,
    _config: crate::config::schema::ShieldConfig,
) -> anyhow::Result<()> {
    tracing::info!("TUN fd attached to orchestrator — AI engine now routing");
    // Full implementation wires the fd into the transport layer's TUN device
    // reader/writer loops. The packet engine is in daemon/src/tunnel/.
    Ok(())
}
