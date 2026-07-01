//! ShadowTLS v3 Handshake State Machine
//!
//! Tracks the state of a ShadowTLS v3 connection through its phases:
//!   Connecting → TlsHandshake → Authenticating → ProxyMode → Closed

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeState {
    /// TCP connection being established.
    Connecting,
    /// Real TLS 1.3 handshake in progress with trusted SNI.
    TlsHandshake,
    /// HMAC auth tag being sent and awaiting server acknowledgement.
    Authenticating,
    /// Fully authenticated — all data is proxy traffic.
    ProxyMode,
    /// Connection closed or failed.
    Closed,
}

impl fmt::Display for HandshakeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connecting      => write!(f, "Connecting"),
            Self::TlsHandshake    => write!(f, "TLS-Handshake"),
            Self::Authenticating  => write!(f, "Authenticating"),
            Self::ProxyMode       => write!(f, "Proxy-Mode"),
            Self::Closed          => write!(f, "Closed"),
        }
    }
}

pub struct HandshakeStateMachine {
    state: HandshakeState,
    bytes_sent: u64,
    bytes_received: u64,
}

impl HandshakeStateMachine {
    pub fn new() -> Self {
        Self { state: HandshakeState::Connecting, bytes_sent: 0, bytes_received: 0 }
    }
    pub fn state(&self) -> HandshakeState { self.state }
    pub fn advance(&mut self, next: HandshakeState) { self.state = next; }
    pub fn record_sent(&mut self, n: u64) { self.bytes_sent += n; }
    pub fn record_received(&mut self, n: u64) { self.bytes_received += n; }
    pub fn is_proxy_ready(&self) -> bool { self.state == HandshakeState::ProxyMode }
}
