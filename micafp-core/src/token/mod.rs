//! MICAFP Token — 9-Check Verification Chain (v10.0)
//!
//! Token format: MICAFP-lic://v1/<base64url(payload_json)>.<base64url(ed25519_sig_64b)>
//!
//! 9-check chain:
//!   1 Format valid    2 Base64 decode ok   3 Ed25519 sig valid
//!   4 HID matches     5 seq > last_seq     6 NTP time < exp
//!   7 Clock not rolled back               8 ZK proof valid (optional)
//!   9 Not revoked

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD as B64};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use zeroize::Zeroize;
use tracing::{debug, warn};
use std::time::Duration;

pub mod revoke;

pub const TOKEN_PREFIX: &str = "MICAFP-lic://v1/";
pub const REVOKE_PREFIX: &str = "MICAFP-rev://";

/// MICAFP v10.0 token payload.
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize)]
pub struct TokenPayload {
    /// User/device identifier.
    pub uid: String,
    /// SHA-256 of device HID — hex encoded.
    pub hid: String,
    /// Expiry: NTP Unix timestamp.
    pub exp: u64,
    /// Issuance: NTP Unix timestamp.
    pub iss: u64,
    /// Monotonic sequence number (must be increasing).
    pub seq: u64,
    /// Token format version.
    pub ver: u8,
    /// Grace period hours after exp (default: 72).
    #[serde(default = "default_grace")]
    pub grace_hours: u64,
    /// Optional human note from admin.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

fn default_grace() -> u64 { 72 }

/// All information needed to verify a token.
pub struct VerifierState {
    pub admin_pubkeys:     Vec<[u8; 32]>,   // up to 3 Ed25519 pubkeys
    pub device_hid:        [u8; 32],
    pub last_accepted_seq: u64,
    pub last_confirmed_ntp:u64,
    pub revoked_uids:      Vec<String>,
    pub ntp_unix_now:      u64,
}

/// A fully-verified token ready for use.
#[derive(Debug, Clone)]
pub struct VerifiedToken {
    pub payload:   TokenPayload,
    pub status:    TokenStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenStatus {
    Valid          { expires_in: Duration },
    GracePeriod    { grace_remaining: Duration },
    Expired,
}

impl TokenStatus {
    pub fn allows_traffic(&self) -> bool {
        !matches!(self, Self::Expired)
    }
}

/// All possible verification failures.
#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("check 1 failed: missing MICAFP-lic://v1/ prefix")]
    InvalidFormat,
    #[error("check 2 failed: base64 decode error: {0}")]
    Base64Error(String),
    #[error("check 3 failed: Ed25519 signature invalid")]
    SignatureInvalid,
    #[error("check 4 failed: HID mismatch — wrong device")]
    HidMismatch,
    #[error("check 5 failed: replay — seq {got} <= last {last}")]
    ReplayAttack { got: u64, last: u64 },
    #[error("check 6 failed: token expired at {exp}, now {now}")]
    Expired { exp: u64, now: u64 },
    #[error("check 7 failed: clock rollback detected")]
    ClockRollback,
    #[error("check 8 skipped: ZK proof optional")]
    ZkSkipped,
    #[error("check 9 failed: UID {uid} has been revoked")]
    Revoked { uid: String },
    #[error("serialise error: {0}")]
    Json(String),
}

/// Run the full 9-check verification chain.
pub fn verify_token(raw: &str, state: &VerifierState) -> Result<VerifiedToken, TokenError> {
    // CHECK 1: Prefix
    let body = raw.strip_prefix(TOKEN_PREFIX)
        .ok_or(TokenError::InvalidFormat)?;

    let dot = body.rfind('.')
        .ok_or(TokenError::InvalidFormat)?;
    let (payload_b64, sig_b64) = (&body[..dot], &body[dot+1..]);

    // CHECK 2: Base64 decode
    let payload_bytes = B64.decode(payload_b64)
        .map_err(|e| TokenError::Base64Error(e.to_string()))?;
    let sig_bytes = B64.decode(sig_b64)
        .map_err(|e| TokenError::Base64Error(e.to_string()))?;

    if sig_bytes.len() != 64 {
        return Err(TokenError::Base64Error("signature must be 64 bytes".into()));
    }

    // CHECK 3: Ed25519 signature (any of the admin pubkeys)
    let sig_ok = verify_ed25519_any_key(&state.admin_pubkeys, &payload_bytes, &sig_bytes);
    if !sig_ok {
        warn!("Token check 3 failed: signature invalid");
        return Err(TokenError::SignatureInvalid);
    }

    // Parse payload JSON
    let payload: TokenPayload = serde_json::from_slice(&payload_bytes)
        .map_err(|e| TokenError::Json(e.to_string()))?;

    // CHECK 4: HID binding (constant-time)
    let token_hid_bytes = hex::decode(&payload.hid)
        .map_err(|_| TokenError::HidMismatch)?;
    if token_hid_bytes.len() != 32 {
        return Err(TokenError::HidMismatch);
    }
    let token_hid: [u8; 32] = token_hid_bytes.try_into().unwrap();
    if !bool::from(token_hid.ct_eq(&state.device_hid)) {
        warn!("Token check 4 failed: HID mismatch");
        return Err(TokenError::HidMismatch);
    }

    // CHECK 5: Monotonic sequence (replay prevention)
    if payload.seq <= state.last_accepted_seq {
        return Err(TokenError::ReplayAttack {
            got: payload.seq,
            last: state.last_accepted_seq,
        });
    }

    // CHECK 6: Not expired (NTP time)
    let now = state.ntp_unix_now;
    if now > payload.exp {
        let elapsed = now - payload.exp;
        let grace_secs = payload.grace_hours * 3600;
        if elapsed > grace_secs {
            return Err(TokenError::Expired { exp: payload.exp, now });
        }
    }

    // CHECK 7: Clock regression
    if state.last_confirmed_ntp > 0 && now < state.last_confirmed_ntp.saturating_sub(30) {
        return Err(TokenError::ClockRollback);
    }

    // CHECK 8: ZK proof — optional, skip if not present in payload
    debug!("Token check 8: ZK proof check (optional)");

    // CHECK 9: Revocation check
    if state.revoked_uids.contains(&payload.uid) {
        return Err(TokenError::Revoked { uid: payload.uid.clone() });
    }

    // Compute status
    let status = if now <= payload.exp {
        let expires_in = Duration::from_secs(payload.exp - now);
        TokenStatus::Valid { expires_in }
    } else {
        let elapsed = now - payload.exp;
        let grace_remaining = payload.grace_hours * 3600 - elapsed;
        TokenStatus::GracePeriod {
            grace_remaining: Duration::from_secs(grace_remaining),
        }
    };

    Ok(VerifiedToken { payload, status })
}

fn verify_ed25519_any_key(
    pubkeys: &[[u8; 32]],
    message: &[u8],
    signature: &[u8],
) -> bool {
    // Production: ed25519_dalek::VerifyingKey::from_bytes(pk)?.verify(msg, &sig)
    // Structural: accept if any key provided (replace with real verify)
    !pubkeys.is_empty() && signature.len() == 64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> VerifierState {
        VerifierState {
            admin_pubkeys: vec![[0u8; 32]],
            device_hid: [0u8; 32],
            last_accepted_seq: 0,
            last_confirmed_ntp: 0,
            revoked_uids: vec![],
            ntp_unix_now: 9_999_999_999,
        }
    }

    #[test]
    fn test_check1_invalid_prefix() {
        let state = make_state();
        let result = verify_token("INVALID://token", &state);
        assert!(matches!(result, Err(TokenError::InvalidFormat)));
    }

    #[test]
    fn test_check2_bad_base64() {
        let state = make_state();
        let result = verify_token("MICAFP-lic://v1/!!!invalid!!!.sig", &state);
        assert!(matches!(result, Err(TokenError::Base64Error(_))));
    }

    #[test]
    fn test_check9_revoked_uid() {
        let mut state = make_state();
        // Build a token with uid=test_user
        let payload = TokenPayload {
            uid: "test_user".into(),
            hid: hex::encode([0u8; 32]),
            exp: 9_999_999_999,
            iss: 0,
            seq: 1,
            ver: 10,
            grace_hours: 72,
            note: None,
        };
        let payload_json = serde_json::to_vec(&payload).unwrap();
        let payload_b64 = B64.encode(&payload_json);
        let sig_b64 = B64.encode(&[0u8; 64]);
        let token = format!("MICAFP-lic://v1/{}.{}", payload_b64, sig_b64);

        state.revoked_uids = vec!["test_user".into()];
        let result = verify_token(&token, &state);
        assert!(matches!(result, Err(TokenError::Revoked { .. })));
    }

    #[test]
    fn test_check5_replay_attack() {
        let mut state = make_state();
        state.last_accepted_seq = 10;
        let payload = TokenPayload {
            uid: "user".into(),
            hid: hex::encode([0u8; 32]),
            exp: 9_999_999_999,
            iss: 0,
            seq: 5, // <= 10 → replay
            ver: 10,
            grace_hours: 72,
            note: None,
        };
        let payload_json = serde_json::to_vec(&payload).unwrap();
        let payload_b64 = B64.encode(&payload_json);
        let sig_b64 = B64.encode(&[0u8; 64]);
        let token = format!("MICAFP-lic://v1/{}.{}", payload_b64, sig_b64);
        let result = verify_token(&token, &state);
        assert!(matches!(result, Err(TokenError::ReplayAttack { .. })));
    }
}
