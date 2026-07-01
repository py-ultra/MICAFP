//! MICAFP v10.0 — Multi-Channel Internet Censorship Avoidance Fingerprint Protocol
//!
//! Complete production implementation with:
//!   - 11 redundant distribution channels (v5.0)
//!   - Smart resource manager: battery, thermal, network-aware (v6.0)
//!   - 9-layer anti-tamper engine (v7.0)
//!   - 9 advanced features: adaptive learning, revocation, ZK proofs (v8.0)
//!   - Full anti-DPI layer: ECH, JA3 cloning, H2 mimicry (v9.0)
//!   - 8 final features: HSM storage, canary tokens, PQC, dead man's switch (v10.0)
//!   - Seamless fallback: hot-swap tunnel on mid-session ISP block

pub mod channel;
pub mod token;
pub mod time;
pub mod hardware;
pub mod cache;
pub mod tamper;
pub mod resource;
pub mod antidpi;
pub mod secure_storage;
pub mod canary;
pub mod anomaly;
pub mod blackout;
pub mod deadman;
pub mod zk;
pub mod multikey;
pub mod engine;
pub mod error;

pub use error::MicafpError;
pub use engine::EngineConfig;
