// ─────────────────────────────────────────────────────────────────────────────
// MICAFP-UnifiedShield v10.0 — Android TUN Interface Handler
//
// TASK-06 implementation.
// Receives the TUN file descriptor from the Kotlin VpnService (via JNI) and
// configures the Rust packet engine. After this point, the daemon AI engine
// owns all routing decisions — no Kotlin or Dart code participates.
//
// Ownership model:
//   Java:   pfd.detachFd()  →  transfers fd ownership to native
//   JNI:    nativeStartTunnel(fd, config_json)  →  calls start_tun()
//   Rust:   OwnedFd takes exclusive ownership, hands to orchestrator
// ─────────────────────────────────────────────────────────────────────────────

use anyhow::{Context, Result};
use tracing::info;

#[cfg(unix)]
use std::os::unix::io::{FromRawFd, OwnedFd};

/// Global flag: whether a TUN session is currently active.
static TUN_ACTIVE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Called from `frb_api::Java_com_micafp_unifiedshield_ShieldVpnService_nativeStartTunnel`.
///
/// # Safety
/// `raw_fd` must be a valid open file descriptor whose ownership has been
/// transferred from Java via `ParcelFileDescriptor.detachFd()`.
#[cfg(unix)]
pub fn start_tun(raw_fd: i32, config_json: &str) -> Result<()> {
    use std::sync::atomic::Ordering;

    if TUN_ACTIVE.load(Ordering::SeqCst) {
        tracing::warn!("start_tun called while TUN session already active — ignoring");
        return Ok(());
    }

    // Safety: fd ownership was transferred from Java; Rust now owns it.
    let owned_fd = unsafe { OwnedFd::from_raw_fd(raw_fd) };
    info!("TUN fd={raw_fd} received from Android VpnService");

    let config: crate::config::schema::ShieldConfig =
        serde_json::from_str(config_json)
            .unwrap_or_default();

    // Publish startup event to Flutter layer.
    crate::orchestrator::publish(crate::frb_api::ShieldEvent::StatusUpdate(
        crate::orchestrator::control_plane::current_snapshot(),
    ));

    TUN_ACTIVE.store(true, Ordering::SeqCst);
    info!("TUN session started; AI engine now owns packet routing");

    // Hand off to the orchestrator. The daemon AI engine owns the fd
    // from this point and processes all packets without Dart/Kotlin involvement.
    crate::orchestrator::attach_tun(owned_fd, config)
        .context("Failed to attach TUN fd to orchestrator")
}

/// Stub for non-Unix targets.
#[cfg(not(unix))]
pub fn start_tun(_raw_fd: i32, _config_json: &str) -> anyhow::Result<()> {
    anyhow::bail!("start_tun is only available on Unix/Android targets")
}

/// Called from `frb_api::Java_com_micafp_unifiedshield_ShieldVpnService_nativeStopTunnel`.
/// Signals the TUN session to stop cleanly.
pub fn stop_tun() -> Result<()> {
    use std::sync::atomic::Ordering;
    TUN_ACTIVE.store(false, Ordering::SeqCst);
    info!("TUN session stop requested via JNI");
    crate::orchestrator::publish(crate::frb_api::ShieldEvent::StatusUpdate(
        crate::frb_api::ShieldStatusSnapshot {
            connected: false,
            ..Default::default()
        },
    ));
    Ok(())
}
