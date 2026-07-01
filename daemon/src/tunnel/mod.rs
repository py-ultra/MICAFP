pub mod amneziawg;
pub mod boringtun_adapter;
pub mod split_tunnel;
pub mod tun_device;
pub mod wireguard;

pub use tun_device::TunDevice;
pub use wireguard::WireGuardTunnel;
pub use amneziawg::AmneziaWGTunnel;
pub use boringtun_adapter::BoringTunAdapter;
pub use split_tunnel::SplitTunnel;

pub type AmneziaWgTunnel  = AmneziaWGTunnel;
pub type BoringtunAdapter = BoringTunAdapter;

use std::fmt;
use std::net::{IpAddr, Ipv4Addr};

/// Tunnel configuration.
#[derive(Debug, Clone)]
pub struct TunnelConfig {
    pub local_ip: Ipv4Addr,
    pub remote_ip: Ipv4Addr,
    pub gateway: Ipv4Addr,
    pub mtu: u16,
}

impl Default for TunnelConfig {
    fn default() -> Self {
        TunnelConfig {
            local_ip: Ipv4Addr::new(10, 0, 0, 1),
            remote_ip: Ipv4Addr::new(10, 0, 0, 2),
            gateway: Ipv4Addr::new(10, 0, 0, 1),
            mtu: 1500,
        }
    }
}

/// Tunnel error type.
#[derive(Debug, Clone)]
pub struct TunnelError(pub String);

impl fmt::Display for TunnelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tunnel error: {}", self.0)
    }
}

impl std::error::Error for TunnelError {}

/// Tunnel state enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TunnelState {
    Idle,
    Connecting,
    Connected,
    Disconnecting,
    Disconnected,
    Error,
}

/// Tunnel statistics.
#[derive(Debug, Clone, Default)]
pub struct TunnelStats {
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

/// Obfuscation mode for tunnel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObfuscationMode {
    None,
    CloudflareWorker,
    CdnGateway,
    IpfsRelay,
    MqttObfuscation,
}
