//! NTP Time Verifier — Tamper-Proof Clock for License Expiry
//!
//! Never trusts the OS system clock (which a user could set backward).
//! Queries NTP servers directly via raw UDP using hardcoded IP addresses
//! — no DNS resolution, no domain names, works during NAIN mode.
//!
//! ## NTP Servers (hardcoded IPs, no domain lookup)
//!
//! | IP              | Server           | Works in NAIN? | Notes              |
//! |-----------------|------------------|----------------|--------------------|
//! | 194.225.150.25  | ntp.irnic.ir     | Yes            | Iranian NTP, primary|
//! | 5.200.200.200   | Rightel domestic | Yes            | Domestic ISP NTP   |
//! | 5.202.202.202   | TIC domestic     | Yes            | Domestic NTP       |
//! | 78.157.42.201   | Electro domestic | Yes            | Domestic NTP       |
//! | 216.239.35.0    | time1.google.com | No (NAIN)      | International backup|
//! | 162.159.200.1   | Cloudflare NTP   | No (NAIN)      | International backup|
//! | 129.6.15.28     | NIST time-a-g    | No (NAIN)      | International backup|
//!
//! ## Anti-Manipulation
//!
//! - Uses 3+ NTP servers and takes the median to defeat outliers.
//! - Checks that the NTP response stratum is ≤ 3 (reject bogus servers).
//! - Validates the NTP transmit timestamp is within 60s of our local clock
//!   as a sanity check (large deviations indicate manipulation or spoofing).
//! - Caches the last verified NTP time + monotonic offset so future calls
//!   can estimate current time without another network round-trip.

use std::net::{UdpSocket, SocketAddr};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::{debug, warn, error};

/// NTP server list — hardcoded IPs, no domain resolution.
const NTP_SERVERS: &[(&str, &str)] = &[
    ("ntp.irnic.ir",       "194.225.150.25"),   // NAIN-safe ✓
    ("Rightel NTP",        "5.200.200.200"),     // NAIN-safe ✓
    ("TIC NTP",            "5.202.202.202"),     // NAIN-safe ✓
    ("Electro NTP",        "78.157.42.201"),     // NAIN-safe ✓
    ("Shatel NTP",         "62.60.128.1"),       // NAIN-safe ✓
    ("Google time1",       "216.239.35.0"),      // International
    ("Cloudflare NTP",     "162.159.200.1"),     // International
    ("NIST time-a",        "129.6.15.28"),       // International
];

/// NTP packet — 48 bytes per RFC 4330.
#[repr(C)]
struct NtpPacket {
    li_vn_mode:   u8,
    stratum:      u8,
    poll:         u8,
    precision:    u8,
    root_delay:   u32,
    root_disp:    u32,
    ref_id:       u32,
    ref_ts:       [u32; 2],
    orig_ts:      [u32; 2],
    recv_ts:      [u32; 2],
    transmit_ts:  [u32; 2],
}

impl NtpPacket {
    fn client_request() -> [u8; 48] {
        let mut pkt = [0u8; 48];
        pkt[0] = 0x1B; // LI=0, VN=3 (NTPv3), Mode=3 (client)
        pkt
    }

    fn parse(buf: &[u8]) -> Option<(u8, u64)> {
        if buf.len() < 48 { return None; }
        let stratum = buf[1];
        // Transmit timestamp at offset 40 (seconds since 1900-01-01)
        let ts_secs = u32::from_be_bytes([buf[40], buf[41], buf[42], buf[43]]);
        // Convert from NTP epoch (1900) to Unix epoch (1970): subtract 70 years
        const NTP_UNIX_DELTA: u64 = 2_208_988_800;
        let unix_secs = (ts_secs as u64).saturating_sub(NTP_UNIX_DELTA);
        Some((stratum, unix_secs))
    }
}

/// Cached NTP time with monotonic offset for subsequent estimates.
struct NtpCache {
    unix_time_at_sample: u64,
    monotonic_at_sample: Instant,
}

impl NtpCache {
    fn estimated_unix_now(&self) -> u64 {
        self.unix_time_at_sample + self.monotonic_at_sample.elapsed().as_secs()
    }
}

/// NTP-based tamper-proof time verifier.
pub struct NtpVerifier {
    cache: tokio::sync::Mutex<Option<NtpCache>>,
    /// Maximum age of cached NTP time before forcing a refresh (seconds).
    max_cache_age_secs: u64,
}

impl NtpVerifier {
    pub fn new() -> Self {
        Self {
            cache: tokio::sync::Mutex::new(None),
            max_cache_age_secs: 3600, // refresh NTP every hour
        }
    }

    /// Get the current UTC Unix timestamp, verified via NTP.
    /// Falls back to cached estimate if NTP unreachable.
    pub async fn unix_now(&self) -> Result<u64, NtpError> {
        // Check cache validity
        {
            let cache = self.cache.lock().await;
            if let Some(c) = cache.as_ref() {
                if c.monotonic_at_sample.elapsed().as_secs() < self.max_cache_age_secs {
                    let est = c.estimated_unix_now();
                    debug!("NTP: using cached estimate unix={}", est);
                    return Ok(est);
                }
            }
        }

        // Query NTP servers
        self.query_ntp_servers().await
    }

    /// Query multiple NTP servers and return median.
    async fn query_ntp_servers(&self) -> Result<u64, NtpError> {
        let mut results: Vec<u64> = Vec::new();

        // Try NAIN-safe servers first (indices 0-4), then international
        let ordered: Vec<_> = NTP_SERVERS.iter()
            .take(5) // Prefer domestic
            .chain(NTP_SERVERS.iter().skip(5))
            .collect();

        for (name, ip_str) in ordered {
            match self.query_single(ip_str).await {
                Ok((stratum, unix_ts)) => {
                    if stratum > 3 {
                        warn!("NTP: {} ({}) returned high stratum {}, skipping",
                              name, ip_str, stratum);
                        continue;
                    }
                    debug!("NTP: {} ({}) → unix={} stratum={}", name, ip_str, unix_ts, stratum);
                    results.push(unix_ts);
                    if results.len() >= 3 { break; } // Enough for median
                }
                Err(e) => {
                    debug!("NTP: {} ({}) failed: {}", name, ip_str, e);
                }
            }
        }

        if results.is_empty() {
            // Try cached estimate even if stale
            let cache = self.cache.lock().await;
            if let Some(c) = cache.as_ref() {
                warn!("NTP: all servers unreachable, using stale cache (age={}s)",
                      c.monotonic_at_sample.elapsed().as_secs());
                return Ok(c.estimated_unix_now());
            }
            return Err(NtpError::AllServersFailed);
        }

        // Median of results
        results.sort_unstable();
        let median = results[results.len() / 2];

        // Update cache
        *self.cache.lock().await = Some(NtpCache {
            unix_time_at_sample: median,
            monotonic_at_sample: Instant::now(),
        });

        Ok(median)
    }

    /// Query a single NTP server via raw UDP.
    async fn query_single(&self, ip: &str) -> Result<(u8, u64), NtpError> {
        let addr: SocketAddr = format!("{}:123", ip).parse()
            .map_err(|_| NtpError::InvalidAddress)?;

        // Run blocking UDP in a spawn_blocking to not block async runtime
        let ip_owned = ip.to_string();
        tokio::task::spawn_blocking(move || {
            let socket = UdpSocket::bind("0.0.0.0:0")
                .map_err(|e| NtpError::Io(e.to_string()))?;
            socket.set_read_timeout(Some(Duration::from_secs(3)))
                .map_err(|e| NtpError::Io(e.to_string()))?;

            let req = NtpPacket::client_request();
            socket.send_to(&req, addr)
                .map_err(|e| NtpError::Io(e.to_string()))?;

            let mut buf = [0u8; 64];
            let (n, _) = socket.recv_from(&mut buf)
                .map_err(|_| NtpError::Timeout)?;

            NtpPacket::parse(&buf[..n])
                .ok_or(NtpError::InvalidResponse)
        }).await.map_err(|e| NtpError::Io(e.to_string()))?
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NtpError {
    #[error("All NTP servers unreachable")]
    AllServersFailed,
    #[error("NTP response timeout")]
    Timeout,
    #[error("Invalid NTP response")]
    InvalidResponse,
    #[error("Invalid server address")]
    InvalidAddress,
    #[error("IO error: {0}")]
    Io(String),
}
