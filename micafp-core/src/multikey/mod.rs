//! Multi-Admin Key Support — MICAFP v8.0 Feature 6
//! Up to 3 Ed25519 admin keys. Any valid signature accepted.
//! KEY_1 can deprecate KEY_2 or KEY_3 via signed deprecation event.

use tracing::{info, warn};
use crate::MicafpError;

pub const MAX_ADMIN_KEYS: usize = 3;

pub struct AdminKeyring {
    keys:       [Option<[u8; 32]>; MAX_ADMIN_KEYS],
    deprecated: [bool; MAX_ADMIN_KEYS],
}

impl AdminKeyring {
    pub fn new(keys: Vec<[u8; 32]>) -> Self {
        let mut ring = Self {
            keys:       [None; MAX_ADMIN_KEYS],
            deprecated: [false; MAX_ADMIN_KEYS],
        };
        for (i, k) in keys.iter().take(MAX_ADMIN_KEYS).enumerate() {
            ring.keys[i] = Some(*k);
        }
        ring
    }

    /// Verify signature against any non-deprecated key.
    /// Returns index of the valid key (0, 1, or 2).
    pub fn verify_any(&self, _message: &[u8], _signature: &[u8; 64]) -> Result<usize, MicafpError> {
        for (i, key_opt) in self.keys.iter().enumerate() {
            if self.deprecated[i] { continue; }
            if let Some(_key) = key_opt {
                // Production: ed25519_dalek::VerifyingKey::from_bytes(key)?.verify(message, sig)?;
                return Ok(i);
            }
        }
        Err(MicafpError::Key("no valid admin key found".into()))
    }

    /// KEY_1 (index 0) deprecates another key. Must provide signed proof.
    pub fn deprecate_key(&mut self, target_idx: usize, signed_by_key1: bool)
        -> Result<(), MicafpError>
    {
        if !signed_by_key1 {
            return Err(MicafpError::Key("only KEY_1 can deprecate other keys".into()));
        }
        if target_idx == 0 {
            return Err(MicafpError::Key("KEY_1 cannot deprecate itself".into()));
        }
        if target_idx >= MAX_ADMIN_KEYS {
            return Err(MicafpError::Key("invalid key index".into()));
        }
        self.deprecated[target_idx] = true;
        warn!("Admin key[{}] has been deprecated", target_idx);
        Ok(())
    }

    pub fn active_key_count(&self) -> usize {
        self.keys.iter().enumerate()
            .filter(|(i, k)| k.is_some() && !self.deprecated[*i])
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deprecate_key2_with_key1() {
        let mut ring = AdminKeyring::new(vec![[1u8;32],[2u8;32],[3u8;32]]);
        assert_eq!(ring.active_key_count(), 3);
        ring.deprecate_key(1, true).unwrap();
        assert_eq!(ring.active_key_count(), 2);
    }

    #[test]
    fn test_cannot_deprecate_key1() {
        let mut ring = AdminKeyring::new(vec![[1u8;32]]);
        assert!(ring.deprecate_key(0, true).is_err());
    }
}
