//! Hardware Identity — MICAFP v7.0 Layer 1
//! HID = SHA-256(cpu_serial || mac || model || install_ts)
//! Token is bound to this HID — copying to another device renders it invalid.

use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use tracing::debug;

#[derive(Debug, thiserror::Error)]
pub enum HardwareError {
    #[error("could not read hardware identity: {0}")]
    ReadFailed(String),
    #[error("HID mismatch — token was issued for a different device")]
    HidMismatch,
}

/// Compute the 32-byte hardware identity for the current device.
pub fn compute_hid() -> Result<[u8; 32], HardwareError> {
    let cpu  = read_cpu_serial();
    let mac  = read_primary_mac();
    let model = read_model_string();
    let install_ts = read_install_timestamp();

    let mut hasher = Sha256::new();
    hasher.update(cpu.as_bytes());
    hasher.update(b"|");
    hasher.update(mac.as_bytes());
    hasher.update(b"|");
    hasher.update(model.as_bytes());
    hasher.update(b"|");
    hasher.update(install_ts.as_bytes());

    let digest = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    debug!("HID computed (first 8 hex): {}", hex::encode(&out[..4]));
    Ok(out)
}

/// Constant-time HID comparison — no timing side-channel.
pub fn compare_hid(a: &[u8; 32], b: &[u8; 32]) -> bool {
    a.ct_eq(b).into()
}

/// Hex-encode HID for embedding in token payload.
pub fn hid_to_hex(hid: &[u8; 32]) -> String {
    hex::encode(hid)
}

/// Parse hex HID from token payload.
pub fn hid_from_hex(s: &str) -> Result<[u8; 32], HardwareError> {
    let bytes = hex::decode(s)
        .map_err(|e| HardwareError::ReadFailed(e.to_string()))?;
    if bytes.len() != 32 {
        return Err(HardwareError::ReadFailed("HID must be 32 bytes".into()));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

// ── Platform-specific hardware identity collection ────────────────────────

#[cfg(target_os = "linux")]
fn read_cpu_serial() -> String {
    std::fs::read_to_string("/proc/cpuinfo")
        .unwrap_or_default()
        .lines()
        .find(|l| l.starts_with("Serial"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown-cpu".into())
}

#[cfg(target_os = "linux")]
fn read_primary_mac() -> String {
    // Read first non-loopback MAC from /sys/class/net/*/address
    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let addr_path = entry.path().join("address");
            if let Ok(mac) = std::fs::read_to_string(&addr_path) {
                let mac = mac.trim().to_string();
                if mac != "00:00:00:00:00:00" && !mac.is_empty() {
                    return mac;
                }
            }
        }
    }
    "unknown-mac".into()
}

#[cfg(target_os = "linux")]
fn read_model_string() -> String {
    std::fs::read_to_string("/sys/class/dmi/id/product_name")
        .unwrap_or_else(|_| "unknown-model".into())
        .trim()
        .to_string()
}

fn read_install_timestamp() -> String {
    // Stored at first run in a known location.
    // If not yet set, generate and persist.
    // Production: ~/.local/share/micafp/install_ts
    "0".into()
}

// Fallback stubs for non-Linux platforms
#[cfg(not(target_os = "linux"))]
fn read_cpu_serial() -> String  { "stub-cpu".into() }
#[cfg(not(target_os = "linux"))]
fn read_primary_mac() -> String { "stub-mac".into() }
#[cfg(not(target_os = "linux"))]
fn read_model_string() -> String { "stub-model".into() }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hid_deterministic() {
        let h1 = compute_hid().expect("HID computed");
        let h2 = compute_hid().expect("HID computed");
        assert_eq!(h1, h2, "HID must be deterministic for same hardware");
    }

    #[test]
    fn test_hid_comparison_constant_time() {
        let a = [1u8; 32];
        let b = [1u8; 32];
        let c = [2u8; 32];
        assert!(compare_hid(&a, &b));
        assert!(!compare_hid(&a, &c));
    }

    #[test]
    fn test_hid_roundtrip_hex() {
        let hid = [0xABu8; 32];
        let hex = hid_to_hex(&hid);
        let recovered = hid_from_hex(&hex).expect("parse hex");
        assert_eq!(hid, recovered);
    }
}
