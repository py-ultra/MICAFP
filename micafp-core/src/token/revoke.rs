//! Emergency Revocation — MICAFP v8.0 Feature 5
//! Format: MICAFP-rev://<base64url({ uid, rev_at } || Ed25519_sig)>

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD as B64};
use serde::{Deserialize, Serialize};
use crate::MicafpError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeEvent {
    pub uid:    String,
    pub rev_at: u64,
}

pub fn parse_revoke_token(raw: &str) -> Result<RevokeEvent, MicafpError> {
    let body = raw.strip_prefix("MICAFP-rev://")
        .ok_or_else(|| MicafpError::Token("missing MICAFP-rev:// prefix".into()))?;
    let dot = body.rfind('.')
        .ok_or_else(|| MicafpError::Token("missing '.' in revoke token".into()))?;
    let payload_bytes = B64.decode(&body[..dot])
        .map_err(|e| MicafpError::Token(e.to_string()))?;
    let event: RevokeEvent = serde_json::from_slice(&payload_bytes)
        .map_err(|e| MicafpError::Token(e.to_string()))?;
    Ok(event)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_revoke_bad_prefix() {
        assert!(parse_revoke_token("WRONG://data").is_err());
    }
}
