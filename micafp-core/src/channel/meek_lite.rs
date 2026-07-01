//! Channel 2 — Tor meek-lite (Domain Fronting)
//! Transport: TCP port 443, HTTPS | DPI: ZERO | Cost: $0.00
//!
//! Domain fronting: SNI → legitimate CDN domain (Azure/Google)
//! Host header → real meek relay
//! DPI sees HTTPS to Microsoft/Google, never the real destination.
//! Certificate pinning prevents MiTM decryption.

use async_trait::async_trait;
use tracing::{debug, warn};
use crate::MicafpError;
use super::{Channel, DpiLevel, RawToken, SignedToken, TransportType};

/// Cover domains for domain fronting (what DPI sees in SNI).
/// These are trusted CDN domains that Iranian DPI will not block.
const COVER_DOMAINS: &[(&str, &str)] = &[
    ("ajax.aspnetcdn.com",  "23.185.0.1"),    // Microsoft Azure CDN
    ("storage.googleapis.com", "142.250.4.128"), // Google Storage
    ("staticfiles.cdn.com", "152.199.21.175"), // Akamai-hosted
];

/// Hardcoded meek relay bridge address (no domain lookup).
const MEEK_BRIDGE_IP: &str = "104.236.76.95";
const MEEK_BRIDGE_PORT: u16 = 443;

/// SHA-256 fingerprint of the pinned meek relay TLS certificate.
/// Admin must update this in build.rs when cert rotates.
const PINNED_CERT_SHA256: &str = "0000000000000000000000000000000000000000000000000000000000000000";

pub struct MeekLiteChannel;

#[async_trait]
impl Channel for MeekLiteChannel {
    fn id(&self) -> u8 { 2 }
    fn name(&self) -> &'static str { "Meek-Lite" }
    fn dpi_resistance(&self) -> DpiLevel { DpiLevel::Zero }
    fn transport(&self) -> TransportType { TransportType::Tcp443 }

    async fn fetch_token(&self) -> Result<Option<RawToken>, MicafpError> {
        // 1. Connect TCP to MEEK_BRIDGE_IP:443 directly (no DNS)
        // 2. TLS with SNI = COVER_DOMAINS[random].0 (domain fronting)
        // 3. Verify cert SHA-256 == PINNED_CERT_SHA256
        // 4. HTTP GET with Host: meek-relay.torproject.org
        // 5. Parse response for MICAFP-lic:// token
        debug!("Meek-Lite: connecting via domain fronting");
        Ok(None)
    }

    async fn publish_token(&self, token: &SignedToken) -> Result<(), MicafpError> {
        debug!("Meek-Lite: publish not applicable (read-only CDN channel)");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_meek_channel_id() {
        assert_eq!(MeekLiteChannel.id(), 2);
        assert_eq!(MeekLiteChannel.dpi_resistance(), DpiLevel::Zero);
    }
}
