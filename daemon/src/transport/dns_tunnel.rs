//! DNS Tunnel transport — SlipNet DNSTT / NoizDNS / VayDNS integration
//!
//! Bridges the Go-based DNS tunneling protocols (imported from SlipNet) into the
//! Rust daemon via the libslipnet_bridge.a CGo c-archive.
//!
//! Protocol variants:
//!   DnsTunnelKind::DNSTT      — stable KCP+Noise DNS tunneling (dnstt)
//!   DnsTunnelKind::NoizDNS    — DPI-resistant DNS tunneling with stealth mode
//!   DnsTunnelKind::VayDNS     — optimized wire-format DNS tunneling with configurable
//!                               record types, QNAME lengths, and rate limiting

use std::net::SocketAddr;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::net::{TcpStream, UdpSocket};
use crate::error::ShieldError;
use super::{Transport, TransportConnection, GenericTransportConnection};

/// Which DNS-tunnel protocol variant to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DnsTunnelKind {
    Dnstt,
    NoizDns,
    VayDns,
}

impl std::fmt::Display for DnsTunnelKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dnstt   => write!(f, "dnstt"),
            Self::NoizDns => write!(f, "noizdns"),
            Self::VayDns  => write!(f, "vaydns"),
        }
    }
}

/// Configuration for a DNS tunnel endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsTunnelConfig {
    pub kind:         DnsTunnelKind,
    pub domain:       String,
    pub resolver:     String,       // e.g. "8.8.8.8:53"
    pub server_key:   String,       // hex-encoded public key for DNSTT/NoizDNS
    pub listen_host:  String,
    pub listen_port:  u16,
    // VayDNS-specific
    pub record_type:  Option<String>, // "NS", "TXT", etc.
    pub rps:          Option<f64>,
    pub max_labels:   Option<u32>,
    // DPI-resistance / stealth
    pub stealth_mode: bool,
}

impl Default for DnsTunnelConfig {
    fn default() -> Self {
        Self {
            kind:         DnsTunnelKind::Dnstt,
            domain:       String::new(),
            resolver:     "8.8.8.8:53".into(),
            server_key:   String::new(),
            listen_host:  "127.0.0.1".into(),
            listen_port:  1080,
            record_type:  None,
            rps:          None,
            max_labels:   None,
            stealth_mode: false,
        }
    }
}

/// DNS Tunnel Transport — wraps the Go bridge's SOCKS5 endpoint.
pub struct DnsTunnelTransport {
    config:          DnsTunnelConfig,
    bridge_tunnel_id: std::sync::atomic::AtomicI32,
}

impl DnsTunnelTransport {
    pub fn new(config: DnsTunnelConfig) -> Arc<Self> {
        Arc::new(Self {
            config,
            bridge_tunnel_id: std::sync::atomic::AtomicI32::new(-1),
        })
    }

    /// Serialize config to JSON and hand to the Go bridge.
    pub async fn start_bridge(&self) -> Result<(), ShieldError> {
        let cfg = serde_json::json!({
            "listen_port":  self.config.listen_port,
            "listen_host":  self.config.listen_host,
            "tunnel_type":  self.config.kind.to_string(),
            "domain":       self.config.domain,
            "resolver":     self.config.resolver,
            "server_key":   self.config.server_key,
            "stealth_mode": self.config.stealth_mode,
            "record_type":  self.config.record_type,
            "rps":          self.config.rps,
            "qname_max_labels": self.config.max_labels,
        });
        let json = serde_json::to_string(&cfg).map_err(|e| ShieldError::Config(e.to_string()))?;
        tracing::info!(kind=%self.config.kind, domain=%self.config.domain, "Starting DNS tunnel via Go bridge");
        // In production: call SlipNetStartDNSTunnel via libslipnet_bridge.a via FFI.
        // The CGo symbol is linked at compile time when building with
        //   cargo build --features go-bridge
        // For feature-gated stub:
        #[cfg(feature = "go-bridge")]
        {
            extern "C" {
                fn SlipNetStartDNSTunnel(config_json: *const std::os::raw::c_char) -> std::os::raw::c_int;
            }
            let cstr = std::ffi::CString::new(json).map_err(|e| ShieldError::Config(e.to_string()))?;
            let id = unsafe { SlipNetStartDNSTunnel(cstr.as_ptr()) };
            if id < 0 {
                return Err(ShieldError::Transport("DNS tunnel bridge failed to start".into()));
            }
            self.bridge_tunnel_id.store(id, std::sync::atomic::Ordering::SeqCst);
        }
        #[cfg(not(feature = "go-bridge"))]
        {
            let _ = json;
            tracing::debug!("go-bridge feature not enabled — DNS tunnel running in stub mode");
        }
        Ok(())
    }

    /// SOCKS5 address where the Go bridge is listening.
    fn socks5_addr(&self) -> String {
        format!("{}:{}", self.config.listen_host, self.config.listen_port)
    }
}

#[async_trait]
impl Transport for DnsTunnelTransport {
    fn name(&self) -> &str {
        match self.config.kind {
            DnsTunnelKind::Dnstt   => "dnstt",
            DnsTunnelKind::NoizDns => "noizdns",
            DnsTunnelKind::VayDns  => "vaydns",
        }
    }

    fn priority(&self) -> u8 { 70 }

    async fn connect(&self, addr: &SocketAddr) -> Result<Box<dyn TransportConnection>, ShieldError> {
        // Connect via SOCKS5 proxy that the Go bridge exposes
        use tokio::io::AsyncWriteExt;
        let socks5 = self.socks5_addr();
        let mut conn = TcpStream::connect(&socks5).await
            .map_err(|e| ShieldError::Transport(format!("SOCKS5 connect to DNS bridge at {socks5}: {e}")))?;

        // SOCKS5 handshake (no-auth)
        conn.write_all(&[0x05, 0x01, 0x00]).await
            .map_err(|e| ShieldError::Transport(format!("SOCKS5 greeting: {e}")))?;
        let mut buf = [0u8; 2];
        use tokio::io::AsyncReadExt;
        conn.read_exact(&mut buf).await
            .map_err(|e| ShieldError::Transport(format!("SOCKS5 auth response: {e}")))?;

        // SOCKS5 CONNECT request
        let mut req = vec![0x05, 0x01, 0x00, 0x01];
        match addr {
            SocketAddr::V4(v4) => {
                req.extend_from_slice(&v4.ip().octets());
                req.extend_from_slice(&v4.port().to_be_bytes());
            },
            SocketAddr::V6(v6) => {
                req[3] = 0x04;
                req.extend_from_slice(&v6.ip().octets());
                req.extend_from_slice(&v6.port().to_be_bytes());
            },
        }
        conn.write_all(&req).await
            .map_err(|e| ShieldError::Transport(format!("SOCKS5 CONNECT: {e}")))?;
        let mut resp = [0u8; 10];
        conn.read_exact(&mut resp).await
            .map_err(|e| ShieldError::Transport(format!("SOCKS5 response: {e}")))?;
        if resp[1] != 0x00 {
            return Err(ShieldError::Transport(format!("SOCKS5 error code: 0x{:02x}", resp[1])));
        }

        tracing::debug!(kind=%self.name(), target=%addr, "DNS tunnel connection established");
        Ok(GenericTransportConnection::new(conn, addr.to_string(), self.name().to_string()))
    }

    async fn is_available(&self) -> bool {
        TcpStream::connect(self.socks5_addr()).await.is_ok()
    }

    fn last_error(&self) -> Option<&ShieldError> { None }
    fn current_sni_domain(&self) -> &str { &self.config.domain }

    async fn rotate_sni_domain(&self) -> Result<String, ShieldError> {
        Ok(self.config.domain.clone())
    }

    fn active_connections(&self) -> usize { 0 }

    async fn shutdown(&self) -> Result<(), ShieldError> {
        #[cfg(feature = "go-bridge")]
        {
            let id = self.bridge_tunnel_id.load(std::sync::atomic::Ordering::SeqCst);
            if id >= 0 {
                extern "C" { fn SlipNetStopTunnel(id: std::os::raw::c_int); }
                unsafe { SlipNetStopTunnel(id) };
            }
        }
        Ok(())
    }
}
