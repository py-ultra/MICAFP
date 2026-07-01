//! ArvanCloud CDN Relay — NAIN-Safe VPN Transport
//!
//! When Iran activates National Internet mode (NAIN), all international BGP
//! routes are withdrawn. Only domestic IPs remain reachable. ArvanCloud (IR CDN)
//! is on the government whitelist and stays accessible throughout NAIN events.
//!
//! ## Strategy
//!
//! VPN server sits behind ArvanCloud CDN. The client connects to ArvanCloud's
//! domestic IP. ArvanCloud forwards traffic to the origin server (which may
//! be outside Iran, but ArvanCloud's backbone handles the cross-border transit
//! since CDN transit is whitelisted even during NAIN).
//!
//! ## Configuration Requirements
//!
//! 1. Domain pointing to ArvanCloud nameservers
//! 2. ArvanCloud CDN enabled with "WebSocket" support turned on
//! 3. Origin server running VLESS/VMess/Trojan over WebSocket+TLS on port 443
//! 4. ArvanCloud cache bypass headers configured so proxy traffic is not cached
//!
//! ## Why This Works During NAIN
//!
//! ArvanCloud has a domestic PoP (Point of Presence) inside Iran. During NAIN,
//! traffic from the user's device travels domestically to ArvanCloud's Iranian
//! PoP. ArvanCloud's internal backbone then forwards to the origin server
//! through its own international links (which are separate from the BGP
//! routes that get withdrawn). The user never directly accesses international IPs.
//!
//! ## Hardcoded ArvanCloud IP Ranges (for NAIN mode direct connection)
//!
//! During NAIN, DNS may also be disrupted. We hardcode ArvanCloud's domestic
//! CDN IPs so the client can connect without DNS resolution:

use std::net::{IpAddr, Ipv4Addr};

/// ArvanCloud domestic CDN IP ranges (Iranian PoPs).
/// These IPs are reachable during NAIN mode.
pub static ARVANCLOUD_DOMESTIC_IPS: &[&str] = &[
    "185.215.232.1",
    "185.215.232.2",
    "185.215.232.10",
    "185.215.232.50",
    "185.143.234.42",
    "185.143.234.50",
    "185.143.234.100",
    "188.114.98.1",
    "188.114.99.1",
];

/// ASNs associated with ArvanCloud (whitelisted during NAIN).
pub static ARVANCLOUD_ASNS: &[u32] = &[
    208743,  // Arvan Cloud AS (primary)
    47447,   // Arvan Cloud AS (secondary)
    210644,  // Arvan CDN
];

/// NAIN-safe transport endpoint configuration.
#[derive(Debug, Clone)]
pub struct NainSafeEndpoint {
    /// The user's ArvanCloud-fronted domain.
    pub domain: String,
    /// Host header to send (same domain for direct fronting).
    pub host_header: String,
    /// Path for WebSocket upgrade.
    pub ws_path: String,
    /// Fallback direct IPs to use if DNS resolution fails during NAIN.
    pub fallback_ips: Vec<IpAddr>,
    /// Port (almost always 443 for NAIN compatibility).
    pub port: u16,
    /// Protocol running behind the CDN (vless-ws-tls, vmess-ws-tls, etc.).
    pub inner_protocol: InnerProtocol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InnerProtocol {
    VlessWsTls,
    VmessWsTls,
    TrojanWsTls,
    VlessGrpcTls,
}

impl Default for NainSafeEndpoint {
    fn default() -> Self {
        Self {
            domain: "your-cdn.arvancloud.ir".into(),
            host_header: "your-cdn.arvancloud.ir".into(),
            ws_path: "/ws".into(),
            fallback_ips: ARVANCLOUD_DOMESTIC_IPS.iter()
                .filter_map(|s| s.parse::<IpAddr>().ok())
                .collect(),
            port: 443,
            inner_protocol: InnerProtocol::VlessWsTls,
        }
    }
}

/// Checks if an IP is within ArvanCloud's known domestic ranges.
pub fn is_arvancloud_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let octets = v4.octets();
            // 185.215.232.0/24
            (octets[0] == 185 && octets[1] == 215 && octets[2] == 232)
            // 185.143.234.0/24
            || (octets[0] == 185 && octets[1] == 143 && octets[2] == 234)
            // 188.114.96.0/20
            || (octets[0] == 188 && octets[1] == 114
                && octets[2] >= 96 && octets[2] <= 111)
        }
        IpAddr::V6(_) => false,
    }
}

/// Generate an Xray/V2Ray/Sing-box compatible outbound config for NAIN mode.
pub fn generate_nain_outbound_config(endpoint: &NainSafeEndpoint) -> serde_json::Value {
    serde_json::json!({
        "tag": "nain-arvancloud",
        "protocol": "vless",
        "settings": {
            "vnext": [{
                "address": endpoint.fallback_ips.first()
                    .map(|ip| ip.to_string())
                    .unwrap_or_else(|| "185.215.232.1".into()),
                "port": endpoint.port,
                "users": [{
                    "id": "YOUR-UUID-HERE",
                    "encryption": "none",
                    "flow": ""
                }]
            }]
        },
        "streamSettings": {
            "network": "ws",
            "security": "tls",
            "tlsSettings": {
                "serverName": endpoint.domain,
                "allowInsecure": false,
                "fingerprint": "chrome"
            },
            "wsSettings": {
                "path": endpoint.ws_path,
                "headers": {
                    "Host": endpoint.host_header
                }
            }
        },
        "comment": "ArvanCloud CDN relay — NAIN-safe. Works when international internet is cut."
    })
}
