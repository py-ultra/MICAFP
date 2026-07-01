//! Network Assessor — comprehensive network capability assessment for Iran
//!
//! Combines DPI scanning, port scanning, and DNS scanning results
//! to provide a complete assessment of the current network environment
//! and recommend the best transport strategy.

use anyhow::Result;
use std::time::Duration;
use tracing::{info, warn};

use super::dpi_scanner::{DpiScanResult, DpiScanner, FavaVersion};
use super::dns_scanner::{DnsScanResult, DnsScanner};
use super::port_scanner::{PortScanResult, PortScanner, PortStatus};

/// Overall network assessment
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkAssessment {
    /// DPI scan results
    pub dpi_result: DpiScanResult,
    /// DNS scan results
    pub dns_result: DnsScanResult,
    /// Port scan results
    pub port_results: Vec<PortScanResult>,
    /// Whether the network is heavily censored
    pub heavily_censored: bool,
    /// Recommended transport priority list
    pub recommended_transports: Vec<String>,
    /// Whether TLS fragmentation is recommended
    pub tls_fragmentation_recommended: bool,
    /// Whether domain fronting is required
    pub domain_fronting_required: bool,
    /// Whether covert channels are needed
    pub covert_channels_needed: bool,
    /// Assessment timestamp
    pub timestamp: String,
    /// Overall risk level (0-100, higher = more censored)
    pub censorship_risk_score: u8,
}

/// Network Assessor
pub struct NetworkAssessor {
    /// Target host for port scanning
    probe_host: String,
}

impl NetworkAssessor {
    /// Create a new network assessor
    pub fn new() -> Self {
        Self {
            probe_host: "8.8.8.8".to_string(),
        }
    }

    /// Run a full network assessment
    pub async fn assess(&self) -> Result<NetworkAssessment> {
        info!("Starting comprehensive network assessment for Iranian censorship...");

        // Run all scans in parallel for speed
        let (dpi_result, dns_result, port_results) = tokio::join!(
            DpiScanner::new().scan(),
            DnsScanner::new().scan(),
            PortScanner::new(&self.probe_host).scan_vpn_ports(),
        );

        let dpi_result = dpi_result?;
        let dns_result = dns_result?;
        let port_results = port_results?;

        // Calculate censorship risk score
        let risk_score = self.calculate_risk_score(&dpi_result, &dns_result, &port_results);

        // Determine recommendations
        let heavily_censored = risk_score > 70;
        let tls_fragmentation_recommended = dpi_result.sni_filtering;
        let domain_fronting_required = dpi_result.tls_fingerprinting;
        let covert_channels_needed = dpi_result.fava_version == FavaVersion::V3
            || (dns_result.injection_detected && port_results.iter().filter(|p| p.status == PortStatus::Open).count() < 3);

        let recommended_transports = self.recommend_transports(&dpi_result, &dns_result, &port_results);

        let assessment = NetworkAssessment {
            dpi_result,
            dns_result,
            port_results,
            heavily_censored,
            recommended_transports,
            tls_fragmentation_recommended,
            domain_fronting_required,
            covert_channels_needed,
            timestamp: chrono::Utc::now().to_rfc3339(),
            censorship_risk_score: risk_score,
        };

        info!(
            "Assessment complete: risk_score={}, heavily_censored={}, transports={:?}",
            assessment.censorship_risk_score,
            assessment.heavily_censored,
            assessment.recommended_transports
        );

        Ok(assessment)
    }

    /// Calculate censorship risk score (0-100)
    fn calculate_risk_score(
        &self,
        dpi: &DpiScanResult,
        dns: &DnsScanResult,
        ports: &[PortScanResult],
    ) -> u8 {
        let mut score: u8 = 0;

        // DPI factors (0-40 points)
        match dpi.fava_version {
            FavaVersion::None => score += 0,
            FavaVersion::V1 => score += 15,
            FavaVersion::V2 => score += 25,
            FavaVersion::V3 => score += 40,
            FavaVersion::Unknown => score += 20,
        }

        // DNS factors (0-30 points)
        if dns.injection_detected {
            score += 15;
        }
        if dns.doh_required {
            score += 10;
        }
        if dns.poisoned_domains.len() > 3 {
            score += 5;
        }

        // Port factors (0-30 points)
        let open_ports = ports.iter().filter(|p| p.status == PortStatus::Open).count();
        let blocked_ports = ports.iter().filter(|p| p.status == PortStatus::Blocked).count();

        if blocked_ports > 5 {
            score += 20;
        } else if blocked_ports > 2 {
            score += 10;
        }

        if open_ports < 3 {
            score += 10;
        }

        score.min(100)
    }

    /// Recommend transport protocols based on scan results
    fn recommend_transports(
        &self,
        dpi: &DpiScanResult,
        dns: &DnsScanResult,
        ports: &[PortScanResult],
    ) -> Vec<String> {
        let mut transports = Vec::new();
        let port_443_open = ports.iter().any(|p| p.port == 443 && p.status == PortStatus::Open);

        // Always recommend Arvan CDN first (Iranian CDN, never blocked)
        transports.push("arvan-cdn".to_string());

        if dpi.sni_filtering {
            // SNI filtering detected — use Shadow TLS v3 and XTLS-Reality
            transports.push("shadow-tls-v3".to_string());
            transports.push("xtls-reality".to_string());
        }

        if port_443_open {
            transports.push("hysteria2".to_string());
            transports.push("naiveproxy".to_string());
        }

        if dpi.tls_fingerprinting {
            // TLS fingerprinting detected — need more sophisticated evasion
            transports.push("xtls-reality".to_string());
        }

        // Chinese CDN workers (not blocked in Iran)
        transports.push("alibaba-cdn".to_string());
        transports.push("bytedance-cdn".to_string());
        transports.push("tencent-cdn".to_string());

        // QUIC-based transports
        transports.push("tuic-v5".to_string());
        transports.push("webtransport".to_string());

        // Covert channels for extreme censorship
        if dpi.fava_version == FavaVersion::V3 {
            transports.push("doq-tunnel".to_string());
            transports.push("mqtt-ws".to_string());
            transports.push("ntp-covert".to_string());
            transports.push("icmp-tunnel".to_string());
        }

        // P2P overlays as last resort
        transports.push("yggdrasil".to_string());
        transports.push("i2p-overlay".to_string());

        // Deduplicate while preserving order
        let mut seen = std::collections::HashSet::new();
        transports.retain(|t| seen.insert(t.clone()));

        transports
    }
}
