//! Port Scanner for detecting blocked ports in Iranian networks
//!
//! Identifies which ports are blocked by the ISP and which
//! transport methods are viable.

use anyhow::Result;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tracing::{debug, info, warn};

/// Port scan status
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PortStatus {
    /// Port is open and reachable
    Open,
    /// Port is blocked/filtered
    Blocked,
    /// Port status is unknown (timeout or error)
    Unknown,
}

/// Port scan result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PortScanResult {
    /// Port number
    pub port: u16,
    /// Port status
    pub status: PortStatus,
    /// Response time in milliseconds (if open)
    pub latency_ms: Option<u64>,
}

/// Common ports to scan for VPN traffic
pub const VPN_RELATED_PORTS: &[u16] = &[
    443,   // HTTPS — primary transport port
    80,    // HTTP — fallback
    53,    // DNS
    123,   // NTP
    853,   // DNS-over-TLS
    8443,  // Alternative HTTPS
    1080,  // SOCKS proxy
    1194,  // OpenVPN (usually blocked)
    51820, // WireGuard (usually blocked)
    8080,  // HTTP proxy
    8888,  // Alternative proxy
    9090,  // WebSocket proxy
];

/// Port Scanner
pub struct PortScanner {
    /// Target host for port scanning
    target_host: String,
    /// Timeout per connection attempt
    timeout: Duration,
}

impl PortScanner {
    /// Create a new port scanner
    pub fn new(target_host: &str) -> Self {
        Self {
            target_host: target_host.to_string(),
            timeout: Duration::from_secs(5),
        }
    }

    /// Scan all VPN-related ports
    pub async fn scan_vpn_ports(&self) -> Result<Vec<PortScanResult>> {
        info!("Scanning VPN-related ports on {}", self.target_host);
        let mut results = Vec::new();

        for &port in VPN_RELATED_PORTS {
            let result = self.scan_port(port).await;
            results.push(result);
        }

        let open = results.iter().filter(|r| r.status == PortStatus::Open).count();
        let blocked = results.iter().filter(|r| r.status == PortStatus::Blocked).count();
        info!(
            "Port scan complete: {} open, {} blocked, {} unknown",
            open,
            blocked,
            results.len() - open - blocked
        );

        Ok(results)
    }

    /// Scan a single port
    pub async fn scan_port(&self, port: u16) -> PortScanResult {
        let addr = format!("{}:{}", self.target_host, port);
        let start = Instant::now();

        match tokio::time::timeout(self.timeout, TcpStream::connect(&addr)).await {
            Ok(Ok(_stream)) => {
                let latency = start.elapsed().as_millis() as u64;
                debug!("Port {} OPEN ({}ms)", port, latency);
                PortScanResult {
                    port,
                    status: PortStatus::Open,
                    latency_ms: Some(latency),
                }
            }
            Ok(Err(e)) if e.kind() == std::io::ErrorKind::ConnectionRefused => {
                debug!("Port {} REFUSED", port);
                PortScanResult {
                    port,
                    status: PortStatus::Blocked,
                    latency_ms: Some(start.elapsed().as_millis() as u64),
                }
            }
            Ok(Err(_)) => {
                debug!("Port {} ERROR", port);
                PortScanResult {
                    port,
                    status: PortStatus::Unknown,
                    latency_ms: None,
                }
            }
            Err(_) => {
                debug!("Port {} TIMEOUT", port);
                PortScanResult {
                    port,
                    status: PortStatus::Blocked,
                    latency_ms: None,
                }
            }
        }
    }

    /// Scan a custom range of ports
    pub async fn scan_range(&self, start_port: u16, end_port: u16) -> Result<Vec<PortScanResult>> {
        let mut results = Vec::new();
        for port in start_port..=end_port {
            results.push(self.scan_port(port).await);
        }
        Ok(results)
    }

    /// Get recommended transport ports based on scan results
    pub fn recommend_transport_ports(results: &[PortScanResult]) -> Vec<u16> {
        results
            .iter()
            .filter(|r| r.status == PortStatus::Open)
            .map(|r| r.port)
            .collect()
    }

    /// Check if port 443 is available (required for most transports)
    pub fn is_https_available(results: &[PortScanResult]) -> bool {
        results
            .iter()
            .any(|r| r.port == 443 && r.status == PortStatus::Open)
    }
}
