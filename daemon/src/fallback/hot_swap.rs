//! HotSwapManager — Atomic Tunnel Replacement Without Disconnection
//!
//! Manages the "swap" moment: builds the new tunnel, then atomically
//! replaces the old one so in-flight packets lose no more than one RTT.

use std::time::Duration;
use tracing::{debug, info, warn};

use crate::isp_detector::protocol_selector::{Protocol, ProtocolConfigHints};
use super::{ActiveTunnel, TunnelError, TunnelStats, BlockSignalSeverity};

/// A stub tunnel used when the real protocol tunnel is being established.
/// Buffers up to N packets so nothing is dropped during the swap window.
struct BufferTunnel {
    buffer: tokio::sync::Mutex<Vec<Vec<u8>>>,
    capacity: usize,
}

impl BufferTunnel {
    fn new(capacity: usize) -> Self {
        Self { buffer: tokio::sync::Mutex::new(Vec::with_capacity(capacity)), capacity }
    }
    async fn drain(&self) -> Vec<Vec<u8>> {
        let mut buf = self.buffer.lock().await;
        std::mem::take(&mut *buf)
    }
}

/// Manager that establishes new tunnels and performs atomic hot-swaps.
pub struct HotSwapManager {
    /// Pre-warmed tunnel cache: protocol → ready tunnel.
    prewarmed: tokio::sync::Mutex<std::collections::HashMap<String, Box<dyn ActiveTunnel>>>,
}

impl HotSwapManager {
    pub fn new() -> Self {
        Self { prewarmed: tokio::sync::Mutex::new(std::collections::HashMap::new()) }
    }

    /// Establish a tunnel for the given protocol.
    /// First checks the pre-warm cache; if hit, returns instantly.
    pub async fn establish_tunnel(
        &self,
        protocol: &Protocol,
        hints: &ProtocolConfigHints,
    ) -> Result<Box<dyn ActiveTunnel>, TunnelError> {
        let key = format!("{:?}", protocol);

        // Check pre-warm cache first (sub-millisecond swap if pre-warmed)
        {
            let mut cache = self.prewarmed.lock().await;
            if let Some(tunnel) = cache.remove(&key) {
                info!("HotSwap: using pre-warmed tunnel for {:?} (instant swap)", protocol);
                return Ok(tunnel);
            }
        }

        debug!("HotSwap: cold-connecting {:?}", protocol);

        // Cold connect — production implementations per protocol:
        match protocol {
            Protocol::VlessReality | Protocol::VlessRealityVision => {
                self.connect_reality(hints).await
            }
            Protocol::ShadowTlsV3 => {
                self.connect_shadow_tls(hints).await
            }
            Protocol::AmneziaWg => {
                self.connect_amnezia_wg(hints).await
            }
            Protocol::Hysteria2 => {
                self.connect_hysteria2(hints).await
            }
            Protocol::NaiveProxy => {
                self.connect_naiveproxy(hints).await
            }
            Protocol::VlessWsTls | Protocol::VmessWsTls | Protocol::TrojanWsTls => {
                self.connect_ws_tls(protocol, hints).await
            }
            Protocol::Psiphon => {
                self.connect_psiphon(hints).await
            }
            Protocol::MeekAzure => {
                self.connect_meek(hints).await
            }
        }
    }

    /// Store a pre-warmed tunnel for instant future use.
    pub async fn store_prewarmed(&self, protocol: Protocol, tunnel: Box<dyn ActiveTunnel>) {
        let key = format!("{:?}", protocol);
        let mut cache = self.prewarmed.lock().await;
        cache.insert(key, tunnel);
        debug!("HotSwap: pre-warmed tunnel stored for {:?}", protocol);
    }

    // ── Per-protocol connection stubs ─────────────────────────────────────
    // Production: each calls the real protocol implementation.
    // Here: return a MockTunnel so the architecture compiles and runs.

    async fn connect_reality(&self, hints: &ProtocolConfigHints)
        -> Result<Box<dyn ActiveTunnel>, TunnelError>
    {
        info!("HotSwap: connecting XTLS Reality (dest={:?}, fp={:?})",
              hints.reality_dest, hints.utls_fingerprint);
        Ok(Box::new(MockTunnel::new(Protocol::VlessReality)))
    }

    async fn connect_shadow_tls(&self, hints: &ProtocolConfigHints)
        -> Result<Box<dyn ActiveTunnel>, TunnelError>
    {
        info!("HotSwap: connecting ShadowTLS v3 (sni={:?})", hints.shadow_tls_sni);
        Ok(Box::new(MockTunnel::new(Protocol::ShadowTlsV3)))
    }

    async fn connect_amnezia_wg(&self, hints: &ProtocolConfigHints)
        -> Result<Box<dyn ActiveTunnel>, TunnelError>
    {
        info!("HotSwap: connecting AmneziaWG (cfg={:?})", hints.amnezia_config);
        Ok(Box::new(MockTunnel::new(Protocol::AmneziaWg)))
    }

    async fn connect_hysteria2(&self, _hints: &ProtocolConfigHints)
        -> Result<Box<dyn ActiveTunnel>, TunnelError>
    {
        info!("HotSwap: connecting Hysteria2");
        Ok(Box::new(MockTunnel::new(Protocol::Hysteria2)))
    }

    async fn connect_naiveproxy(&self, _hints: &ProtocolConfigHints)
        -> Result<Box<dyn ActiveTunnel>, TunnelError>
    {
        info!("HotSwap: connecting NaiveProxy");
        Ok(Box::new(MockTunnel::new(Protocol::NaiveProxy)))
    }

    async fn connect_ws_tls(&self, protocol: &Protocol, hints: &ProtocolConfigHints)
        -> Result<Box<dyn ActiveTunnel>, TunnelError>
    {
        let cdn = hints.cdn_provider.as_deref().unwrap_or("direct");
        info!("HotSwap: connecting {:?} via CDN={}", protocol, cdn);
        Ok(Box::new(MockTunnel::new(protocol.clone())))
    }

    async fn connect_psiphon(&self, _hints: &ProtocolConfigHints)
        -> Result<Box<dyn ActiveTunnel>, TunnelError>
    {
        info!("HotSwap: connecting Psiphon (Iran config)");
        Ok(Box::new(MockTunnel::new(Protocol::Psiphon)))
    }

    async fn connect_meek(&self, _hints: &ProtocolConfigHints)
        -> Result<Box<dyn ActiveTunnel>, TunnelError>
    {
        info!("HotSwap: connecting Meek-Azure");
        Ok(Box::new(MockTunnel::new(Protocol::MeekAzure)))
    }
}

// ── Mock tunnel for structural completeness ─────────────────────────────────

struct MockTunnel {
    protocol: Protocol,
    stats: TunnelStats,
    established: std::time::Instant,
}

impl MockTunnel {
    fn new(protocol: Protocol) -> Self {
        Self {
            protocol,
            stats: TunnelStats::default(),
            established: std::time::Instant::now(),
        }
    }
}

#[async_trait::async_trait]
impl ActiveTunnel for MockTunnel {
    fn protocol(&self) -> Protocol { self.protocol.clone() }

    async fn send(&self, data: &[u8]) -> Result<usize, TunnelError> {
        Ok(data.len())
    }

    async fn recv(&self, buf: &mut [u8]) -> Result<usize, TunnelError> {
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(0)
    }

    async fn keepalive_ping(&self) -> Result<Duration, TunnelError> {
        tokio::time::sleep(Duration::from_millis(30)).await;
        Ok(Duration::from_millis(30))
    }

    async fn is_alive(&self) -> bool { true }

    async fn close(self: Box<Self>) {
        debug!("MockTunnel {:?} closed after {:?}", self.protocol, self.established.elapsed());
    }

    fn stats(&self) -> TunnelStats { self.stats.clone() }
}
