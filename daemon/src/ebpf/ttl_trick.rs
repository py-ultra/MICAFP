//! eBPF TTL Manipulation — Expires Packets at DPI Middlebox
//!
//! Sends the TCP SYN and first data packet with a low TTL (e.g. 5) so
//! they expire before reaching the destination but after passing through
//! the DPI box. The DPI box records the connection as "SYN seen" but
//! never sees the payload. Subsequent packets carry the real TTL.
//!
//! ## TTL Values
//!
//! Iranian DPI middleboxes are typically 2-4 hops from the user.
//! The destination server is 8-15 hops away.
//! Setting TTL=5 ensures:
//!   - Packet reaches DPI (2-4 hops away)
//!   - Packet expires before destination (ICMP TTL exceeded sent back)
//!   - Client retransmits with full TTL — DPI sees a "new" connection
//!     and the first data packet is now different
//!
//! This technique breaks stateful DPI that relies on seeing the SYN+data
//! combination to classify a connection.
//!
//! ## Implementation
//!
//! The eBPF tc hook modifies the IP TTL field for the first N packets
//! of each new TCP connection, then restores normal TTL for the rest.

/// TTL trick configuration.
#[derive(Debug, Clone, Copy)]
pub struct TtlTrickConfig {
    /// Low TTL value for trick packets (SYN + first data). Default: 5.
    pub trick_ttl: u8,
    /// Normal TTL for subsequent packets. Default: 64.
    pub normal_ttl: u8,
    /// Number of initial packets to apply trick TTL. Default: 2.
    pub trick_packet_count: u8,
    /// Only apply to new TCP connections (SYN flag). Default: true.
    pub syn_only: bool,
}

impl Default for TtlTrickConfig {
    fn default() -> Self {
        Self {
            trick_ttl: 5,
            normal_ttl: 64,
            trick_packet_count: 2,
            syn_only: false,
        }
    }
}

/// ISP-specific TTL trick configurations.
/// DPI distance varies by ISP network topology.
pub fn config_for_isp(isp_id: &str) -> TtlTrickConfig {
    match isp_id {
        // Irancell DPI is typically 3 hops from user
        "irancell" => TtlTrickConfig { trick_ttl: 4, trick_packet_count: 3, ..Default::default() },
        // ParsOnline DPI is typically 2 hops from user
        "pars_online" => TtlTrickConfig { trick_ttl: 3, trick_packet_count: 3, ..Default::default() },
        // MCI DPI typically 3 hops
        "mci" => TtlTrickConfig { trick_ttl: 4, trick_packet_count: 2, ..Default::default() },
        // Shatel DPI typically 2 hops (fixed line — closer CPE)
        "shatel" => TtlTrickConfig { trick_ttl: 3, trick_packet_count: 2, ..Default::default() },
        // Mokhaberat DPI at hub — typically 4 hops
        "mokhaberat" => TtlTrickConfig { trick_ttl: 5, trick_packet_count: 2, ..Default::default() },
        _ => TtlTrickConfig::default(),
    }
}
