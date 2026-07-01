//! Unified error type for MICAFP v10.0

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MicafpError {
    #[error("channel error: {0}")]
    Channel(String),
    #[error("token error: {0}")]
    Token(String),
    #[error("time error: {0}")]
    Time(String),
    #[error("cache error: {0}")]
    Cache(String),
    #[error("tamper detected: {0}")]
    Tamper(String),
    #[error("hardware error: {0}")]
    Hardware(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("zk proof error: {0}")]
    Zk(String),
    #[error("key error: {0}")]
    Key(String),
    #[error("blackout: {0}")]
    Blackout(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialise: {0}")]
    Serialise(#[from] serde_json::Error),
}
