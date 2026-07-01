//! Slipstream transport — SlipNet QUIC-based high-performance tunnel
//!
//! Slipstream uses QUIC (via quinn) to establish a high-throughput,
//! low-latency tunnel with optional SSH chaining for extra encryption.
//! It complements the existing hysteria2 and TUIC v5 transports with
//! a different QUIC implementation focused on anti-censorship.

use std::net::SocketAddr;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::ShieldError;
use super::{Transport, TransportConnection, GenericTransportConnection};

/// Slipstream tunnel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlipstreamConfig {
    pub server_addr:      String,
    pub server_port:      u16,
    pub listen_host:      String,
    pub listen_port:      u16,
    pub tls_sni:          String,
    pub insecure_skip_verify: bool,
    // SSH chaining (optional)
    pub ssh_enabled:      bool,
    pub ssh_host:         String,
    pub ssh_port:         u16,
    pub ssh_user:         String,
    pub ssh_key_pem:      String,
}

impl Default for SlipstreamConfig {
    fn default() -> Self {
        Self {
            server_addr:      String::new(),
            server_port:      443,
            listen_host:      "127.0.0.1".into(),
            listen_port:      1080,
            tls_sni:          String::new(),
            insecure_skip_verify: false,
            ssh_enabled:      false,
            ssh_host:         String::new(),
            ssh_port:         22,
            ssh_user:         String::new(),
            ssh_key_pem:      String::new(),
        }
    }
}

/// Slipstream QUIC Transport.
pub struct SlipstreamTransport {
    config: SlipstreamConfig,
    bridge_id: std::sync::atomic::AtomicI32,
}

impl SlipstreamTransport {
    pub fn new(config: SlipstreamConfig) -> Arc<Self> {
        Arc::new(Self {
            config,
            bridge_id: std::sync::atomic::AtomicI32::new(-1),
        })
    }

    pub async fn start_bridge(&self) -> Result<(), ShieldError> {
        let cfg = serde_json::json!({
            "listen_port":   self.config.listen_port,
            "listen_host":   self.config.listen_host,
            "tunnel_type":   "slipstream",
            "server_addr":   format!("{}:{}", self.config.server_addr, self.config.server_port),
            "tls_sni":       self.config.tls_sni,
            "insecure":      self.config.insecure_skip_verify,
            "ssh_enabled":   self.config.ssh_enabled,
            "ssh_host":      self.config.ssh_host,
            "ssh_port":      self.config.ssh_port,
            "ssh_user":      self.config.ssh_user,
        });
        let json = serde_json::to_string(&cfg).map_err(|e| ShieldError::Config(e.to_string()))?;
        tracing::info!(server=%self.config.server_addr, "Starting Slipstream QUIC tunnel");

        #[cfg(feature = "go-bridge")]
        {
            extern "C" {
                fn SlipNetStartSlipstream(config_json: *const std::os::raw::c_char) -> std::os::raw::c_int;
            }
            let cstr = std::ffi::CString::new(json).map_err(|e| ShieldError::Config(e.to_string()))?;
            let id = unsafe { SlipNetStartSlipstream(cstr.as_ptr()) };
            if id < 0 {
                return Err(ShieldError::Transport("Slipstream bridge failed to start".into()));
            }
            self.bridge_id.store(id, std::sync::atomic::Ordering::SeqCst);
        }
        #[cfg(not(feature = "go-bridge"))]
        { let _ = json; }
        Ok(())
    }

    fn socks5_addr(&self) -> String {
        format!("{}:{}", self.config.listen_host, self.config.listen_port)
    }
}

#[async_trait]
impl Transport for SlipstreamTransport {
    fn name(&self) -> &str { "slipstream" }
    fn priority(&self) -> u8 { 80 }

    async fn connect(&self, addr: &SocketAddr) -> Result<Box<dyn TransportConnection>, ShieldError> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpStream;

        let socks5 = self.socks5_addr();
        let mut conn = TcpStream::connect(&socks5).await
            .map_err(|e| ShieldError::Transport(format!("SOCKS5 connect to Slipstream bridge at {socks5}: {e}")))?;

        // SOCKS5 no-auth handshake
        conn.write_all(&[0x05, 0x01, 0x00]).await?;
        let mut buf = [0u8; 2];
        conn.read_exact(&mut buf).await?;

        // CONNECT request
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
        conn.write_all(&req).await?;
        let mut resp = [0u8; 10];
        conn.read_exact(&mut resp).await?;
        if resp[1] != 0x00 {
            return Err(ShieldError::Transport(format!("SOCKS5 CONNECT failed: 0x{:02x}", resp[1])));
        }
        Ok(GenericTransportConnection::new(conn, addr.to_string(), "slipstream".into()))
    }

    async fn is_available(&self) -> bool {
        tokio::net::TcpStream::connect(self.socks5_addr()).await.is_ok()
    }

    fn last_error(&self) -> Option<&ShieldError> { None }
    fn current_sni_domain(&self) -> &str { &self.config.tls_sni }

    async fn rotate_sni_domain(&self) -> Result<String, ShieldError> {
        Ok(self.config.tls_sni.clone())
    }

    fn active_connections(&self) -> usize { 0 }

    async fn shutdown(&self) -> Result<(), ShieldError> {
        #[cfg(feature = "go-bridge")]
        {
            let id = self.bridge_id.load(std::sync::atomic::Ordering::SeqCst);
            if id >= 0 {
                extern "C" { fn SlipNetStopTunnel(id: std::os::raw::c_int); }
                unsafe { SlipNetStopTunnel(id) };
            }
        }
        Ok(())
    }
}
