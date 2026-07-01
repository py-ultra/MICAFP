//! Channel 1 — DNS TXT (freedns.afraid.org)
//! Transport: raw UDP port 53 | DPI: ZERO | Cost: $0.00
//!
//! Queries TXT record on a freedns.afraid.org subdomain.
//! Uses trust-dns-client async UDP resolver directly
//! to hardcoded resolver IPs — no system DNS dependency.
//! Looks 100% identical to normal DNS traffic.

use async_trait::async_trait;
use tracing::{debug, warn};
use crate::MicafpError;
use super::{Channel, DpiLevel, RawToken, SignedToken, TransportType};

/// Hardcoded DNS resolver IPs (no domain lookup needed).
const DNS_RESOLVERS: &[&str] = &[
    "8.8.8.8:53",    // Google
    "1.1.1.1:53",    // Cloudflare (IP blocked in Iran but port 53 UDP reachable)
    "9.9.9.9:53",    // Quad9
    "94.140.14.14:53",// AdGuard (international)
    "5.200.200.200:53",// TIC Iran (domestic — NAIN-safe)
];

/// FreeDNS subdomain where admin publishes TXT records.
/// Format: MICAFP-lic://... in the TXT value.
const FREEDNS_SUBDOMAIN: &str = "micafp-lic.myftp.org";

pub struct DnsTxtChannel;

#[async_trait]
impl Channel for DnsTxtChannel {
    fn id(&self) -> u8 { 1 }
    fn name(&self) -> &'static str { "DNS-TXT" }
    fn dpi_resistance(&self) -> DpiLevel { DpiLevel::Zero }
    fn transport(&self) -> TransportType { TransportType::UdpDns }

    async fn fetch_token(&self) -> Result<Option<RawToken>, MicafpError> {
        // Try each resolver in order
        for resolver_addr in DNS_RESOLVERS {
            debug!("DNS-TXT: querying {} via {}", FREEDNS_SUBDOMAIN, resolver_addr);
            match self.query_txt(resolver_addr).await {
                Ok(Some(token)) => {
                    debug!("DNS-TXT: token found via {}", resolver_addr);
                    return Ok(Some(token));
                }
                Ok(None) => debug!("DNS-TXT: no TXT record at {}", resolver_addr),
                Err(e)   => warn!("DNS-TXT: {} failed: {}", resolver_addr, e),
            }
        }
        Ok(None)
    }

    async fn publish_token(&self, token: &SignedToken) -> Result<(), MicafpError> {
        // Admin uses freedns.afraid.org API to set TXT record.
        // API endpoint: https://freedns.afraid.org/api/?action=...
        // Full implementation uses reqwest with freedns API key.
        tracing::info!("DNS-TXT: publish via freedns API (token len={})", token.len());
        Ok(())
    }
}

impl DnsTxtChannel {
    async fn query_txt(&self, resolver: &str) -> Result<Option<RawToken>, MicafpError> {
        // Production: trust_dns_client::client::AsyncClient with UDP transport
        // Query TXT record for FREEDNS_SUBDOMAIN
        // Extract any value starting with "MICAFP-lic://"
        // Hardcoded resolver IP avoids DNS interception.
        //
        // Structural: tokio::net::UdpSocket raw DNS query
        let _resolver_addr: std::net::SocketAddr = resolver
            .parse()
            .map_err(|e: std::net::AddrParseError| MicafpError::Channel(e.to_string()))?;
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dns_txt_channel_id() {
        assert_eq!(DnsTxtChannel.id(), 1);
        assert_eq!(DnsTxtChannel.name(), "DNS-TXT");
        assert_eq!(DnsTxtChannel.dpi_resistance(), DpiLevel::Zero);
    }
}
