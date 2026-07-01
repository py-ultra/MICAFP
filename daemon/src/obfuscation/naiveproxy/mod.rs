//! NaiveProxy Integration — HTTP/2 Browser Traffic Masquerade
//!
//! NaiveProxy routes VPN traffic through a Caddy server using the
//! real Chromium network stack (via a forked Chrome binary). The
//! proxy traffic is completely indistinguishable from Chrome browser
//! traffic because it literally IS Chrome HTTP/2 over TLS.
//!
//! ## Why NaiveProxy Beats Other Protocols on Certain Iranian ISPs
//!
//! DPI systems model "what Chrome traffic looks like" based on:
//!   - TLS fingerprint (JA3 hash)
//!   - HTTP/2 SETTINGS frame parameters
//!   - HTTP/2 header ordering (HPACK compression state)
//!   - ALPN negotiation order
//!   - Certificate pinning behaviour
//!   - TCP congestion algorithm fingerprint
//!
//! NaiveProxy passes ALL of these checks because Chromium itself
//! handles the TLS and HTTP/2 layers. No imitation — it's the real thing.
//!
//! ## Best Iranian ISPs for NaiveProxy
//!
//! | ISP           | Recommended | Reason                              |
//! |---------------|-------------|-------------------------------------|
//! | Asiatech      | ✓ Best      | Medium DPI, no ML classifier        |
//! | Afranet       | ✓ Best      | Medium DPI, no active probing        |
//! | Rightel       | ✓ Good      | FAVA v2.1, HTTP/2 not fingerprinted |
//! | Mobinnet      | ✓ Good      | Light filtering                     |
//! | Irancell      | ~ Partial   | FAVA v4 can detect flow patterns    |
//! | ParsOnline    | ✗ Avoid     | FAVA v4.1 ML classifies naiveproxy  |
//!
//! ## Server Requirements
//!
//! Server side: Caddy with forward-proxy plugin:
//! ```json
//! {
//!   "apps": { "http": { "servers": { "proxy": {
//!     "listen": [":443"],
//!     "routes": [{ "handle": [{
//!       "handler": "forward_proxy",
//!       "hide_ip": true,
//!       "hide_via": true,
//!       "auth_user_deprecated": "user",
//!       "auth_pass_deprecated": "pass"
//!     }]}]
//!   }}}}
//! }
//! ```

use std::net::SocketAddr;

/// NaiveProxy client configuration.
#[derive(Debug, Clone)]
pub struct NaiveProxyConfig {
    /// Caddy server address.
    pub server: SocketAddr,
    /// Domain name of the Caddy server (for TLS SNI).
    pub domain: String,
    /// HTTP CONNECT proxy credentials.
    pub username: String,
    pub password: String,
    /// Listen address for local SOCKS5/HTTP proxy.
    pub local_listen: String,
    /// Use HTTP/2 (recommended) or HTTP/1.1 (fallback).
    pub use_http2: bool,
    /// Padding enabled to normalise traffic patterns.
    pub padding: bool,
}

impl Default for NaiveProxyConfig {
    fn default() -> Self {
        Self {
            server: "0.0.0.0:443".parse().unwrap(),
            domain: "your-caddy-server.example.com".into(),
            username: "user".into(),
            password: "strong-password".into(),
            local_listen: "127.0.0.1:1080".into(),
            use_http2: true,
            padding: true,
        }
    }
}

/// Generate a Caddy v2 JSON config for NaiveProxy server.
pub fn generate_caddy_config(domain: &str, username: &str, password: &str) -> serde_json::Value {
    serde_json::json!({
        "apps": {
            "http": {
                "servers": {
                    "naive-proxy": {
                        "listen": [":443"],
                        "tls_connection_policies": [{}],
                        "routes": [{
                            "handle": [{
                                "handler": "forward_proxy",
                                "hide_ip": true,
                                "hide_via": true,
                                "probe_resistance": {
                                    "domain": domain
                                },
                                "basic_auth": {
                                    username: password
                                }
                            }]
                        }]
                    }
                }
            },
            "tls": {
                "automation": {
                    "policies": [{
                        "subjects": [domain],
                        "on_demand": false
                    }]
                }
            }
        },
        "_comment": "NaiveProxy Caddy config for UnifiedShield v8.0"
    })
}

/// ISP-specific NaiveProxy recommendations.
pub fn recommendation_for_isp(isp_id: &str) -> &'static str {
    match isp_id {
        "asiatech" | "afranet"  => "STRONGLY RECOMMENDED — best choice for these ISPs",
        "rightel"  | "mobinnet" => "RECOMMENDED — good alternative to Reality",
        "mci"      | "shatel"   => "USABLE — combine with traffic padding for best results",
        "irancell"              => "PARTIAL — FAVA v4 may classify after many flows; rotate servers",
        "pars_online"           => "NOT RECOMMENDED — FAVA v4.1 ML has learned NaiveProxy patterns",
        "mokhaberat"            => "USABLE — works but Reality is preferred on TCI",
        _                       => "USABLE — test and monitor",
    }
}
