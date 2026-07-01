//! QUIC Availability Probe
//!
//! Tests whether UDP/QUIC port 443 is reachable from the current network.
//! Some Iranian ISPs (MCI, Irancell, Shatel, ParsOnline) block UDP 443.
//!
//! Method: Send a minimal QUIC Initial packet to a known-good server
//! (Cloudflare 1.1.1.1 or Google 8.8.8.8) and wait for a response.
//! If no response within 2 seconds, assume QUIC is blocked.
//!
//! We use multiple probe targets to avoid false negatives from server-side
//! QUIC support variability:
//!   - 1.1.1.1:443  (Cloudflare, blocked in Iran but QUIC probe works)
//!   - 8.8.4.4:443  (Google, partially blocked)
//!   - speedtest.net:443 (Ookla, usually unblocked)

use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

/// Probe targets for QUIC availability test.
static QUIC_PROBE_TARGETS: &[&str] = &[
    "1.1.1.1:443",         // Cloudflare
    "8.8.4.4:443",         // Google DNS
    "208.67.222.222:443",  // OpenDNS
];

/// Minimal QUIC Initial packet bytes (RFC 9000 §17.2.2).
/// Contains a random DCID and version 0x00000001 (QUIC v1).
/// Server will respond with a Version Negotiation or ServerHello
/// if QUIC is reachable.
const QUIC_PROBE_PACKET: &[u8] = &[
    0xC3,                           // Long header, QUIC v1
    0x00, 0x00, 0x00, 0x01,         // Version: QUIC v1
    0x08,                           // DCID length: 8 bytes
    0x01, 0x02, 0x03, 0x04,         // DCID (random)
    0x05, 0x06, 0x07, 0x08,
    0x00,                           // SCID length: 0
    0x00,                           // Token length: 0
    0x40, 0x19,                     // Packet length (var-int): 25 bytes
    0x00,                           // Packet number: 0
];

/// Test if QUIC is available by sending a probe packet and waiting for response.
pub async fn test_quic_availability() -> bool {
    for target_str in QUIC_PROBE_TARGETS {
        if let Ok(result) = probe_target(target_str) {
            if result {
                tracing::debug!("QUIC probe succeeded via {}", target_str);
                return true;
            }
        }
    }
    tracing::debug!("QUIC probe failed on all targets — QUIC likely blocked");
    false
}

fn probe_target(target: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(Duration::from_millis(2000)))?;

    let addr: SocketAddr = target.parse()?;
    socket.send_to(QUIC_PROBE_PACKET, addr)?;

    let mut buf = [0u8; 256];
    match socket.recv_from(&mut buf) {
        Ok((n, _)) => {
            // Any response means QUIC is reachable
            Ok(n > 0)
        }
        Err(_) => Ok(false), // Timeout or ICMP unreachable
    }
}
