//! Encrypted Persistent Cache — MICAFP v7.0 Layer 5
use std::path::PathBuf;
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit, OsRng}};
use hkdf::Hkdf;
use sha2::Sha256;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;
use tracing::{debug, info, warn};
use crate::MicafpError;
use crate::channel::stats::ChannelStats;

const CACHE_MAGIC: &[u8] = b"MICAFP\x0A\x00";
const NONCE_LEN: usize = 12;
const SALT_LEN:  usize = 32;
const APP_SECRET: &[u8] = b"MICAFP-APP-SECRET-CHANGE-BEFORE-SHIPPING-v10";

#[derive(Debug, Clone, Serialize, Deserialize, Zeroize)]
pub struct TamperEvent {
    pub event_type: String,
    pub timestamp:  u64,
    pub detail:     String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Zeroize)]
pub struct CacheState {
    pub token:                    Option<String>,
    pub last_ntp:                 u64,
    pub last_accepted_seq:        u64,
    pub blackout_entered:         Option<u64>,
    pub tamper_log:               Vec<TamperEvent>,
    pub channel_stats:            Vec<ChannelStats>,
    pub network_confirmed_count:  u64,
    pub airplane_cycles:          u32,
    pub revoked_uids:             Vec<String>,
    pub admin_pubkeys:            Vec<String>,
    pub dead_mans_last_heartbeat: u64,
    pub cert_pin_sha256:          Option<String>,
    pub last_renewal_relay_ts:    u64,
}

impl Default for CacheState {
    fn default() -> Self {
        Self {
            token: None, last_ntp: 0, last_accepted_seq: 0,
            blackout_entered: None, tamper_log: vec![],
            channel_stats: crate::channel::ALL_CHANNEL_IDS.iter()
                .map(|&id| ChannelStats::new(id)).collect(),
            network_confirmed_count: 0, airplane_cycles: 0,
            revoked_uids: vec![], admin_pubkeys: vec![],
            dead_mans_last_heartbeat: 0, cert_pin_sha256: None,
            last_renewal_relay_ts: 0,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("cache not found")]
    NotFound,
    #[error("cache is tampered or corrupted")]
    Tampered,
    #[error("cache IO error: {0}")]
    Io(String),
    #[error("serialise error: {0}")]
    Serialise(String),
    #[error("encryption error: {0}")]
    Crypto(String),
}

pub struct EncryptedCache { path: PathBuf, key: [u8; 32] }

impl EncryptedCache {
    pub fn new(path: PathBuf, hid: &[u8; 32]) -> Self {
        Self { path, key: Self::derive_key(hid) }
    }

    pub fn default_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home).join(".local/share/micafp/cache.v10")
    }

    fn derive_key(hid: &[u8; 32]) -> [u8; 32] {
        let mut ikm = Vec::with_capacity(32 + APP_SECRET.len());
        ikm.extend_from_slice(hid);
        ikm.extend_from_slice(APP_SECRET);
        let hk = Hkdf::<Sha256>::new(None, &ikm);
        let mut okm = [0u8; 32];
        hk.expand(b"MICAFP-cache-v10", &mut okm).unwrap();
        ikm.zeroize();
        okm
    }

    pub fn load(&self) -> Result<CacheState, CacheError> {
        let data = std::fs::read(&self.path).map_err(|_| CacheError::NotFound)?;
        if !data.starts_with(CACHE_MAGIC) { return Err(CacheError::Tampered); }
        let h = CACHE_MAGIC.len();
        if data.len() < h + NONCE_LEN + SALT_LEN + 16 { return Err(CacheError::Tampered); }
        let nonce_bytes = &data[h .. h+NONCE_LEN];
        let ciphertext  = &data[h+NONCE_LEN+SALT_LEN ..];
        let cipher  = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let nonce   = Nonce::from_slice(nonce_bytes);
        let plain   = cipher.decrypt(nonce, ciphertext).map_err(|_| CacheError::Tampered)?;
        let state: CacheState = serde_json::from_slice(&plain)
            .map_err(|e| CacheError::Serialise(e.to_string()))?;
        debug!("Cache loaded: seq={}", state.last_accepted_seq);
        Ok(state)
    }

    pub fn save(&self, state: &CacheState) -> Result<(), CacheError> {
        let json = serde_json::to_vec(state)
            .map_err(|e| CacheError::Serialise(e.to_string()))?;
        let mut nonce_bytes = [0u8; NONCE_LEN];
        let mut salt_bytes  = [0u8; SALT_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        OsRng.fill_bytes(&mut salt_bytes);
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let nonce  = Nonce::from_slice(&nonce_bytes);
        let ct = cipher.encrypt(nonce, json.as_ref())
            .map_err(|e| CacheError::Crypto(e.to_string()))?;
        let mut out = Vec::with_capacity(CACHE_MAGIC.len()+NONCE_LEN+SALT_LEN+ct.len());
        out.extend_from_slice(CACHE_MAGIC);
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&salt_bytes);
        out.extend_from_slice(&ct);
        if let Some(p) = self.path.parent() { std::fs::create_dir_all(p).map_err(|e| CacheError::Io(e.to_string()))?; }
        std::fs::write(&self.path, &out).map_err(|e| CacheError::Io(e.to_string()))?;
        info!("Cache saved ({} bytes)", out.len());
        Ok(())
    }

    pub fn append_tamper_event(&self, evt: &str, detail: &str, ts: u64) -> Result<(), CacheError> {
        let mut s = self.load().unwrap_or_default();
        s.tamper_log.push(TamperEvent { event_type: evt.into(), timestamp: ts, detail: detail.into() });
        self.save(&s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_path() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("micafp-test-{}", rand::random::<u64>()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("cache.v10")
    }

    #[test]
    fn test_save_load_roundtrip() {
        let cache = EncryptedCache::new(tmp_path(), &[42u8; 32]);
        let mut s = CacheState::default();
        s.last_ntp = 1_700_000_000;
        s.last_accepted_seq = 7;
        cache.save(&s).unwrap();
        let loaded = cache.load().unwrap();
        assert_eq!(loaded.last_ntp, 1_700_000_000);
        assert_eq!(loaded.last_accepted_seq, 7);
    }

    #[test]
    fn test_tampered_cache_fails() {
        let path = tmp_path();
        let cache = EncryptedCache::new(path.clone(), &[1u8; 32]);
        cache.save(&CacheState::default()).unwrap();
        let mut data = std::fs::read(&path).unwrap();
        *data.last_mut().unwrap() ^= 0xFF;
        std::fs::write(&path, &data).unwrap();
        assert!(matches!(cache.load(), Err(CacheError::Tampered)));
    }
}
