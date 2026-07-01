//! Automatic ISP Detection Engine
//!
//! Detects the user's ISP using multiple methods in order of reliability:
//!
//!   1. ASN lookup via local database (fastest, ~1ms)
//!   2. IP range matching from isp-profiles.json
//!   3. DNS suffix probing (check reverse DNS of gateway)
//!   4. Active QUIC probe (determines if QUIC/UDP is available)
//!   5. DNS poison test (try resolving known blocked domains)
//!   6. Latency fingerprinting (matches known ISP latency profiles)
//!
//! The result feeds into the protocol selector which picks the optimal
//! obfuscation strategy for the detected ISP.

pub mod asn_lookup;
pub mod poison_tester;
pub mod quic_probe;
pub mod protocol_selector;

use std::net::IpAddr;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Detected ISP information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedIsp {
    /// ISP profile ID (matches isp-profiles.json).
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Detection method that succeeded.
    pub detection_method: DetectionMethod,
    /// Confidence 0.0-1.0.
    pub confidence: f32,
    /// Whether QUIC/UDP is available on this connection.
    pub quic_available: Option<bool>,
    /// Whether DNS is poisoned on this connection.
    pub dns_poisoned: Option<bool>,
    /// Measured latency to a domestic probe target.
    pub domestic_latency_ms: Option<u32>,
    /// Measured latency to an international probe target.
    pub international_latency_ms: Option<u32>,
    /// Whether NAIN (national intranet) mode is currently active.
    pub nain_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DetectionMethod {
    AsnLookup,
    IpRange,
    DnsSuffix,
    LatencyFingerprint,
    ManualOverride,
    Unknown,
}

/// ISP detection engine.
pub struct IspDetector {
    /// Embedded ISP profile data (from isp-profiles.json).
    profiles: serde_json::Value,
}

impl IspDetector {
    pub fn new(profiles_json: serde_json::Value) -> Self {
        Self { profiles }
    }

    /// Run full ISP detection pipeline.
    pub async fn detect(&self) -> DetectedIsp {
        let start = Instant::now();

        // 1. Get public IP and local gateway IP
        let public_ip = self.get_public_ip().await;
        debug!("Public IP: {:?}", public_ip);

        // 2. Try ASN lookup
        if let Some(ip) = &public_ip {
            if let Some(isp) = self.lookup_by_asn(ip).await {
                info!("ISP detected via ASN: {} (confidence: {:.0}%)",
                      isp.id, isp.confidence * 100.0);
                return self.enrich(isp).await;
            }
        }

        // 3. Try IP range matching
        if let Some(ip) = &public_ip {
            if let Some(isp) = self.lookup_by_ip_range(ip) {
                info!("ISP detected via IP range: {}", isp.id);
                return self.enrich(isp).await;
            }
        }

        // 4. DNS suffix probe
        if let Some(isp) = self.probe_dns_suffix().await {
            info!("ISP detected via DNS suffix: {}", isp.id);
            return self.enrich(isp).await;
        }

        // 5. Latency fingerprinting
        if let Some(isp) = self.fingerprint_by_latency().await {
            info!("ISP detected via latency fingerprint: {} (confidence: {:.0}%)",
                  isp.id, isp.confidence * 100.0);
            return self.enrich(isp).await;
        }

        warn!("ISP detection failed — using default profile");
        DetectedIsp {
            id: "unknown".into(),
            name: "Unknown ISP".into(),
            detection_method: DetectionMethod::Unknown,
            confidence: 0.0,
            quic_available: None,
            dns_poisoned: None,
            domestic_latency_ms: None,
            international_latency_ms: None,
            nain_active: false,
        }
    }

    /// Enrich detected ISP with QUIC probe, DNS poison test, NAIN check.
    async fn enrich(&self, mut isp: DetectedIsp) -> DetectedIsp {
        // Run QUIC probe, DNS poison test, and NAIN check concurrently
        let (quic, poisoned, nain) = tokio::join!(
            quic_probe::test_quic_availability(),
            poison_tester::test_dns_poison(),
            self.test_nain_active(),
        );
        isp.quic_available = Some(quic);
        isp.dns_poisoned = Some(poisoned);
        isp.nain_active = nain;
        isp
    }

    async fn get_public_ip(&self) -> Option<IpAddr> {
        // Production: HTTP GET to ipv4.icanhazip.com or api.ipify.org
        // Using domestic fallback: api.ip.sb or similar that works in Iran
        None
    }

    async fn lookup_by_asn(&self, ip: &IpAddr) -> Option<DetectedIsp> {
        // Production: use maxmind GeoLite2-ASN.mmdb or ipinfo.io API
        None
    }

    fn lookup_by_ip_range(&self, ip: &IpAddr) -> Option<DetectedIsp> {
        let profiles = self.profiles["profiles"].as_array()?;
        for profile in profiles {
            let ranges = profile["ip_ranges"].as_array()?;
            for range in ranges {
                let cidr = range.as_str()?;
                // Production: use ipnetwork crate to check containment
                // if ipnetwork::IpNetwork::from_str(cidr)?.contains(*ip) { ... }
            }
        }
        None
    }

    async fn probe_dns_suffix(&self) -> Option<DetectedIsp> { None }
    async fn fingerprint_by_latency(&self) -> Option<DetectedIsp> { None }
    async fn test_nain_active(&self) -> bool {
        // Probe 8.8.8.8:53 with 2s timeout — if unreachable, NAIN may be active
        false
    }
}
