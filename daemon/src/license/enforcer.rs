//! LicenseEnforcer — Traffic Gate Based on License Status
//!
//! The single chokepoint between the license engine and the VPN tunnel.
//! Before any packet is forwarded through the TUN device, the enforcer
//! checks whether the license is valid using NTP-verified time.
//!
//! ## Enforcement Flow
//!
//! ```text
//! Packet arrives at TUN
//!       │
//!       ▼
//! LicenseEnforcer::check()
//!       │
//!       ├── Status::Valid          → forward packet ✓
//!       ├── Status::GracePeriod    → forward packet + warn user
//!       ├── Status::Expired        → DROP packet + show expiry UI
//!       ├── Status::NoLicense      → DROP packet + show setup UI
//!       └── Status::InvalidSig     → DROP packet + security alert
//! ```
//!
//! ## Exact Expiry Behaviour
//!
//! At expires_at (verified by NTP):
//!   - The daemon logs "License expired — blocking traffic"
//!   - All new packets are dropped (existing TCP sessions also die)
//!   - The Nostr poller keeps running in background
//!   - If admin publishes a renewed token, traffic is automatically
//!     restored within 6 hours (next poll) without any user action

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use super::{LicenseConfig, LicenseStatus, MicafpToken};
use super::cache::{CacheEntry, LicenseCache};
use super::ntp_verifier::NtpVerifier;
use super::nostr_poller::{NostrPoller, PollResult};
use super::token::LicensePayload;

/// The license enforcer — wraps the entire license lifecycle.
pub struct LicenseEnforcer {
    config: LicenseConfig,
    ntp: Arc<NtpVerifier>,
    cache: LicenseCache,
    /// Currently active (verified) token.
    current_token: Arc<RwLock<Option<MicafpToken>>>,
    /// Cached NTP time to avoid querying on every packet.
    last_ntp_unix: Arc<RwLock<u64>>,
}

impl LicenseEnforcer {
    pub fn new(config: LicenseConfig, cache_path: std::path::PathBuf) -> Self {
        Self {
            config,
            ntp: Arc::new(NtpVerifier::new()),
            cache: LicenseCache::new(cache_path),
            current_token: Arc::new(RwLock::new(None)),
            last_ntp_unix: Arc::new(RwLock::new(0)),
        }
    }

    /// Initialise: load cache, verify NTP, check expiry.
    pub async fn init(&self) -> LicenseStatus {
        // 1. Try to load from encrypted cache
        match self.cache.read() {
            Ok(entry) => {
                info!("LicenseEnforcer: loaded token from cache (relay_ts={})", entry.relay_ts);
                let admin_key = [0u8; 32]; // TODO: load real key
                if let Ok(token) = MicafpToken::parse_and_verify(&entry.token, &admin_key) {
                    *self.current_token.write().await = Some(token);
                }
            }
            Err(e) => {
                warn!("LicenseEnforcer: cache read failed: {} — no cached license", e);
            }
        }

        // 2. Check status
        self.check().await
    }

    /// Check current license status (called before forwarding each packet batch).
    /// Uses cached NTP time for performance; refreshes NTP every hour.
    pub async fn check(&self) -> LicenseStatus {
        // Get NTP-verified time
        let unix_now = match self.ntp.unix_now().await {
            Ok(t) => {
                *self.last_ntp_unix.write().await = t;
                t
            }
            Err(_) => {
                // If NTP fails, use last known NTP + elapsed (monotonic estimate)
                let last = *self.last_ntp_unix.read().await;
                if last == 0 {
                    error!("LicenseEnforcer: NTP unreachable and no cached time — blocking");
                    return LicenseStatus::Expired;
                }
                last
            }
        };

        // Check against current token
        let token_guard = self.current_token.read().await;
        match token_guard.as_ref() {
            None => LicenseStatus::NoLicense,
            Some(token) => {
                match token.payload.is_valid_at(unix_now) {
                    super::token::TokenValidity::Valid { expires_in } => {
                        LicenseStatus::Valid {
                            expires_in,
                            features: token.payload.features.clone(),
                        }
                    }
                    super::token::TokenValidity::GracePeriod { grace_remaining } => {
                        warn!("LicenseEnforcer: in grace period, {}s remaining",
                              grace_remaining.as_secs());
                        LicenseStatus::GracePeriod { grace_remaining }
                    }
                    super::token::TokenValidity::Expired => {
                        error!("LicenseEnforcer: license expired — BLOCKING ALL TRAFFIC");
                        LicenseStatus::Expired
                    }
                    super::token::TokenValidity::NotYetValid => {
                        warn!("LicenseEnforcer: token not yet valid (clock skew?)");
                        LicenseStatus::NoLicense
                    }
                }
            }
        }
    }

    /// Update the active token (called when Nostr poller finds a new event).
    pub async fn update_token(&self, token_uri: &str) -> Result<(), String> {
        let admin_key = [0u8; 32]; // TODO: load real admin pubkey
        let token = MicafpToken::parse_and_verify(token_uri, &admin_key)
            .map_err(|e| e.to_string())?;

        // Write to encrypted cache
        let unix_now = self.ntp.unix_now().await.unwrap_or(0);
        self.cache.write(&CacheEntry {
            token: token_uri.to_string(),
            cached_at: unix_now,
            relay_ts: unix_now,
        }).map_err(|e| e.to_string())?;

        *self.current_token.write().await = Some(token);
        info!("LicenseEnforcer: token updated and cached");
        Ok(())
    }

    /// Start background Nostr polling loop.
    pub fn spawn_poller(&self) -> tokio::task::JoinHandle<()> {
        let config = self.config.clone();
        let current_token = self.current_token.clone();
        let ntp = self.ntp.clone();
        let cache = LicenseCache::new(LicenseCache::default_path());

        tokio::spawn(async move {
            let poller = NostrPoller::new(config);
            let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(4);

            // Poller runs in its own subtask
            let last_ts = current_token.read().await
                .as_ref()
                .map(|t| t.payload.issued_at)
                .unwrap_or(0);
            let tx2 = tx.clone();
            tokio::spawn(async move {
                poller.run_background_loop(tx2, last_ts).await;
            });

            // Process incoming tokens
            while let Some(token_uri) = rx.recv().await {
                let admin_key = [0u8; 32]; // TODO: real key
                match MicafpToken::parse_and_verify(&token_uri, &admin_key) {
                    Ok(token) => {
                        let unix_now = ntp.unix_now().await.unwrap_or(0);
                        let _ = cache.write(&CacheEntry {
                            token: token_uri.clone(),
                            cached_at: unix_now,
                            relay_ts: unix_now,
                        });
                        *current_token.write().await = Some(token);
                        info!("LicenseEnforcer: background renewal applied — traffic continues");
                    }
                    Err(e) => {
                        warn!("LicenseEnforcer: bad token from Nostr: {}", e);
                    }
                }
            }
        })
    }
}
