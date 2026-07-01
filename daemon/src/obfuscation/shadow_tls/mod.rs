//! ShadowTLS v3 — Anti-Active-Probing TLS Disguise Layer
//!
//! ShadowTLS v3 wraps any proxy protocol inside a *real* TLS handshake
//! with a trusted server (e.g., www.apple.com). Unlike fake-TLS approaches,
//! ShadowTLS actually completes the TLS handshake with the trusted server
//! so that DPI active probing receives a *genuine* TLS certificate.
//!
//! ## Why ShadowTLS v3 vs v1/v2?
//!
//! | Version | Active-probing resistant | Server-side auth | Traffic padding |
//! |---------|--------------------------|------------------|-----------------|
//! | v1      | No                       | No               | No              |
//! | v2      | Partial (HMAC in header) | No               | No              |
//! | v3      | Yes (full auth)          | Yes (pre-shared) | Yes             |
//!
//! v3 uses a pre-shared password to generate a per-connection HMAC tag
//! embedded in the TLS application data stream. Only the real ShieldVPN
//! server recognises this tag. Active probers get a genuine TLS connection
//! to www.apple.com and see nothing suspicious.
//!
//! ## Flow
//!
//! 1. Client connects to ShieldVPN server IP:443.
//! 2. Client performs *real* TLS handshake with trusted SNI (e.g., apple.com).
//!    The server proxies this handshake to the real apple.com and relays it back.
//! 3. Client sends HMAC-SHA256 derived tag using pre-shared password + nonce.
//! 4. Server recognises tag → switches to proxy mode (VLESS/VMess/Trojan).
//! 5. If tag is absent/wrong (active probing) → relay real apple.com traffic.
//!
//! ## Iran-Specific Configuration
//!
//! Best SNIs for Iranian DPI evasion (avoid SNIs that are themselves blocked):
//!   - www.apple.com       (Apple CDN — high trust score in Iranian DPI)
//!   - www.microsoft.com   (Windows Update — never blocked)
//!   - update.googleapis.com (Android updates — expected traffic)
//!   - addons.mozilla.org  (Firefox extensions — trusted)
//!
//! Do NOT use:
//!   - Any .ir domain (domestic, no obfuscation benefit)
//!   - Any CDN blocked in Iran (Cloudflare, Fastly, etc.)

pub mod client;
pub mod handshake;
pub mod hmac_auth;
pub mod server;

use std::net::SocketAddr;
use zeroize::Zeroize;

/// ShadowTLS v3 configuration.
#[derive(Debug, Clone)]
pub struct ShadowTlsV3Config {
    /// Pre-shared password for HMAC authentication.
    /// Must match exactly between client and server.
    pub password: ShadowTlsPassword,
    /// SNI to use for the real TLS handshake (trusted domain).
    pub sni: String,
    /// Fallback SNIs if primary is filtered.
    pub fallback_snis: Vec<String>,
    /// TLS handshake timeout.
    pub handshake_timeout_ms: u64,
    /// Enable strict mode: disconnect on any unexpected byte before auth.
    pub strict_mode: bool,
    /// Enable traffic padding to fixed-size records.
    pub padding_enabled: bool,
    /// Target record size for padding (bytes).
    pub padding_target_bytes: usize,
}

/// Zero-on-drop password container.
#[derive(Debug, Clone)]
pub struct ShadowTlsPassword(pub String);

impl Drop for ShadowTlsPassword {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl Default for ShadowTlsV3Config {
    fn default() -> Self {
        Self {
            password: ShadowTlsPassword("change-me-strong-password".into()),
            sni: "www.apple.com".into(),
            fallback_snis: vec![
                "www.microsoft.com".into(),
                "update.googleapis.com".into(),
                "addons.mozilla.org".into(),
            ],
            handshake_timeout_ms: 8000,
            strict_mode: true,
            padding_enabled: true,
            padding_target_bytes: 1400,
        }
    }
}

/// Per-connection HMAC authenticator for ShadowTLS v3.
pub struct ShadowTlsAuthenticator {
    /// HMAC-SHA256 key derived from password via HKDF.
    key: [u8; 32],
}

impl ShadowTlsAuthenticator {
    /// Derive HMAC key from password using HKDF-SHA256.
    pub fn from_password(password: &ShadowTlsPassword) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Production: use proper HKDF (hkdf crate).
        // Here: deterministic derivation for structure demonstration.
        let mut hasher = DefaultHasher::new();
        password.0.hash(&mut hasher);
        let seed = hasher.finish().to_le_bytes();
        let mut key = [0u8; 32];
        for (i, b) in seed.iter().cycle().take(32).enumerate() {
            key[i] = *b ^ (i as u8);
        }
        Self { key }
    }

    /// Generate authentication tag for a given nonce.
    pub fn generate_tag(&self, nonce: &[u8]) -> [u8; 32] {
        // Production: HMAC-SHA256(key, nonce)
        // Placeholder deterministic computation:
        let mut tag = [0u8; 32];
        for (i, (k, n)) in self.key.iter()
            .zip(nonce.iter().cycle())
            .take(32)
            .enumerate()
        {
            tag[i] = k ^ n ^ (i as u8).wrapping_mul(7);
        }
        tag
    }

    /// Verify a received authentication tag.
    pub fn verify_tag(&self, nonce: &[u8], tag: &[u8; 32]) -> bool {
        let expected = self.generate_tag(nonce);
        // Constant-time comparison
        expected.iter().zip(tag.iter()).fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0
    }
}

/// Best SNI domains for Iranian DPI evasion, ranked by reliability.
pub const IRAN_RECOMMENDED_SNIS: &[(&str, &str)] = &[
    ("www.apple.com",           "Apple CDN — very high DPI trust, never blocked in Iran"),
    ("www.microsoft.com",       "Windows Update — blocked would break enterprise; very safe"),
    ("update.googleapis.com",   "Android OTA updates — expected on mobile ISPs"),
    ("addons.mozilla.org",      "Firefox add-ons — trusted on all Iranian ISPs"),
    ("www.speedtest.net",       "Ookla speedtest — whitelisted by most ISPs for QoS"),
    ("dl.google.com",           "Google downloads — high volume, trusted by DPI"),
    ("ocsp.apple.com",          "Apple OCSP — certificate validation traffic, always allowed"),
    ("captive.apple.com",       "iOS captive portal check — extremely trusted"),
];
