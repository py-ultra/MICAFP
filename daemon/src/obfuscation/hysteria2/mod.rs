//! Hysteria2 Integration — QUIC-Based High-Speed Obfuscated Tunnel
//!
//! Hysteria2 is a high-performance proxy protocol built on QUIC (UDP).
//! It uses a custom congestion control algorithm (BBR + Brutal) and
//! obfuscates QUIC packets to make them look like regular HTTPS traffic.
//!
//! ## Performance vs Other Protocols
//!
//! | Protocol        | Throughput  | Latency | Iran Bypass |
//! |----------------|-------------|---------|-------------|
//! | VLESS Reality  | ~~150 Mbps  | Low     | Excellent   |
//! | Hysteria2      | 200-500 Mbps| Very Low| Good*       |
//! | ShadowTLS v3   | ~~150 Mbps  | Low     | Excellent   |
//! | NaiveProxy     | ~~100 Mbps  | Medium  | Very Good   |
//!
//! *Hysteria2 only works on ISPs that do NOT block QUIC/UDP 443.
//!
//! ## Iranian ISP QUIC Status (verified 2025)
//!
//! | ISP           | QUIC/UDP 443 | Use Hysteria2? |
//! |---------------|-------------|----------------|
//! | Rightel       | ✓ Open      | YES            |
//! | Asiatech      | ✓ Open      | YES            |
//! | Afranet       | ✓ Open      | YES            |
//! | Mobinnet      | ✓ Open      | YES            |
//! | MCI           | ✗ Blocked   | NO             |
//! | Irancell      | ✗ Blocked   | NO             |
//! | Shatel        | ✗ Blocked   | NO             |
//! | ParsOnline    | ✗ Blocked   | NO             |
//! | Mokhaberat    | ✗ Blocked   | NO             |
//!
//! ## Hysteria2 Obfuscation: salamander
//!
//! Hysteria2's built-in "salamander" obfuscation XORs QUIC packets
//! with a PSK-derived keystream, making them look like random UDP
//! traffic rather than QUIC. ISPs cannot block it via QUIC DPI.
//!
//! ## Bandwidth Throttle Bypass
//!
//! The "Brutal" congestion control in Hysteria2 aggressively claims
//! bandwidth even on throttled connections. On ISPs that throttle
//! VPN traffic to 128 kbps, Hysteria2 Brutal can often achieve
//! 5-10x the speed of TCP-based protocols under the same throttling.

/// Hysteria2 client configuration.
#[derive(Debug, Clone)]
pub struct Hysteria2Config {
    /// Server address (IP:port, port is usually 443 or 2053).
    pub server: String,
    /// Authentication password.
    pub auth: String,
    /// Obfuscation type ("salamander" or empty for no obfuscation).
    pub obfs_type: String,
    /// Obfuscation password (for salamander).
    pub obfs_password: String,
    /// Bandwidth hint for Brutal congestion control (Mbps download).
    pub bandwidth_up_mbps: u32,
    /// Bandwidth hint upload (Mbps).
    pub bandwidth_down_mbps: u32,
    /// TLS SNI for the server.
    pub sni: String,
    /// Disable TLS certificate verification (NOT recommended in production).
    pub insecure: bool,
    /// ALPN values to advertise.
    pub alpn: Vec<String>,
    /// SOCKS5 listen address for local proxy.
    pub socks5_listen: String,
    /// HTTP proxy listen address.
    pub http_listen: String,
}

impl Default for Hysteria2Config {
    fn default() -> Self {
        Self {
            server: "your-server.example.com:443".into(),
            auth: "strong-password".into(),
            obfs_type: "salamander".into(),
            obfs_password: "obfs-password".into(),
            bandwidth_up_mbps: 50,
            bandwidth_down_mbps: 200,
            sni: "your-server.example.com".into(),
            insecure: false,
            alpn: vec!["h3".into()],
            socks5_listen: "127.0.0.1:1080".into(),
            http_listen: "127.0.0.1:8080".into(),
        }
    }
}

impl Hysteria2Config {
    /// Generate a Hysteria2 client YAML configuration file.
    pub fn to_yaml(&self) -> String {
        format!(
r#"server: {server}
auth: {auth}

obfs:
  type: {obfs_type}
  salamander:
    password: {obfs_password}

bandwidth:
  up: {up} mbps
  down: {down} mbps

tls:
  sni: {sni}
  insecure: {insecure}
  alpn:
{alpn}

socks5:
  listen: {socks5}

http:
  listen: {http}

# Brutal congestion control — maximises throughput on throttled ISPs
# fastOpen: true       # TCP Fast Open equivalent for QUIC
# lazy: false          # Connect immediately, not on first data
"#,
            server = self.server,
            auth = self.auth,
            obfs_type = self.obfs_type,
            obfs_password = self.obfs_password,
            up = self.bandwidth_up_mbps,
            down = self.bandwidth_down_mbps,
            sni = self.sni,
            insecure = self.insecure,
            alpn = self.alpn.iter().map(|a| format!("    - {}", a)).collect::<Vec<_>>().join("\n"),
            socks5 = self.socks5_listen,
            http = self.http_listen,
        )
    }
}

/// Check if Hysteria2 is likely to work on a given ISP.
pub fn is_recommended_for_isp(isp_id: &str) -> bool {
    matches!(isp_id, "rightel" | "asiatech" | "afranet" | "mobinnet")
}

/// ISP-specific Hysteria2 notes.
pub fn notes_for_isp(isp_id: &str) -> &'static str {
    match isp_id {
        "rightel"   => "QUIC open — Hysteria2 excellent, use salamander obfs",
        "asiatech"  => "QUIC open — Hysteria2 best for high-speed fiber",
        "afranet"   => "QUIC open — Hysteria2 very effective, datacenter network",
        "mobinnet"  => "QUIC open — Hysteria2 good on TD-LTE despite higher latency",
        "mci"       => "QUIC blocked — use Hysteria2 over TCP port 443 (limited support) or switch to Reality",
        "irancell"  => "QUIC blocked by FAVA v4 — do NOT use Hysteria2",
        "shatel"    => "QUIC blocked — switch to Reality or ShadowTLS",
        "pars_online"=> "QUIC blocked + ML DPI — do NOT use Hysteria2",
        "mokhaberat"=> "QUIC blocked — use Reality or AmneziaWG",
        _           => "Test QUIC availability first with: nc -u server_ip 443",
    }
}
