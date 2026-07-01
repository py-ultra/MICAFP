//! ShadowTLS v3 Server — Proxy/Relay Switcher
//!
//! The server-side of ShadowTLS v3:
//!   1. Accept TCP connection
//!   2. Immediately proxy TLS handshake to real trusted server (e.g. apple.com)
//!      and relay both directions — client sees a real cert
//!   3. Monitor TLS application data for the HMAC authentication tag
//!   4. If tag verified → stop relaying to apple.com, switch to proxy handler
//!   5. If timeout with no valid tag → keep relaying apple.com traffic (active probe)

use std::net::SocketAddr;
use tracing::{debug, info, warn};

use super::hmac_auth::ShadowTlsKeyDeriver;

/// Server connection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerConnState {
    /// Proxying TLS handshake to trusted server.
    HandshakeRelay,
    /// Watching application data for auth tag.
    WatchingForAuth,
    /// Authenticated — in proxy mode.
    ProxyMode,
    /// Not authenticated — pure relay to trusted server (active probe defence).
    PureRelay,
}

/// Server-side ShadowTLS v3 connection handler.
pub struct ShadowTlsServerConn {
    client_addr: SocketAddr,
    state: ServerConnState,
    trusted_sni: String,
    key: [u8; 32],
    bytes_relayed: u64,
    bytes_proxied: u64,
}

impl ShadowTlsServerConn {
    pub fn new(client_addr: SocketAddr, password: &str, trusted_sni: &str) -> Self {
        let key = ShadowTlsKeyDeriver::derive_key(password, trusted_sni.as_bytes());
        Self {
            client_addr,
            state: ServerConnState::HandshakeRelay,
            trusted_sni: trusted_sni.to_string(),
            key,
            bytes_relayed: 0,
            bytes_proxied: 0,
        }
    }

    pub fn state(&self) -> ServerConnState { self.state }

    /// Process incoming data from client.
    /// Returns true if data should be forwarded to real proxy, false if relayed.
    pub fn process_client_data(&mut self, data: &[u8]) -> bool {
        match self.state {
            ServerConnState::HandshakeRelay => {
                // Still in handshake phase — relay everything
                self.bytes_relayed += data.len() as u64;
                false
            }
            ServerConnState::WatchingForAuth => {
                // Check if data starts with a valid auth token (nonce + tag = 64 bytes)
                if data.len() >= 64 {
                    let nonce: [u8; 32] = data[..32].try_into().unwrap();
                    let received: [u8; 32] = data[32..64].try_into().unwrap();
                    if ShadowTlsKeyDeriver::verify_tag(&self.key, &nonce, &[], &received) {
                        info!("ShadowTLS v3: client {} authenticated", self.client_addr);
                        self.state = ServerConnState::ProxyMode;
                        self.bytes_proxied += data.len() as u64;
                        return true; // Forward to proxy
                    }
                }
                // Not authenticated — pure relay mode
                warn!("ShadowTLS v3: no valid auth from {} — active probe?", self.client_addr);
                self.state = ServerConnState::PureRelay;
                self.bytes_relayed += data.len() as u64;
                false
            }
            ServerConnState::ProxyMode => {
                self.bytes_proxied += data.len() as u64;
                true // Forward to proxy
            }
            ServerConnState::PureRelay => {
                self.bytes_relayed += data.len() as u64;
                false // Relay to trusted server
            }
        }
    }

    /// Called when TLS handshake completes.
    pub fn on_handshake_complete(&mut self) {
        debug!("ShadowTLS v3: handshake complete for {}, watching for auth", self.client_addr);
        self.state = ServerConnState::WatchingForAuth;
    }

    pub fn bytes_relayed(&self) -> u64 { self.bytes_relayed }
    pub fn bytes_proxied(&self) -> u64 { self.bytes_proxied }
}
