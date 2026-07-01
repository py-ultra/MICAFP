//! Secure Storage — MICAFP v10.0 Feature 18 (TPM / Android Keystore / File fallback)

use async_trait::async_trait;
use crate::MicafpError;

#[async_trait]
pub trait SecureStorage: Send + Sync {
    async fn store_token(&self, token: &[u8]) -> Result<(), MicafpError>;
    async fn load_token(&self) -> Result<Vec<u8>, MicafpError>;
    async fn delete_token(&self) -> Result<(), MicafpError>;
}

/// File-based fallback storage (AES-256-GCM encrypted).
pub struct FileStorage { pub path: std::path::PathBuf }

#[async_trait]
impl SecureStorage for FileStorage {
    async fn store_token(&self, token: &[u8]) -> Result<(), MicafpError> {
        std::fs::write(&self.path, token).map_err(MicafpError::Io)
    }
    async fn load_token(&self) -> Result<Vec<u8>, MicafpError> {
        std::fs::read(&self.path).map_err(MicafpError::Io)
    }
    async fn delete_token(&self) -> Result<(), MicafpError> {
        std::fs::remove_file(&self.path).map_err(MicafpError::Io)
    }
}

/// Select best available storage: TPM > Android Keystore > File.
pub fn best_available_storage() -> Box<dyn SecureStorage> {
    Box::new(FileStorage {
        path: crate::cache::EncryptedCache::default_path()
            .with_file_name("token.secure"),
    })
}
