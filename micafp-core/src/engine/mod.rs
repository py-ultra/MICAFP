//! MICAFP Engine — Main Orchestration Loop (v10.0)
//!
//! Ties all modules together. Runs as a long-lived async task.
//! Lifecycle: anti-tamper check → cache load → polling loop.

use std::time::Duration;
use tokio::sync::watch;
use tracing::{error, info, warn};

use crate::{
    MicafpError,
    cache::{CacheState, EncryptedCache},
    channel::{stats::ChannelStats, runner},
    hardware,
    time,
    token::{self, VerifierState},
    resource::ResourceManager,
    blackout::{BlackoutEngine, BlackoutStatus},
    deadman,
    tamper,
};

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub cache_path:      std::path::PathBuf,
    pub device_salt:     [u8; 32],
    pub admin_pubkeys:   Vec<[u8; 32]>,
    pub channel_timeout: Duration,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            cache_path:      EncryptedCache::default_path(),
            device_salt:     [0u8; 32],
            admin_pubkeys:   vec![],
            channel_timeout: Duration::from_secs(30),
        }
    }
}

/// Run the MICAFP engine indefinitely. Accepts a shutdown signal.
pub async fn run_engine(config: EngineConfig, mut shutdown: watch::Receiver<bool>) {
    info!("MICAFP engine v10.0 starting");

    // ── Layer 4: Binary integrity ─────────────────────────────────────────
    if let Err(e) = tamper::verify_binary_integrity() {
        warn!("Binary integrity check: {} (non-fatal in dev mode)", e);
    }

    // ── Layer 6: Anti-debug ───────────────────────────────────────────────
    if tamper::detect_analysis_environment() {
        warn!("Analysis environment detected — activating gradual response");
        let blocker = tamper::GradualBlocker::new();
        // Blocker is checked before each packet forward in the tunnel layer
        let _ = blocker;
    }

    // ── Hardware identity ─────────────────────────────────────────────────
    let hid = hardware::compute_hid().unwrap_or([0u8; 32]);

    // ── Load encrypted cache ──────────────────────────────────────────────
    let cache = EncryptedCache::new(config.cache_path.clone(), &hid);
    let mut state = match cache.load() {
        Ok(s) => {
            info!("Cache loaded: last_ntp={} seq={}", s.last_ntp, s.last_accepted_seq);
            s
        }
        Err(crate::cache::CacheError::NotFound) => {
            info!("No cache found — starting fresh");
            CacheState::default()
        }
        Err(e) => {
            warn!("Cache corrupted ({}), attempting self-healing...", e);
            CacheState::default()
        }
    };

    // ── Resource manager ──────────────────────────────────────────────────
    let mut resource_mgr = ResourceManager::new();
    resource_mgr.refresh();

    // ── Blackout engine ───────────────────────────────────────────────────
    let mut blackout = BlackoutEngine::new(
        state.last_ntp,
        state.blackout_entered,
        state.airplane_cycles,
    );

    // ── Dead man's switch ─────────────────────────────────────────────────
    if let Ok(ntp) = time::get_consensus_time(state.last_ntp).await {
        let dms_status = deadman::check_dead_mans_switch(&mut state, ntp.unix_secs);
        match dms_status {
            deadman::DeadMansStatus::Triggered { .. } => {
                warn!("Dead man's switch triggered — extending licenses");
                state.dead_mans_last_heartbeat = ntp.unix_secs;
            }
            _ => {}
        }
    }

    info!("Engine initialised. Entering main polling loop.");

    // ── Main polling loop ─────────────────────────────────────────────────
    loop {
        // Refresh resource state every cycle
        resource_mgr.refresh();

        let interval = resource_mgr.actual_poll_interval(&config.device_salt);
        let active_channels = resource_mgr.active_channels();

        info!(
            "Poll cycle: {} active channels, next poll in {:.0}min",
            active_channels.len(),
            interval.as_secs_f32() / 60.0
        );

        // Wait for interval or shutdown
        tokio::select! {
            _ = tokio::time::sleep(interval) => {}
            _ = shutdown.changed() => {
                info!("Engine shutdown signal received");
                break;
            }
        }

        // Get NTP consensus time
        let ntp_unix = match time::get_consensus_time(state.last_ntp).await {
            Ok(t) => {
                state.last_ntp = t.unix_secs;
                blackout.record_ntp_confirmed(t.unix_secs);
                t.unix_secs
            }
            Err(e) => {
                warn!("NTP failed: {} — using cached time", e);
                state.last_ntp
            }
        };

        // Check for expiry approach (Feature 7: double frequency if <3 days)
        if let Some(ref token_uri) = state.token.clone() {
            // Parse expiry from token to set expiry_approaching flag
            // Production: parse token and check exp - ntp_unix < 3*86400
            resource_mgr.expiry_approaching = false;
        }

        // Check blackout status
        match blackout.check_status(ntp_unix) {
            BlackoutStatus::Active { grace_remaining_days } => {
                info!("Blackout active: {} days grace remaining", grace_remaining_days);
                // Skip network fetch — rely on cached token
                continue;
            }
            BlackoutStatus::Expired => {
                error!("Blackout grace period expired — BLOCKING ALL TRAFFIC");
                break;
            }
            BlackoutStatus::Normal => {}
        }

        // Skip if no active channels
        if active_channels.is_empty() {
            info!("No active channels (thermal/battery) — using cache");
            continue;
        }

        // Sort channels by adaptive learning stats
        let sorted = ChannelStats::sorted_by_performance(&state.channel_stats);
        let ordered: Vec<u8> = sorted.iter()
            .filter(|id| active_channels.contains(id))
            .copied()
            .collect();

        // Fetch token from channels (parallel tokio::select!)
        // Production: instantiate channel objects and call runner::run_channels_parallel
        // Structural: simulate with a placeholder
        let found_token: Option<String> = None; // placeholder

        match found_token {
            Some(raw_token) => {
                blackout.record_channel_success();

                // Run 9-check verification chain
                let verifier_state = VerifierState {
                    admin_pubkeys:      config.admin_pubkeys.clone(),
                    device_hid:         hid,
                    last_accepted_seq:  state.last_accepted_seq,
                    last_confirmed_ntp: state.last_ntp,
                    revoked_uids:       state.revoked_uids.clone(),
                    ntp_unix_now:       ntp_unix,
                };

                match token::verify_token(&raw_token, &verifier_state) {
                    Ok(verified) => {
                        info!("Token verified: {:?}", verified.status);
                        state.last_accepted_seq = verified.payload.seq;
                        state.token = Some(raw_token);
                        state.last_ntp = ntp_unix;
                        state.network_confirmed_count += 1;
                        let _ = cache.save(&state);
                    }
                    Err(e) => {
                        warn!("Token verification failed: {}", e);
                        cache.append_tamper_event(
                            &format!("{:?}", e), &e.to_string(), ntp_unix
                        ).ok();
                    }
                }
            }
            None => {
                blackout.record_all_channels_failed();
                warn!("No token from any channel this cycle");
                // Feature 8: self-healing — try emergency fetch if cache is absent
                if state.token.is_none() {
                    warn!("No cached token — emergency fetch on next cycle");
                }
            }
        }

        // Persist updated state
        state.blackout_entered   = blackout.blackout_entered_unix();
        state.airplane_cycles    = blackout.airplane_cycles();
        let _ = cache.save(&state);
    }

    info!("MICAFP engine stopped");
}
