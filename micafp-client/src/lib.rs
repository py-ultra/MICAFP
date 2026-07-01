//! MICAFP Client SDK — v10.0
//! Embeds into Android/iOS/Linux/Windows apps.
//! Single entry point: call `check_license()` before forwarding each traffic batch.

pub use micafp_core::{
    MicafpError,
    EngineConfig,
    engine::run_engine,
    token::{TokenStatus, VerifiedToken},
    resource::{BatteryState, ThermalState, NetworkType},
};

/// High-level license gate. Returns true if traffic should be forwarded.
/// Called from the VPN tunnel before forwarding each packet batch.
pub async fn is_traffic_allowed(
    cache_path: std::path::PathBuf,
    admin_pubkeys: Vec<[u8; 32]>,
) -> bool {
    use micafp_core::{
        cache::EncryptedCache,
        hardware,
        time,
        token::{self, VerifierState},
    };

    let hid = hardware::compute_hid().unwrap_or([0u8; 32]);
    let cache = EncryptedCache::new(cache_path, &hid);
    let state = match cache.load() {
        Ok(s)  => s,
        Err(_) => return false,
    };

    let ntp_unix = match time::get_consensus_time(state.last_ntp).await {
        Ok(t)  => t.unix_secs,
        Err(_) => {
            // Grace: use cached time if NTP unreachable
            if state.last_ntp == 0 { return false; }
            state.last_ntp
        }
    };

    let token_uri = match &state.token {
        Some(t) => t.clone(),
        None    => return false,
    };

    let verifier = VerifierState {
        admin_pubkeys,
        device_hid:         hid,
        last_accepted_seq:  state.last_accepted_seq,
        last_confirmed_ntp: state.last_ntp,
        revoked_uids:       state.revoked_uids.clone(),
        ntp_unix_now:       ntp_unix,
    };

    match token::verify_token(&token_uri, &verifier) {
        Ok(verified) => verified.status.allows_traffic(),
        Err(_)       => false,
    }
}
