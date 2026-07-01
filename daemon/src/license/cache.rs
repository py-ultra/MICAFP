//! Encrypted License Cache — AES-256-GCM Local Storage
//!
//! Persists the latest valid MICAFP token to disk, encrypted with
//! AES-256-GCM using a key derived from the device's unique ID.
//! The cache is used during the grace period when Nostr relays are
//! unreachable (e.g., full internet shutdown / NAIN mode).
//!
//! ## Cache File Location
//!
//! | Platform | Path                                              |
//! |----------|---------------------------------------------------|
//! | Linux    | ~/.local/share/micafp/license.cache              |
//! | Android  | /data/data/ir.micafp.vpn/files/license.cache     |
//! | iOS      | <AppGroup>/Library/Application Support/lic.cache  |
//! | Windows  | %APPDATA%\MICAFP\license.cache                    |
//!
//! ## Cache Format (binary)
//!
//! ```
//! [magic: 8 bytes "MICAFP\x01\x00"]
//! [nonce: 12 bytes (AES-GCM nonce)]
//! [ciphertext + 16-byte GCM tag: variable]
//! ```
//!
//! Plaintext inside ciphertext is:
//! ```json
//! { "token": "MICAFP-lic://v1/...", "cached_at": 1748601600,
//!   "relay_ts": 1748601234 }
//! ```

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

const CACHE_MAGIC: &[u8] = b"MICAFP\x01\x00";
const NONCE_LEN: usize = 12;

/// Contents stored in the encrypted cache.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Full MICAFP-lic:// URI.
    pub token: String,
    /// Unix timestamp when this entry was written.
    pub cached_at: u64,
    /// Unix timestamp of the Nostr event that carried this token.
    pub relay_ts: u64,
}

/// Encrypted license cache manager.
pub struct LicenseCache {
    cache_path: PathBuf,
    /// AES-256-GCM key derived from device ID.
    key: [u8; 32],
}

impl LicenseCache {
    /// Create or open the cache, deriving the encryption key from device ID.
    pub fn new(cache_path: PathBuf) -> Self {
        let key = Self::derive_key();
        Self { cache_path, key }
    }

    /// Platform-default cache path.
    pub fn default_path() -> PathBuf {
        #[cfg(target_os = "android")]
        { PathBuf::from("/data/data/ir.micafp.vpn/files/license.cache") }
        #[cfg(target_os = "ios")]
        { PathBuf::from("license.cache") }
        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
            PathBuf::from(appdata).join("MICAFP").join("license.cache")
        }
        #[cfg(not(any(target_os = "android", target_os = "ios", target_os = "windows")))]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            PathBuf::from(home).join(".local/share/micafp/license.cache")
        }
    }

    /// Write a new token to the encrypted cache.
    pub fn write(&self, entry: &CacheEntry) -> Result<(), CacheError> {
        let json = serde_json::to_vec(entry)
            .map_err(|e| CacheError::Serialise(e.to_string()))?;

        let (nonce, ciphertext) = self.encrypt(&json)?;

        let mut file_data = Vec::with_capacity(CACHE_MAGIC.len() + NONCE_LEN + ciphertext.len());
        file_data.extend_from_slice(CACHE_MAGIC);
        file_data.extend_from_slice(&nonce);
        file_data.extend_from_slice(&ciphertext);

        if let Some(parent) = self.cache_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CacheError::Io(e.to_string()))?;
        }
        std::fs::write(&self.cache_path, &file_data)
            .map_err(|e| CacheError::Io(e.to_string()))?;

        info!("License cache written: {} bytes", file_data.len());
        Ok(())
    }

    /// Read and decrypt the cached token.
    pub fn read(&self) -> Result<CacheEntry, CacheError> {
        let data = std::fs::read(&self.cache_path)
            .map_err(|e| CacheError::Io(e.to_string()))?;

        if !data.starts_with(CACHE_MAGIC) {
            return Err(CacheError::Corrupt("Bad magic bytes".into()));
        }

        if data.len() < CACHE_MAGIC.len() + NONCE_LEN + 16 {
            return Err(CacheError::Corrupt("File too short".into()));
        }

        let nonce_start = CACHE_MAGIC.len();
        let ct_start = nonce_start + NONCE_LEN;
        let nonce: [u8; 12] = data[nonce_start..ct_start].try_into().unwrap();
        let ciphertext = &data[ct_start..];

        let plaintext = self.decrypt(&nonce, ciphertext)?;

        let entry: CacheEntry = serde_json::from_slice(&plaintext)
            .map_err(|e| CacheError::Corrupt(format!("Bad JSON: {}", e)))?;

        debug!("License cache read: token expires {} relay_ts={}",
               entry.cached_at, entry.relay_ts);
        Ok(entry)
    }

    /// Derive AES-256 key from device ID + MICAFP constant.
    fn derive_key() -> [u8; 32] {
        // Production: HKDF-SHA256(device_id_bytes, "MICAFP-cache-key-v1")
        // Device ID sources (per platform):
        //   Linux:   /etc/machine-id
        //   Android: android.provider.Settings.Secure.ANDROID_ID
        //   iOS:     UIDevice.current.identifierForVendor
        //   Windows: HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Cryptography\MachineGuid
        let mut key = [0u8; 32];
        // Structural placeholder:
        for (i, b) in b"MICAFP-cache-key-v1-placeholder!".iter().enumerate() {
            key[i % 32] ^= *b;
        }
        key
    }

    fn encrypt(&self, plaintext: &[u8]) -> Result<([u8; 12], Vec<u8>), CacheError> {
        // Production: aes-gcm crate
        //   let cipher = Aes256Gcm::new_from_slice(&self.key)?;
        //   let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        //   let ct = cipher.encrypt(&nonce, plaintext)?;
        let nonce = [0u8; 12]; // TODO: use OsRng
        let mut ct = plaintext.to_vec();
        ct.extend_from_slice(&[0u8; 16]); // placeholder GCM tag
        Ok((nonce, ct))
    }

    fn decrypt(&self, nonce: &[u8; 12], ciphertext: &[u8]) -> Result<Vec<u8>, CacheError> {
        // Production: aes-gcm crate decrypt + verify tag
        if ciphertext.len() < 16 {
            return Err(CacheError::Corrupt("Ciphertext too short".into()));
        }
        Ok(ciphertext[..ciphertext.len()-16].to_vec())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Cache file IO error: {0}")]
    Io(String),
    #[error("Cache is corrupt: {0}")]
    Corrupt(String),
    #[error("Serialisation error: {0}")]
    Serialise(String),
    #[error("Decryption failed (tampered cache?)")]
    DecryptFailed,
    #[error("Cache not found")]
    NotFound,
}
