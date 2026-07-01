//! Nostr Relay Poller — Fetches MICAFP License Events
//!
//! Connects to Nostr relays via WebSocket over TLS (wss://) using
//! hardcoded IP addresses. No DNS resolution. Filters for events
//! published by the admin pubkey with kind 30000 (MICAFP license).
//!
//! Every poll cycle:
//!   1. Shuffle relay list (random order per cycle)
//!   2. Add jitter delay (100–900ms) to avoid fingerprinting
//!   3. Try relays one by one until 3 respond
//!   4. Take the event with the highest created_at
//!   5. Extract and return the MICAFP-lic:// token

use std::time::Duration;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use super::{NOSTR_RELAYS, LicenseConfig};

/// A Nostr event (NIP-01 format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NostrEvent {
    pub id:         String,
    pub pubkey:     String,
    pub created_at: u64,
    pub kind:       u32,
    pub tags:       Vec<Vec<String>>,
    pub content:    String,
    pub sig:        String,
}

/// NIP-01 REQ subscription filter.
#[derive(Debug, Serialize)]
struct NostrFilter {
    authors: Vec<String>,
    kinds:   Vec<u32>,
    limit:   u32,
}

/// Result of a single poll cycle.
#[derive(Debug)]
pub enum PollResult {
    /// New token found that is newer than the cached one.
    NewToken(String),
    /// Found a token, but it's the same as what we already have.
    NoChange,
    /// All relays failed or returned no events.
    NoRelay,
}

/// Nostr relay poller.
pub struct NostrPoller {
    config: LicenseConfig,
}

impl NostrPoller {
    pub fn new(config: LicenseConfig) -> Self {
        Self { config }
    }

    /// Run one poll cycle. Returns the extracted MICAFP-lic:// token if found.
    pub async fn poll_once(&self, last_known_ts: u64) -> PollResult {
        // Jitter: 100–900ms random delay to avoid timing fingerprinting
        let jitter_ms = rand::thread_rng().gen_range(100u64..900);
        tokio::time::sleep(Duration::from_millis(jitter_ms)).await;

        // Shuffle relay list
        let mut relay_order: Vec<usize> = (0..NOSTR_RELAYS.len()).collect();
        {
            use rand::seq::SliceRandom;
            relay_order.shuffle(&mut rand::thread_rng());
        }

        let mut best_event: Option<NostrEvent> = None;
        let mut successful_relays = 0usize;

        for &idx in relay_order.iter().take(self.config.relays_per_cycle * 3) {
            let (hostname, ip, port) = NOSTR_RELAYS[idx];

            debug!("Nostr poll: connecting to {} ({}:{})", hostname, ip, port);

            match self.fetch_from_relay(hostname, ip, *port).await {
                Ok(events) => {
                    successful_relays += 1;
                    debug!("Nostr: {} returned {} events", hostname, events.len());

                    // Pick the newest event
                    for event in events {
                        if event.created_at > best_event.as_ref()
                            .map(|e| e.created_at).unwrap_or(0)
                        {
                            best_event = Some(event);
                        }
                    }

                    if successful_relays >= self.config.relays_per_cycle { break; }
                }
                Err(e) => {
                    debug!("Nostr: {} ({}) failed: {}", hostname, ip, e);
                }
            }
        }

        match best_event {
            None => {
                warn!("Nostr poll: no events found from {} relays", successful_relays);
                PollResult::NoRelay
            }
            Some(event) => {
                if event.created_at <= last_known_ts {
                    debug!("Nostr poll: event not newer than cached (ts={})", event.created_at);
                    return PollResult::NoChange;
                }

                // Extract MICAFP-lic:// token from event content
                let content = &event.content;
                if let Some(start) = content.find("MICAFP-lic://") {
                    let token_str: String = content[start..]
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string();

                    info!("Nostr: found new MICAFP token (relay_ts={})", event.created_at);
                    PollResult::NewToken(token_str)
                } else {
                    warn!("Nostr: event found but no MICAFP-lic:// token in content");
                    PollResult::NoRelay
                }
            }
        }
    }

    /// Connect to a relay and fetch the latest admin events.
    async fn fetch_from_relay(
        &self,
        hostname: &str,
        ip: &str,
        port: u16,
    ) -> Result<Vec<NostrEvent>, String> {
        // Production implementation:
        //
        //   1. Connect TCP to ip:port (not DNS — direct IP)
        //   2. TLS handshake with SNI = hostname
        //   3. WebSocket upgrade: GET wss://hostname/
        //   4. Send NIP-01 REQ message:
        //      ["REQ", "sub1", {"authors":[admin_pubkey], "kinds":[30000], "limit":1}]
        //   5. Read EVENT messages until EOSE
        //   6. Send CLOSE message
        //   7. Return collected events
        //
        // Libraries: tokio-tungstenite + rustls with custom connector
        // that uses the hardcoded IP but presents the hostname as SNI.

        // Structural stub — production replaces this with real WSS:
        Ok(vec![])
    }

    /// Run the continuous background polling loop.
    pub async fn run_background_loop(
        &self,
        token_sink: tokio::sync::mpsc::Sender<String>,
        mut last_known_ts: u64,
    ) {
        info!("Nostr poller: starting background loop (interval={}h)",
              self.config.poll_interval.as_secs() / 3600);

        loop {
            // Wait for poll interval + jitter
            let jitter = rand::thread_rng().gen_range(0..self.config.poll_jitter_secs);
            let wait = self.config.poll_interval + Duration::from_secs(jitter);
            tokio::time::sleep(wait).await;

            match self.poll_once(last_known_ts).await {
                PollResult::NewToken(token) => {
                    info!("Nostr poller: new license token received");
                    last_known_ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default().as_secs();
                    let _ = token_sink.send(token).await;
                }
                PollResult::NoChange => {
                    debug!("Nostr poller: no new token");
                }
                PollResult::NoRelay => {
                    warn!("Nostr poller: all relays unreachable — license cache will be used");
                }
            }
        }
    }
}
