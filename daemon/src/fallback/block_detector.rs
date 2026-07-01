//! Block Signal Types and Detection Events
//!
//! Defines all blocking signals that the HealthMonitor can emit.
//! Each signal carries a timestamp, severity, and evidence payload
//! so the FallbackEngine can make informed decisions about whether
//! to switch protocols immediately or wait through a grace period.

use std::time::Instant;
use super::BlockSignalSeverity;

/// A detected blocking signal from the HealthMonitor.
#[derive(Debug, Clone)]
pub struct BlockSignal {
    /// Which detection method fired.
    pub signal_type: BlockSignalType,
    /// Severity determines grace period or immediate action.
    pub severity: BlockSignalSeverity,
    /// When the signal was emitted.
    pub timestamp: Instant,
    /// Evidence string for logging/telemetry.
    pub evidence: String,
}

/// All possible blocking signal types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockSignalType {
    /// Bytes/sec dropped below threshold for N consecutive seconds.
    /// Evidence: "throughput=42bps threshold=1024bps window=5s"
    ThroughputCollapse,

    /// Keepalive ping received no pong within timeout.
    /// Evidence: "timeout=8s last_successful_rtt=45ms"
    KeepaliveTimeout,

    /// Multiple TCP RST packets received in short window.
    /// Evidence: "rst_count=4 window=10s src_ips=[1.2.3.4]"
    RstStorm,

    /// TLS handshake failure rate spiked.
    /// Evidence: "failures_per_min=12.3 threshold=5.0"
    TlsFailureSpike,

    /// eBPF detected active probing of our server IP by DPI.
    /// Evidence: "probe_src=185.51.200.2 probe_count=3"
    ActiveProbeDetected,

    /// Tunnel throughput is asymmetrically throttled (upload throttled, DL ok).
    /// Evidence: "up_bps=512 down_bps=45000"
    AsymmetricThrottle,

    /// Received ICMP Unreachable for the tunnel destination port.
    /// Evidence: "icmp_type=3 icmp_code=3 from=gateway"
    IcmpUnreachable,

    /// DNS resolution of tunnel server failed or returned poison IP.
    /// Evidence: "resolved=10.10.34.35 poison=true"
    DnsPoisonedServer,
}

impl BlockSignalType {
    /// Default severity for each signal type.
    pub fn default_severity(&self) -> BlockSignalSeverity {
        match self {
            Self::ThroughputCollapse  => BlockSignalSeverity::High,
            Self::KeepaliveTimeout    => BlockSignalSeverity::High,
            Self::RstStorm            => BlockSignalSeverity::Critical,
            Self::TlsFailureSpike     => BlockSignalSeverity::Medium,
            Self::ActiveProbeDetected => BlockSignalSeverity::High,
            Self::AsymmetricThrottle  => BlockSignalSeverity::Medium,
            Self::IcmpUnreachable     => BlockSignalSeverity::Critical,
            Self::DnsPoisonedServer   => BlockSignalSeverity::Critical,
        }
    }

    /// Human-readable description for logging.
    pub fn description(&self) -> &'static str {
        match self {
            Self::ThroughputCollapse  => "Throughput collapsed below threshold",
            Self::KeepaliveTimeout    => "Keepalive ping timed out",
            Self::RstStorm            => "TCP RST storm detected (DPI reset attack)",
            Self::TlsFailureSpike     => "TLS handshake failure rate spiked",
            Self::ActiveProbeDetected => "DPI active probing detected on server IP",
            Self::AsymmetricThrottle  => "Asymmetric throttling detected",
            Self::IcmpUnreachable     => "ICMP port unreachable received",
            Self::DnsPoisonedServer   => "Server domain returned poisoned DNS response",
        }
    }
}

impl BlockSignal {
    pub fn new(signal_type: BlockSignalType, evidence: impl Into<String>) -> Self {
        let severity = signal_type.default_severity();
        Self {
            signal_type,
            severity,
            timestamp: Instant::now(),
            evidence: evidence.into(),
        }
    }

    pub fn with_severity(mut self, severity: BlockSignalSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn is_critical(&self) -> bool {
        self.severity == BlockSignalSeverity::Critical
    }

    pub fn age(&self) -> std::time::Duration {
        self.timestamp.elapsed()
    }
}
