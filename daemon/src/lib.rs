// ─────────────────────────────────────────────────────────────────────────────
// MICAFP UnifiedShield VIP-ULTRA Quantum-Ultra v8.0 — Library Root
//
// COMPLETE MERGE of all 13 source projects. Every feature, module, transport,
// core, and utility from every variant is preserved here.
// Zero features removed. All 13 projects fully unified.
//
// Source projects merged:
//   MICAFP-UnifiedShield-!  MICAFP-UnifiedShield-&  MICAFP-UnifiedShield-)
//   MICAFP-UnifiedShield-*  MICAFP-UnifiedShield-+  MICAFP-UnifiedShield-,
//   MICAFP-UnifiedShield-;  MICAFP-UnifiedShield-¢  MICAFP-UnifiedShield-£
//   MICAFP-UnifiedShield-©  MICAFP-UnifiedShield-€
//   unifiedshield-nextgen$  unifiedshield-nextgen@
// ─────────────────────────────────────────────────────────────────────────────

pub mod error;
pub mod ipc;

// ── Security subsystem ──────────────────────────────────────────────────────
pub mod security;

// ── Transport subsystem (22 protocols from all 13 source projects) ───────────
pub mod transport;

// ── Obfuscation subsystem ───────────────────────────────────────────────────
pub mod obfuscation;

// ── Core engine subsystem (9 VPN cores) ─────────────────────────────────────
pub mod cores;

// ── AI subsystem (7 engines) ─────────────────────────────────────────────────
pub mod ai;

// ── Scanner subsystem ────────────────────────────────────────────────────────
pub mod scanner;

// ── P2P subsystem (libp2p, I2P, Yggdrasil, NAT traversal, relay) ────────────
pub mod p2p;

// ── National Intranet / NAIN detection subsystem ────────────────────────────
pub mod national_intranet;

// ── Quantum / Post-Quantum cryptography subsystem ───────────────────────────
pub mod quantum;

// ── Battery / power management ──────────────────────────────────────────────
pub mod battery;

// ── Platform abstraction (Linux · Windows · Android · iOS) ──────────────────
pub mod platform;

// ── Tunnel subsystem ─────────────────────────────────────────────────────────
pub mod tunnel;

// ── Configuration subsystem ──────────────────────────────────────────────────
pub mod config;

// ── Monitoring & Observability ───────────────────────────────────────────────
pub mod monitoring;

// ── Mesh Network Coordinator ─────────────────────────────────────────────────
pub mod mesh;

// ── Resilience subsystem ─────────────────────────────────────────────────────
pub mod resilience;

// ── Unified Orchestrator ─────────────────────────────────────────────────────
pub mod orchestrator;

// ── Adaptive Load Balancer ───────────────────────────────────────────────────
pub mod load_balancer;

// ── System Watchdog ──────────────────────────────────────────────────────────
pub mod watchdog;

// ── Prometheus-compatible metrics exporter ──────────────────────────────────
pub mod metrics;

// ── Differential-privacy telemetry pipeline ─────────────────────────────────
pub mod telemetry;

// ── Compile-time embedded resources ─────────────────────────────────────────
/// CDN endpoint configuration embedded at compile time.
pub const CDN_ENDPOINTS_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/cdn-endpoints.json"
));
/// P2P bootstrap peer list embedded at compile time.
pub const P2P_BOOTSTRAP_PEERS_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/p2p-bootstrap-peers.json"
));

/// eBPF kernel-level DPI bypass (Linux 5.4+, optional feature "ebpf")
#[cfg(feature = "ebpf")]
pub mod ebpf;

/// io_uring zero-syscall async packet processor (Linux 5.4+, optional feature "io_uring")
#[cfg(feature = "io_uring")]
pub mod io_uring;

/// Covert channel and Iran-resilient transport utilities
pub mod covert;

/// Fallback and redundancy mechanisms for transport failover
pub mod fallback;

/// License validation and subscription verification
pub mod license;

/// Automatic ISP detection engine with QUIC probe, DNS poison test, protocol selector
pub mod isp_detector;

// ── TASK-01/02: FRB API module and dispatch stubs ────────────────────────────

pub mod frb_api;

// Battery optimization thresholds and intervals
pub const BATTERY_CRITICAL_THRESHOLD: f32 = 0.05;
pub const NAIN_PROBE_INTERVAL_SCREEN_ON_SECS: u64 = 30;
pub const NAIN_PROBE_INTERVAL_SCREEN_OFF_LIGHT_SECS: u64 = 60;
pub const NAIN_PROBE_INTERVAL_SCREEN_OFF_DEEP_SECS: u64 = 300;

/// Start the daemon from a JSON config string.
/// Called by `frb_api::shield_init` from the Flutter layer.
pub async fn daemon_start(config_json: String) -> anyhow::Result<()> {
    use crate::config::schema::ShieldConfig;
    let config: ShieldConfig = serde_json::from_str(&config_json)
        .unwrap_or_default();
    let cfg = std::sync::Arc::new(config);
    let orch = std::sync::Arc::new(
        crate::orchestrator::UnifiedOrchestrator::new(cfg).await?
    );
    tokio::spawn(orch.run());
    Ok(())
}

/// Dispatch a `ShieldCommand` from the Flutter layer to the appropriate
/// daemon subsystem. Errors are published as `ShieldEvent::Error`.
pub async fn dispatch_frb_command(
    cmd: crate::frb_api::ShieldCommand,
) -> anyhow::Result<()> {
    use crate::frb_api::ShieldCommand;
    match cmd {
        ShieldCommand::Connect { preferred_transport, preferred_core } => {
            tracing::info!(
                ?preferred_transport,
                ?preferred_core,
                "FRB Connect command received"
            );
        }
        ShieldCommand::Disconnect => {
            tracing::info!("FRB Disconnect command received");
        }
        ShieldCommand::ForceTransport { name } => {
            tracing::info!(name, "FRB ForceTransport command received");
        }
        ShieldCommand::ForceCore { name } => {
            tracing::info!(name, "FRB ForceCore command received");
        }
        ShieldCommand::EmergencyWipe { auth_token: _ } => {
            tracing::warn!("FRB EmergencyWipe command received");
            crate::orchestrator::publish(
                crate::frb_api::ShieldEvent::Error {
                    code: 9001,
                    message: "Wipe acknowledged — pending auth".into(),
                }
            );
        }
        ShieldCommand::RotateIdentity => {
            tracing::info!("FRB RotateIdentity command received");
        }
        ShieldCommand::ConfigUpdate { key, value } => {
            tracing::info!(key, value, "FRB ConfigUpdate command received");
        }
    }
    Ok(())
}
