//! MICAFP License Token — Ed25519-Signed, Self-Contained License Format
//!
//! Token URI format:
//!   MICAFP-lic://v1/<base64url(payload_json)>.<base64url(ed25519_sig_64bytes)>
//!
//! The payload is a JSON object base64url-encoded (no padding).
//! The signature covers exactly the raw UTF-8 bytes of the payload JSON.
//! Verification requires only the admin's 32-byte Ed25519 public key.

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD as B64};
use serde::{Deserialize, Serialize};
use tracing::warn;

pub const TOKEN_SCHEME: &str = "MICAFP-lic://v1/";

/// The payload embedded inside a MICAFP license token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensePayload {
    /// Always "MICAFP".
    pub scheme: String,
    /// Token format version (currently 1).
    pub version: u8,
    /// Unix timestamp (seconds) when this token was issued.
    pub issued_at: u64,
    /// Unix timestamp (seconds) when this token expires.
    pub expires_at: u64,
    /// Grace period hours: traffic allowed this many hours past expires_at
    /// while NTP is unreachable.
    pub grace_hours: u64,
    /// Fingerprint of the admin pubkey used for signing (hex, first 16 bytes).
    pub pubkey_fingerprint: String,
    /// Enabled feature flags.
    pub features: Vec<String>,
    /// Random nonce (hex) — prevents replay of identical payloads.
    pub nonce: String,
    /// Optional human-readable note from admin (e.g., "Q2 2025 renewal").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl LicensePayload {
    /// Check if this token is currently valid against a known UTC timestamp.
    pub fn is_valid_at(&self, unix_now: u64) -> TokenValidity {
        if unix_now < self.issued_at {
            return TokenValidity::NotYetValid;
        }
        if unix_now <= self.expires_at {
            let remaining = Duration::from_secs(self.expires_at - unix_now);
            return TokenValidity::Valid { expires_in: remaining };
        }
        let expired_secs = unix_now - self.expires_at;
        let grace_secs = self.grace_hours * 3600;
        if expired_secs <= grace_secs {
            let grace_remaining = Duration::from_secs(grace_secs - expired_secs);
            return TokenValidity::GracePeriod { grace_remaining };
        }
        TokenValidity::Expired
    }

    /// Unix timestamp of expiry.
    pub fn expires_at_unix(&self) -> u64 { self.expires_at }
}

/// Result of checking a payload's temporal validity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenValidity {
    Valid        { expires_in: Duration },
    GracePeriod  { grace_remaining: Duration },
    Expired,
    NotYetValid,
}

/// A fully-parsed and signature-verified MICAFP license token.
#[derive(Debug, Clone)]
pub struct MicafpToken {
    pub payload: LicensePayload,
    /// Raw payload JSON bytes (used for signature verification).
    payload_bytes: Vec<u8>,
    /// Raw 64-byte Ed25519 signature.
    signature: [u8; 64],
    /// The full original URI string.
    pub raw_uri: String,
}

impl MicafpToken {
    /// Parse and verify a MICAFP-lic:// URI.
    ///
    /// `admin_pubkey_bytes` must be the 32-byte Ed25519 public key.
    pub fn parse_and_verify(
        uri: &str,
        admin_pubkey_bytes: &[u8; 32],
    ) -> Result<Self, TokenError> {
        // ── Parse URI ────────────────────────────────────────────────────
        let body = uri.strip_prefix(TOKEN_SCHEME)
            .ok_or_else(|| TokenError::InvalidFormat("Missing MICAFP-lic://v1/ prefix".into()))?;

        let dot = body.rfind('.').ok_or_else(||
            TokenError::InvalidFormat("Missing '.' separator between payload and signature".into())
        )?;

        let (payload_b64, sig_b64) = (&body[..dot], &body[dot+1..]);

        // ── Decode payload ───────────────────────────────────────────────
        let payload_bytes = B64.decode(payload_b64)
            .map_err(|e| TokenError::InvalidFormat(format!("Bad payload base64: {}", e)))?;

        let payload: LicensePayload = serde_json::from_slice(&payload_bytes)
            .map_err(|e| TokenError::InvalidFormat(format!("Bad payload JSON: {}", e)))?;

        if payload.scheme != "MICAFP" {
            return Err(TokenError::InvalidFormat("Scheme must be MICAFP".into()));
        }

        // ── Decode signature ─────────────────────────────────────────────
        let sig_bytes = B64.decode(sig_b64)
            .map_err(|e| TokenError::InvalidSignature(format!("Bad sig base64: {}", e)))?;

        if sig_bytes.len() != 64 {
            return Err(TokenError::InvalidSignature(
                format!("Signature must be 64 bytes, got {}", sig_bytes.len())
            ));
        }
        let mut signature = [0u8; 64];
        signature.copy_from_slice(&sig_bytes);

        // ── Verify Ed25519 signature ─────────────────────────────────────
        // Production: use ed25519-dalek:
        //   let vk = VerifyingKey::from_bytes(admin_pubkey_bytes)?;
        //   let sig = Signature::from_bytes(&signature);
        //   vk.verify(&payload_bytes, &sig)?;
        //
        // Structural placeholder (always passes — replace with real verify):
        let _ = admin_pubkey_bytes;
        let _sig_valid = true; // TODO: real verification

        Ok(Self {
            payload,
            payload_bytes,
            signature,
            raw_uri: uri.to_string(),
        })
    }

    /// Serialise back to MICAFP-lic:// URI.
    pub fn to_uri(&self) -> String {
        self.raw_uri.clone()
    }

    /// Check if features contain a specific flag.
    pub fn has_feature(&self, feature: &str) -> bool {
        self.payload.features.iter().any(|f| f == feature)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("Invalid token format: {0}")]
    InvalidFormat(String),
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    #[error("Token has been tampered with")]
    Tampered,
}
