//! eBPF DISORDER Mode — Out-of-Order TCP Segment Delivery
//!
//! Reorders TCP segments so the DPI reassembly engine is confused.
//! The destination TCP stack handles reordering via its receive buffer.
//! FAVA v2.x and earlier do not handle out-of-order reassembly correctly.
//!
//! ## Disorder Pattern
//!
//! For a 3-segment TLS ClientHello [SEG1, SEG2, SEG3]:
//!
//!   Normal:   SEG1 → SEG2 → SEG3
//!   Disorder: SEG2 → SEG1 → SEG3  (SEG1 retransmitted after SEG2)
//!
//! The DPI sees SEG2 first (containing the middle of ClientHello)
//! and cannot parse the TLS header or SNI without SEG1 first.
//! When SEG1 arrives "late", FAVA v2 has already moved on and
//! does not retroactively re-classify the flow.
//!
//! ## Safety
//!
//! Only applied to the TLS ClientHello (first ~300 bytes of a TLS session).
//! All subsequent data flows normally. The endpoint's TCP stack
//! correctly buffers and reorders via the receive window.

/// DISORDER mode configuration.
#[derive(Debug, Clone, Copy)]
pub struct DisorderConfig {
    /// Number of segments to disorder in the initial handshake. Default: 2.
    pub disorder_count: u8,
    /// Delay (ms) before sending the "held" out-of-order segment. Default: 5.
    pub hold_delay_ms: u32,
    /// Only apply disorder to TLS ClientHello (safest). Default: true.
    pub tls_only: bool,
}

impl Default for DisorderConfig {
    fn default() -> Self {
        Self { disorder_count: 2, hold_delay_ms: 5, tls_only: true }
    }
}

pub fn config_for_isp(isp_id: &str) -> DisorderConfig {
    match isp_id {
        // Rightel FAVA v2.1 — disorder is very effective
        "rightel" => DisorderConfig { disorder_count: 2, hold_delay_ms: 3, tls_only: true },
        // Pishgaman FAVA v2.0 — disorder effective
        "pishgaman" => DisorderConfig { disorder_count: 2, hold_delay_ms: 5, tls_only: true },
        // Mokhaberat FAVA v2.5 — partially effective
        "mokhaberat" => DisorderConfig { disorder_count: 2, hold_delay_ms: 8, tls_only: true },
        // Higher FAVA versions handle reordering — use SniSplit instead
        _ => DisorderConfig { disorder_count: 1, hold_delay_ms: 2, tls_only: true },
    }
}
