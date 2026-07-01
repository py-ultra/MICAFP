// ─────────────────────────────────────────────────────────────────────────────
// MICAFP-UnifiedShield-vip-ultra-Quantum-ultra v8.0 — Error Types
// Complete unified error system for all 13 source projects.
// ─────────────────────────────────────────────────────────────────────────────

use thiserror::Error;

/// Structured error codes for every failure mode in UnifiedShield.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ErrorCode {
    // ── IPC Errors (1xxx) ────────────────────────────────────────────────────
    IpcConnectionFailed   = 1001,
    IpcChannelClosed      = 1002,
    IpcMessageParseError  = 1003,
    IpcTimeout            = 1004,

    // ── Transport Errors (2xxx) ──────────────────────────────────────────────
    TransportConnectionFailed = 2001,
    TransportTimeout          = 2002,
    AllTransportsExhausted    = 2003,
    DpiBlockDetected          = 2004,

    // ── Config Errors (3xxx) ─────────────────────────────────────────────────
    ConfigParseFailed     = 3001,
    ConfigNotFound        = 3002,
    ConfigUpdateFailed    = 3003,

    // ── Crypto Errors (4xxx) ─────────────────────────────────────────────────
    CryptoKeyExchangeFailed = 4001,
    CryptoSignatureInvalid  = 4002,
    CryptoDecryptionFailed  = 4003,
    CryptoPostQuantumFailed = 4004,

    // ── Anti-Forensics Errors (5xxx) ─────────────────────────────────────────
    AntiForensicsWipeFailed = 5001,
    AntiForensicsDeviceSecretCorrupted = 5002,

    // ── NAIN Errors (7xxx) ──────────────────────────────────────────────────
    NainCovertChannelFailed = 7001,
    NainAcousticChannelFailed = 7002,
    NainMode = 7003,

    // ── P2P Errors (8xxx) ──────────────────────────────────────────────────
    P2pI2pError = 8001,

    // ── AI / Inference Errors (6xxx) ─────────────────────────────────────────
    AiInferenceFailed     = 6001,
    AiModelNotFound       = 6002,
    AiTransportSelectionFailed = 6003,

    // ── Unknown / Generic ────────────────────────────────────────────────────
    Unknown               = 9999,
}

impl ErrorCode {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

/// Structured IPC error payload sent to the UI.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IpcErrorResponse {
    pub code: i32,
    pub message: String,
    pub category: String,
    pub source: Option<String>,
    pub timestamp_ms: u64,
}

/// The unified error type for the ShieldDaemon.
/// Uses a variant-based approach with fallback to generic error strings
/// for extensibility across all 13 merged projects.
#[derive(Debug, Clone, Error)]
pub enum ShieldError {
    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Crypto (post-quantum) error: {0}")]
    CryptoPostQuantum(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("IPC error [{code:?}]: {message}")]
    Ipc { code: ErrorCode, message: String },

    #[error("IO error: {0}")]
    Io(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Connection refused: {0}")]
    ConnectionRefused(String),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("All transports exhausted: {0}")]
    AllTransportsExhausted(String),

    #[error("All endpoints exhausted: {0}")]
    AllEndpointsExhausted(String),

    #[error("NAIN detected — switching to covert channel: {0}")]
    NainDetected(String),

    #[error("NAIN mode error: {0}")]
    NainMode(String),

    #[error("DPI block detected — triggering failover: {0}")]
    DpiBlock(String),

    #[error("QUIC error: {0}")]
    QuicError(String),

    #[error("TLS handshake failed: {0}")]
    TlsHandshakeFailed(String),

    #[error("DNS resolution failed: {0}")]
    DnsResolutionFailed(String),

    #[error("ICMP error: {0}")]
    IcmpError(String),

    #[error("P2P error: {0}")]
    P2p(String),

    #[error("P2P/I2P error: {0}")]
    P2pI2pError(String),

    #[error("AI error: {0}")]
    Ai(String),

    #[error("AI transport selection failed: {0}")]
    AiTransportSelectionFailed(String),

    #[error("Anti-forensics error: {0}")]
    AntiForensics(String),

    #[error("Anti-forensics device secret corrupted")]
    AntiForensicsDeviceSecretCorrupted,

    #[error("NAIN covert channel failed: {0}")]
    NainCovertChannelFailed(String),

    #[error("NAIN acoustic channel failed: {0}")]
    NainAcousticChannelFailed(String),

    #[error("CDN worker error: {0}")]
    CdnWorkerError(String),

    #[error("Endpoint unreachable")]
    EndpointUnreachable,

    #[error("Transport unavailable")]
    TransportUnavailable,

    #[error("Rate limited")]
    RateLimited,

    #[error("Quantum key exchange failed: {0}")]
    QuantumKex(String),

    #[error("Peer exchange failed: {0}")]
    PeerExchange(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl ShieldError {
    /// Create an IPC-category error.
    pub fn ipc(code: ErrorCode, message: impl Into<String>) -> Self {
        ShieldError::Ipc { code, message: message.into() }
    }

    /// Create a config-category error.
    pub fn config(message: impl Into<String>) -> Self {
        ShieldError::Config(message.into())
    }

    /// Create a transport-category error.
    pub fn transport(message: impl Into<String>) -> Self {
        ShieldError::Transport(message.into())
    }

    /// Create a crypto-category error.
    pub fn crypto(message: impl Into<String>) -> Self {
        ShieldError::Crypto(message.into())
    }

    /// Create a crypto-category error with source info.
    pub fn crypto_with_source(message: impl Into<String>) -> Self {
        ShieldError::Crypto(message.into())
    }

    /// Create a NAIN mode error.
    pub fn nain_mode(code: ErrorCode, message: impl Into<String>) -> Self {
        ShieldError::Ipc { code, message: message.into() }
    }

    /// Create a P2P error.
    pub fn p2p(code: ErrorCode, message: impl Into<String>) -> Self {
        ShieldError::Ipc { code, message: message.into() }
    }

    /// Create an AI error.
    pub fn ai(code: ErrorCode, message: impl Into<String>) -> Self {
        ShieldError::Ipc { code, message: message.into() }
    }

    /// Create an anti-forensics error.
    pub fn anti_forensics(code: ErrorCode, message: impl Into<String>) -> Self {
        ShieldError::Ipc { code, message: message.into() }
    }

    /// Create an error from code and message.
    pub fn from_code(code: ErrorCode, message: &str) -> Self {
        ShieldError::Ipc { code, message: message.to_string() }
    }
}

impl From<anyhow::Error> for ShieldError {
    fn from(e: anyhow::Error) -> Self {
        ShieldError::Unknown(e.to_string())
    }
}

impl From<std::io::Error> for ShieldError {
    fn from(e: std::io::Error) -> Self {
        ShieldError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for ShieldError {
    fn from(e: serde_json::Error) -> Self {
        ShieldError::Serialization(e.to_string())
    }
}
