// ─────────────────────────────────────────────────────────────────────────────
// Transport subsystem — All 27 protocols (13+SlipNet+MoaV merge = 16 projects)
// MICAFP-UnifiedShield-vip-ultra-Quantum-ultra-Quantum v9.0
// New in v9.0: dns_tunnel (DNSTT/NoizDNS/VayDNS), slipstream (QUIC),
//              ssh_ext (TLS/WS/HTTP-CONNECT/payload), naiveproxy_ext
// ─────────────────────────────────────────────────────────────────────────────

// ── SlipNet DNS tunneling transports (v9.0) ──────────────────────────────
pub mod dns_tunnel;
pub mod slipstream;

pub mod cdn_tunnel;
pub mod cdn_worker;
pub mod chinese_cdn;
pub mod cloudflare_worker;
pub mod doh_tunnel;
pub mod domain_fronting;
pub mod doq_tunnel;
pub mod hysteria2;
pub mod icmp_tunnel;
pub mod manager;
pub mod meek;
pub mod mqtt_tunnel;
pub mod mqtt_ws;
pub mod multihop_chain;
pub mod naive_proxy;
pub mod pluggable_transport;
pub mod reality;
pub mod shadow_tls;
pub mod tuic_v5;
pub mod vless;
pub mod webrtc_relay;
pub mod webtransport;

use crate::error::ShieldError;
use async_trait::async_trait;
use std::sync::Arc;
use std::net::SocketAddr;

pub use manager::TransportManager;
pub use multihop_chain::MultiHopChainTransport;

/// Alias — TransportStream is the byte-stream abstraction.
/// Some submodules refer to it as TransportConnection.

/// A connected transport byte-stream handle.
/// Alias of TransportStream for backward compatibility.
pub trait TransportConnection: Send + Sync {
    fn send_bytes(&mut self, data: &[u8]) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>>;
    fn recv_bytes(&mut self) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<Vec<u8>>> + Send + '_>>;
    fn close_conn(&mut self) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>>;
}

/// Generic wrapper for transport connections with any AsyncRead/AsyncWrite stream.
/// Implements TransportConnection trait for streams from all protocols.
pub struct GenericTransportConnection<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + Sync> {
    stream: S,
    sni_domain: String,
    transport_name: String,
}

impl<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + Sync + 'static> GenericTransportConnection<S> {
    /// Create a new generic transport connection.
    pub fn new(stream: S, sni_domain: String, transport_name: String) -> Box<dyn TransportConnection> {
        Box::new(GenericTransportConnection {
            stream,
            sni_domain,
            transport_name,
        })
    }
}

impl<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + Sync> TransportConnection for GenericTransportConnection<S> {
    fn send_bytes(&mut self, data: &[u8]) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>> {
        use tokio::io::AsyncWriteExt;
        Box::pin(async move {
            self.stream.write_all(data).await?;
            Ok(())
        })
    }

    fn recv_bytes(&mut self) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<Vec<u8>>> + Send + '_>> {
        use tokio::io::AsyncReadExt;
        Box::pin(async move {
            let mut buffer = vec![0u8; 65536];
            let n = self.stream.read(&mut buffer).await?;
            buffer.truncate(n);
            Ok(buffer)
        })
    }

    fn close_conn(&mut self) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>> {
        Box::pin(async move { Ok(()) })
    }
}

/// Core transport trait — implemented by all 22 protocols
#[async_trait]
pub trait Transport: Send + Sync {
    /// Human-readable transport name.
    fn name(&self) -> &str;

    /// Priority for UCB1 bandit selection (higher = preferred).
    fn priority(&self) -> u8;

    /// Establish a connection to the target address.
    async fn connect(&self, addr: &SocketAddr) -> Result<Box<dyn TransportConnection>, ShieldError>;

    /// Check if transport is available.
    async fn is_available(&self) -> bool;

    /// Get the last error, if any.
    fn last_error(&self) -> Option<&ShieldError>;

    /// Get current SNI domain.
    fn current_sni_domain(&self) -> &str;

    /// Rotate to next SNI domain or fronting domain.
    async fn rotate_sni_domain(&self) -> Result<String, ShieldError>;

    /// Get current active connections count.
    fn active_connections(&self) -> usize;

    /// Shutdown this transport.
    async fn shutdown(&self) -> Result<(), ShieldError>;
}

// ── Re-exports for submodule convenience (continued) ────────────────────────

/// ISP profile re-exported for transport submodules.
pub use crate::config::isp_profile::IspProfile;

/// Battery state re-exported for transport submodules.
pub use crate::ipc::BatteryState;

/// Statistics for a single transport endpoint.
#[derive(Debug, Clone, Default)]
pub struct EndpointStats {
    pub attempts: u32,
    pub successes: u32,
    pub failures: u32,
    pub avg_latency_ms: f64,
    pub last_attempt_ts: u64,
    pub last_success_ts: u64,
}

impl EndpointStats {
    pub fn success_rate(&self) -> f64 {
        if self.attempts == 0 { 0.5 } else { self.successes as f64 / self.attempts as f64 }
    }
}

/// Transport weight for the load balancer.
#[derive(Debug, Clone)]
pub struct TransportWeight {
    pub name: String,
    pub weight: u32,
    pub current_weight: f64,
    pub effective_weight: f64,
    pub enabled: bool,
}

/// Exponential backoff with full jitter.
pub fn exponential_backoff_with_jitter(attempt: u32, base_ms: u64, max_ms: u64) -> std::time::Duration {
    use rand::Rng;
    let cap = (base_ms * 2u64.pow(attempt)).min(max_ms);
    let jitter = rand::thread_rng().gen_range(0..=cap);
    std::time::Duration::from_millis(jitter)
}

/// Resolve a domain name to a SocketAddr with default port.
pub async fn resolve_domain(domain: &str, default_port: u16) -> Result<SocketAddr, ShieldError> {
    use tokio::net::lookup_host;

    let addr_str = format!("{}:{}", domain, default_port);
    let mut addrs = lookup_host(&addr_str).await
        .map_err(|e| ShieldError::from_code(crate::error::ErrorCode::TransportError, &format!("DNS resolution failed for {}: {}", domain, e)))?;

    addrs.next()
        .ok_or_else(|| ShieldError::from_code(crate::error::ErrorCode::TransportError, &format!("No addresses found for {}", domain)))
}

