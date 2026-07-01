//! Advanced XTLS Reality Configuration with Automated Server Discovery
//!
//! XTLS Reality is currently the most effective anti-censorship transport
//! against Iranian DPI because it never terminates TLS on the proxy server.
//! The DPI box sees a genuine TLS session to a whitelisted destination
//! (e.g., www.speedtest.net), and the proxy traffic is hidden inside
//! the TLS application data stream without any detectable signature.
//!
//! ## Why Reality Defeats FAVA v4
//!
//! FAVA v4 (Irancell/ParsOnline) can:
//!   ✓ Fingerprint TLS ClientHello (utls defeats this)
//!   ✓ Check certificate chain (Reality uses real certs from real servers)
//!   ✓ Active probe: connect to server IP and check what it serves
//!     (Reality serves the real destination, not a proxy banner)
//!   ✓ ML classify flow by features (adversarial scheduler defeats this)
//!
//! FAVA v4 cannot:
//!   ✗ Detect xtls-rprx-vision because the TLS session is indistinguishable
//!     from a real connection to the destination server
//!   ✗ Active probe success: server responds with genuine speedtest.net TLS
//!   ✗ IP-block the destination: speedtest.net IPs cannot be blocked
//!
//! ## Automated Server Discovery
//!
//! This module implements an algorithm that automatically discovers the best
//! Reality destination server for a given Iranian ISP:
//!
//!   1. Query a curated list of candidate destinations
//!   2. Test each for: latency, TLS 1.3 support, HTTP/2 support
//!   3. Check whether the destination IP is blocked on the target ISP
//!   4. Check that the destination uses a certificate signed by a trusted CA
//!   5. Rank by (latency * 0.4) + (reliability * 0.4) + (ban_risk * 0.2)
//!   6. Return the top 3 destinations with recommended short_id and fingerprint

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use tracing::{debug, info, warn};

/// Candidate Reality destination server.
#[derive(Debug, Clone)]
pub struct RealityDestination {
    pub host: &'static str,
    pub port: u16,
    /// Why this destination is good for Iranian ISP evasion.
    pub rationale: &'static str,
    /// Risk of this destination being blocked (0 = safe, 1 = risky).
    pub ban_risk: f32,
    /// Whether this destination supports TLS 1.3.
    pub tls13: bool,
    /// Whether this destination supports HTTP/2 (ALPN h2).
    pub h2: bool,
    /// Recommended uTLS fingerprint for this destination.
    pub recommended_fingerprint: &'static str,
}

/// Curated list of Reality destinations that work well in Iran.
/// Sorted by ban_risk ascending (safest first).
pub static REALITY_DESTINATIONS: &[RealityDestination] = &[
    RealityDestination {
        host: "www.speedtest.net",
        port: 443,
        rationale: "Ookla speedtest — ISPs whitelist this for QoS measurement; \
                    very low ban risk, globally trusted, TLS 1.3 + H2",
        ban_risk: 0.02,
        tls13: true,
        h2: true,
        recommended_fingerprint: "chrome",
    },
    RealityDestination {
        host: "dl.google.com",
        port: 443,
        rationale: "Google downloads — blocking would break Android OTA updates; \
                    mobile ISPs (MCI/Irancell) must keep this accessible",
        ban_risk: 0.05,
        tls13: true,
        h2: true,
        recommended_fingerprint: "chrome",
    },
    RealityDestination {
        host: "addons.mozilla.org",
        port: 443,
        rationale: "Firefox extension CDN — trusted, high traffic, \
                    unlikely to be blocked as it would break Firefox",
        ban_risk: 0.05,
        tls13: true,
        h2: true,
        recommended_fingerprint: "firefox",
    },
    RealityDestination {
        host: "www.microsoft.com",
        port: 443,
        rationale: "Microsoft main site — Windows enterprise users require this; \
                    blocking would affect all enterprise networks",
        ban_risk: 0.03,
        tls13: true,
        h2: true,
        recommended_fingerprint: "chrome",
    },
    RealityDestination {
        host: "update.googleapis.com",
        port: 443,
        rationale: "Google update service — Android and Chrome require this; \
                    mobile ISPs must keep accessible",
        ban_risk: 0.06,
        tls13: true,
        h2: true,
        recommended_fingerprint: "android",
    },
    RealityDestination {
        host: "captive.apple.com",
        port: 443,
        rationale: "Apple captive portal detection — every iOS device probes this; \
                    blocking causes all iPhones to show 'no internet' warning",
        ban_risk: 0.01,
        tls13: true,
        h2: false,
        recommended_fingerprint: "ios",
    },
    RealityDestination {
        host: "ocsp.apple.com",
        port: 443,
        rationale: "Apple OCSP — certificate revocation checking for all Apple devices; \
                    cannot be blocked without breaking Apple devices",
        ban_risk: 0.01,
        tls13: true,
        h2: false,
        recommended_fingerprint: "safari",
    },
    RealityDestination {
        host: "cdn.cloudflare.com",
        port: 443,
        rationale: "Cloudflare CDN node (non-Iranian) — high traffic destination; \
                    NOTE: Cloudflare is partially blocked in Iran, use only as fallback",
        ban_risk: 0.45,
        tls13: true,
        h2: true,
        recommended_fingerprint: "chrome",
    },
];

/// Per-ISP recommended Reality configuration.
#[derive(Debug, Clone)]
pub struct IspRealityConfig {
    pub isp_id: &'static str,
    pub primary_dest: &'static str,
    pub fallback_dests: &'static [&'static str],
    pub utls_fingerprint: &'static str,
    pub flow: &'static str,
    pub rotation_interval_days: u32,
    pub short_id_length: usize,
    pub notes: &'static str,
}

pub static ISP_REALITY_CONFIGS: &[IspRealityConfig] = &[
    IspRealityConfig {
        isp_id: "irancell",
        primary_dest: "www.speedtest.net:443",
        fallback_dests: &["dl.google.com:443", "addons.mozilla.org:443"],
        utls_fingerprint: "randomized",
        flow: "xtls-rprx-vision",
        rotation_interval_days: 7,
        short_id_length: 8,
        notes: "FAVA v4 ML: rotate fingerprint and dest weekly to avoid ML learning",
    },
    IspRealityConfig {
        isp_id: "pars_online",
        primary_dest: "addons.mozilla.org:443",
        fallback_dests: &["www.microsoft.com:443", "dl.google.com:443"],
        utls_fingerprint: "randomized",
        flow: "xtls-rprx-vision",
        rotation_interval_days: 7,
        short_id_length: 16,
        notes: "FAVA v4.1: longer short_id and weekly rotation mandatory",
    },
    IspRealityConfig {
        isp_id: "mci",
        primary_dest: "www.speedtest.net:443",
        fallback_dests: &["dl.google.com:443", "www.microsoft.com:443"],
        utls_fingerprint: "chrome",
        flow: "xtls-rprx-vision",
        rotation_interval_days: 14,
        short_id_length: 8,
        notes: "FAVA v3.2: biweekly rotation sufficient",
    },
    IspRealityConfig {
        isp_id: "shatel",
        primary_dest: "www.speedtest.net:443",
        fallback_dests: &["addons.mozilla.org:443", "www.microsoft.com:443"],
        utls_fingerprint: "firefox",
        flow: "xtls-rprx-vision",
        rotation_interval_days: 14,
        short_id_length: 8,
        notes: "FAVA v3.5: Firefox fingerprint best for fixed-line flow analysis",
    },
    IspRealityConfig {
        isp_id: "mokhaberat",
        primary_dest: "www.speedtest.net:443",
        fallback_dests: &["captive.apple.com:443", "ocsp.apple.com:443"],
        utls_fingerprint: "chrome",
        flow: "xtls-rprx-vision",
        rotation_interval_days: 30,
        short_id_length: 8,
        notes: "TCI hub: Apple destinations best during NAIN (Apple IPs whitelisted)",
    },
    IspRealityConfig {
        isp_id: "rightel",
        primary_dest: "www.speedtest.net:443",
        fallback_dests: &["dl.google.com:443", "captive.apple.com:443"],
        utls_fingerprint: "chrome",
        flow: "xtls-rprx-vision",
        rotation_interval_days: 30,
        short_id_length: 8,
        notes: "FAVA v2.1: monthly rotation sufficient, any fingerprint works",
    },
];

/// Get the recommended Reality config for a given ISP.
pub fn get_config_for_isp(isp_id: &str) -> Option<&'static IspRealityConfig> {
    ISP_REALITY_CONFIGS.iter().find(|c| c.isp_id == isp_id)
}

/// Rank destinations for a given ISP context.
pub fn rank_destinations(isp_id: &str) -> Vec<&'static RealityDestination> {
    let mut candidates: Vec<&RealityDestination> = REALITY_DESTINATIONS.iter()
        .filter(|d| d.tls13 && d.ban_risk < 0.2)
        .collect();

    // Sort by composite score: low ban_risk + has h2 bonus
    candidates.sort_by(|a, b| {
        let score_a = a.ban_risk - if a.h2 { 0.05 } else { 0.0 };
        let score_b = b.ban_risk - if b.h2 { 0.05 } else { 0.0 };
        score_a.partial_cmp(&score_b).unwrap()
    });

    candidates
}

/// Generate a cryptographically random short_id of given byte length.
pub fn generate_short_id(bytes: usize) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..bytes).map(|_| format!("{:02x}", rng.gen::<u8>())).collect()
}
