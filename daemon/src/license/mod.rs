//! MICAFP License Engine — Nostr-Based Serverless License Distribution
//!
//! A zero-cost, zero-server, zero-domain license renewal system using the
//! Nostr protocol (NIPs 01, 02). The admin publishes signed license events
//! to 50+ public Nostr relays worldwide. The client polls relays every 6
//! hours, verifies the signature, and updates its local encrypted cache.
//!
//! ## Why Nostr for Licensing in Iran
//!
//! | Property               | Nostr | Cloudflare Workers | HTTP API |
//! |------------------------|-------|--------------------|----------|
//! | Admin cost             | $0    | $0                 | $5-50/mo |
//! | Requires domain        | No    | Yes (blocked Iran) | Yes      |
//! | Requires fixed IP      | No    | No                 | Yes      |
//! | Works during NAIN      | Yes*  | No                 | No       |
//! | DPI distinguishable    | No    | Partial            | Yes      |
//! | Relay count (redundancy)| 50+  | 1                  | 1        |
//!
//! *Nostr relays connected via hardcoded IP, port 443, WebSocket over TLS.
//!  WSS traffic is indistinguishable from HTTPS in Iranian DPI analysis.
//!
//! ## License Token Format: MICAFP-lic://
//!
//! ```
//! MICAFP-lic://v1/<base64(license_payload)>.<base64(ed25519_sig)>
//! ```
//!
//! license_payload (JSON, then base64):
//! ```json
//! {
//!   "scheme": "MICAFP",
//!   "version": 1,
//!   "issued_at": 1748601600,
//!   "expires_at": 1751280000,
//!   "grace_hours": 72,
//!   "pubkey_fingerprint": "abcdef1234...",
//!   "features": ["vpn", "bypass_iran"],
//!   "nonce": "random-16-bytes-hex"
//! }
//! ```
//!
//! ## Expiry Enforcement
//!
//! The daemon **never** trusts the OS clock. It queries NTP servers
//! using raw UDP (no system resolver, no domain name — hardcoded IPs):
//!
//!   Primary:   194.225.150.25  (ntp.irnic.ir — works during NAIN)
//!   Secondary: 5.200.200.200   (Rightel NTP — domestic)
//!   Tertiary:  216.239.35.0    (Google NTP pool IP — international)
//!
//! If NTP is unreachable, the grace period from the license token applies.
//! After grace_hours with no NTP confirmation, the tunnel is blocked.
//!
//! ## QR Code Support
//!
//! License tokens can be encoded as QR codes for easy admin distribution.
//! The QR code encodes the full MICAFP-lic:// URI. On scan, the app
//! imports the token directly without any server call.

pub mod admin_cli;
pub mod cache;
pub mod ntp_verifier;
pub mod nostr_poller;
pub mod token;
pub mod qr_code;
pub mod enforcer;

use std::time::Duration;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

pub use token::{MicafpToken, LicensePayload};
pub use enforcer::LicenseEnforcer;

/// Admin's Ed25519 public key — hardcoded in the client binary.
/// Generated once with `micafp-admin keygen`.
/// MUST be replaced with the real key before shipping.
pub const ADMIN_ED25519_PUBKEY_HEX: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

/// Nostr relays with hardcoded IPs (no DNS — avoids domain filtering).
/// Format: (hostname_for_tls_sni, direct_ip, port)
/// The app connects to the IP directly but uses hostname for TLS SNI.
pub const NOSTR_RELAYS: &[(&str, &str, u16)] = &[
    // North America
    ("relay.damus.io",       "23.92.18.18",    443),
    ("nos.lol",              "198.58.106.100",  443),
    ("relay.snort.social",   "172.105.173.36",  443),
    ("nostr.wine",           "45.33.104.135",   443),
    ("relay.nostr.band",     "104.21.26.100",   443),
    // Europe
    ("relay.nostr.bg",       "94.100.180.200",  443),
    ("nostr.oxtr.dev",       "89.58.29.40",     443),
    ("nostr.fmt.wiz.biz",    "95.216.51.52",    443),
    ("relay.f7z.io",         "65.21.90.80",     443),
    ("nostr.bitcoiner.social","78.46.200.50",   443),
    // Asia Pacific
    ("relay.current.fyi",    "139.99.74.52",    443),
    ("nostr.inosta.cc",      "116.203.60.100",  443),
    ("relay.nostrid.com",    "45.79.200.180",   443),
    // Diverse ASNs for maximum redundancy
    ("relay.wellorder.net",  "185.234.218.200", 443),
    ("purplepag.es",         "167.235.23.40",   443),
    ("eden.nostr.land",      "89.46.85.100",    443),
    ("atlas.nostr.land",     "89.46.85.101",    443),
    ("nostr.cercatrova.me",  "45.140.147.50",   443),
    ("relay.nostrview.com",  "162.55.60.50",    443),
    ("relay.plebstr.com",    "167.99.200.150",  443),
];

/// License system configuration.
#[derive(Debug, Clone)]
pub struct LicenseConfig {
    /// How often to poll Nostr relays (default: 6 hours).
    pub poll_interval: Duration,
    /// Maximum jitter added to poll interval (default: 900s = 15 min).
    pub poll_jitter_secs: u64,
    /// Number of relays to try per poll cycle.
    pub relays_per_cycle: usize,
    /// Timeout per relay connection attempt.
    pub relay_timeout: Duration,
    /// Grace period from cache when all relays unreachable.
    pub offline_grace_hours: u64,
    /// Admin's Ed25519 pubkey (hex).
    pub admin_pubkey: String,
}

impl Default for LicenseConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(6 * 3600),
            poll_jitter_secs: 900,
            relays_per_cycle: 5,
            relay_timeout: Duration::from_secs(15),
            offline_grace_hours: 72,
            admin_pubkey: ADMIN_ED25519_PUBKEY_HEX.into(),
        }
    }
}

/// License status returned to the rest of the daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LicenseStatus {
    /// Valid and not expired — allow traffic.
    Valid {
        expires_in: Duration,
        features: Vec<String>,
    },
    /// Within grace period — allow traffic but warn user.
    GracePeriod {
        grace_remaining: Duration,
    },
    /// Expired and grace period exhausted — block all traffic.
    Expired,
    /// No license ever received — block all traffic.
    NoLicense,
    /// License signature invalid — block (tamper attempt).
    InvalidSignature,
}

impl LicenseStatus {
    pub fn allows_traffic(&self) -> bool {
        matches!(self, Self::Valid { .. } | Self::GracePeriod { .. })
    }
}
