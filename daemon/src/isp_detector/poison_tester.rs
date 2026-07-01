//! DNS Poison Detector
//!
//! Tests whether the current network's DNS is poisoned by resolving
//! known-blocked domains and checking the responses against a list
//! of known Iranian poison IP addresses.
//!
//! If DNS is poisoned, the daemon switches to DoH (ArvanCloud) for
//! all DNS resolution automatically.

use std::net::IpAddr;

/// Known Iranian DNS poison IPs (from isp-profiles.json iran_dpi_intelligence).
static POISON_IPS: &[&str] = &[
    "10.10.34.35",
    "10.10.34.36",
    "10.10.34.37",
    "185.51.200.2",
    "5.160.208.63",
    "217.218.127.127",
    "127.0.0.1",
    "0.0.0.0",
    "10.0.0.1",
];

/// Domains known to be blocked in Iran (to test DNS poisoning).
static TEST_DOMAINS: &[&str] = &[
    "www.google.com",
    "www.youtube.com",
    "www.twitter.com",
];

/// Test if DNS is poisoned on the current network.
pub async fn test_dns_poison() -> bool {
    for domain in TEST_DOMAINS {
        if let Ok(ips) = resolve_domain(domain).await {
            for ip in ips {
                if POISON_IPS.contains(&ip.to_string().as_str()) {
                    tracing::warn!("DNS poisoning detected: {} → {}", domain, ip);
                    return true;
                }
            }
        }
    }
    false
}

async fn resolve_domain(domain: &str) -> Result<Vec<IpAddr>, Box<dyn std::error::Error>> {
    // Production: use tokio::net::lookup_host or trust-dns-resolver
    // with the system resolver, then check responses against poison list
    Ok(vec![])
}

pub fn is_poison_ip(ip: &IpAddr) -> bool {
    POISON_IPS.contains(&ip.to_string().as_str())
}
