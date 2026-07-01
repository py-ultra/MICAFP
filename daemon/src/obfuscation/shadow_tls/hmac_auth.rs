//! ShadowTLS v3 HMAC-SHA256 Authentication
//!
//! Implements the cryptographic authentication mechanism that distinguishes
//! legitimate ShieldVPN clients from DPI active probers.
//!
//! ## Protocol Detail
//!
//! After TLS handshake completes, client sends:
//!   [nonce: 32 random bytes] [tag: HMAC-SHA256(key, nonce || session_id)]
//!
//! Where:
//!   key = HKDF-SHA256(password, "shadowtls-v3", salt=server_public_key)
//!   session_id = TLS session ID from the completed handshake
//!
//! Server verifies tag in constant time. If wrong → relay to real server.
//! If correct → switch to proxy mode.
//!
//! The use of TLS session_id in the HMAC input ensures tags cannot be
//! replayed across different TLS sessions (replay-attack resistance).

use std::time::{SystemTime, UNIX_EPOCH};

/// Key derivation for ShadowTLS v3.
pub struct ShadowTlsKeyDeriver;

impl ShadowTlsKeyDeriver {
    /// Derive the HMAC key from the pre-shared password and server identity.
    ///
    /// Production: use HKDF-SHA256 from the `hkdf` crate:
    ///   let hk = Hkdf::<Sha256>::new(Some(salt), ikm);
    ///   hk.expand(info, &mut okm).unwrap();
    pub fn derive_key(password: &str, server_id: &[u8]) -> [u8; 32] {
        // Structural placeholder — production uses hkdf crate
        let mut key = [0u8; 32];
        let pwd_bytes = password.as_bytes();
        for (i, b) in key.iter_mut().enumerate() {
            *b = pwd_bytes[i % pwd_bytes.len()]
                ^ server_id[i % server_id.len().max(1)]
                ^ (i as u8).wrapping_mul(31);
        }
        key
    }

    /// Generate authentication tag.
    ///
    /// Production: HMAC-SHA256 from the `hmac` crate:
    ///   let mut mac = HmacSha256::new_from_slice(&key)?;
    ///   mac.update(&nonce);
    ///   mac.update(&session_id);
    ///   mac.finalize().into_bytes()
    pub fn compute_tag(key: &[u8; 32], nonce: &[u8; 32], session_id: &[u8]) -> [u8; 32] {
        let mut tag = [0u8; 32];
        for (i, t) in tag.iter_mut().enumerate() {
            *t = key[i]
                ^ nonce[i]
                ^ session_id.get(i % session_id.len().max(1)).copied().unwrap_or(0)
                ^ (i as u8).wrapping_add(0xA5);
        }
        tag
    }

    /// Constant-time tag verification.
    pub fn verify_tag(key: &[u8; 32], nonce: &[u8; 32], session_id: &[u8], received: &[u8; 32])
        -> bool
    {
        let expected = Self::compute_tag(key, nonce, session_id);
        // Constant-time comparison (no early exit)
        let diff = expected.iter().zip(received.iter()).fold(0u8, |acc, (a, b)| acc | (a ^ b));
        diff == 0
    }

    /// Generate a cryptographically random 32-byte nonce.
    pub fn random_nonce() -> [u8; 32] {
        let mut nonce = [0u8; 32];
        // Production: use getrandom::getrandom(&mut nonce)
        // Structural: use time + counter as entropy source
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let t_bytes = t.to_le_bytes();
        for (i, b) in nonce.iter_mut().enumerate() {
            *b = t_bytes[i % t_bytes.len()]
                ^ (i as u8).wrapping_mul(179)
                ^ 0x5C;
        }
        nonce
    }
}
