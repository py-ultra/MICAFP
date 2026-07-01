//! Anti-DPI Layer — MICAFP v9.0, Features 10–16
//!
//! Makes every outbound packet completely indistinguishable from Chrome
//! browser traffic at all layers: IP, TCP, TLS, HTTP/2, timing, volume.

use rand::{Rng, RngCore};
use tracing::debug;

/// Chrome 124 cipher suites in exact order (Feature 11 — JA3 clone).
pub const CHROME_124_CIPHER_SUITES: &[u16] = &[
    0x1301, 0x1302, 0x1303,
    0xc02b, 0xc02f, 0xc02c, 0xc030,
    0xcca9, 0xcca8,
];

/// Chrome 124 TLS extension IDs in exact order.
pub const CHROME_124_EXTENSION_ORDER: &[u16] = &[
    0x0000, 0x0017, 0xff01, 0x000a, 0x000b,
    0x0023, 0x0010, 0x0005, 0x0012, 0x0033,
    0x002b, 0x002d, 0x001b, 0x0015,
];

/// Real Chrome 124 User-Agent strings for rotation (Feature 12).
pub const CHROME_124_USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.6367.201 Safari/537.36",
    "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.6367.82 Mobile Safari/537.36",
];

/// Select a random Chrome UA string per-session (Feature 12).
pub fn random_user_agent() -> &'static str {
    let idx = rand::thread_rng().gen_range(0..CHROME_124_USER_AGENTS.len());
    CHROME_124_USER_AGENTS[idx]
}

/// Protocol step for timing normalisation (Feature 13).
#[derive(Debug, Clone, Copy)]
pub enum ProtocolStep {
    TcpSyn,
    TlsClientHello,
    Http2Headers,
    ResponseAck,
}

/// Human-like timing delay per protocol step (Feature 13).
/// Between full poll cycles there is complete silence — no keepalive.
pub async fn human_timing_delay(step: ProtocolStep) {
    use std::time::Duration;
    let ms: u64 = match step {
        ProtocolStep::TcpSyn         => rand::thread_rng().gen_range(2..=8),
        ProtocolStep::TlsClientHello => rand::thread_rng().gen_range(1..=3),
        ProtocolStep::Http2Headers   => rand::thread_rng().gen_range(5..=20),
        ProtocolStep::ResponseAck    => 1,
    };
    debug!("Anti-DPI timing: {:?} → {}ms delay", step, ms);
    tokio::time::sleep(Duration::from_millis(ms)).await;
}

/// Pad TLS application record to next 512-byte boundary (Feature 14).
pub fn pad_record_to_512(data: &mut Vec<u8>) {
    if data.is_empty() { return; }
    let target = ((data.len() / 512) + 1) * 512;
    let needed = target - data.len();
    let mut padding = vec![0u8; needed];
    rand::thread_rng().fill_bytes(&mut padding);
    data.extend_from_slice(&padding);
}

/// Channel transport diversity (Feature 15).
/// Returns the port/protocol for each channel ID.
pub fn channel_transport(id: u8) -> (&'static str, u16) {
    match id {
        1  => ("UDP",  53),
        2  => ("TCP",  443),
        3  => ("DTLS", 443),
        4  => ("UDP",  6881),
        5  => ("WSS",  443),
        6  => ("HTTPS",443),
        7  => ("HTTPS",443),
        8  => ("UDP",  0),
        9  => ("TCP",  0),
        10 => ("UDP",  0),
        _  => ("TCP",  443),
    }
}

/// obfs4 detection risk threshold (Feature 16).
/// If detection_risk > 0.3, wrap channel in obfs4.
pub fn should_use_obfs4(detection_risk: f32) -> bool {
    detection_risk > 0.3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pad_record_multiple_of_512() {
        let mut d = vec![0u8; 100];
        pad_record_to_512(&mut d);
        assert_eq!(d.len(), 512);

        let mut d2 = vec![0u8; 513];
        pad_record_to_512(&mut d2);
        assert_eq!(d2.len(), 1024);

        let mut d3 = vec![0u8; 1024];
        pad_record_to_512(&mut d3);
        assert_eq!(d3.len(), 1536);
    }

    #[test]
    fn test_random_user_agent_valid() {
        let ua = random_user_agent();
        assert!(ua.contains("Chrome/124"));
    }

    #[test]
    fn test_obfs4_threshold() {
        assert!(!should_use_obfs4(0.2));
        assert!(!should_use_obfs4(0.3));
        assert!(should_use_obfs4(0.31));
        assert!(should_use_obfs4(0.9));
    }
}
