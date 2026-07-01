// ─────────────────────────────────────────────────────────────────────────────
// MICAFP UnifiedShield VIP-ULTRA — Gossip Protocol (Epidemic Routing)
// Epidemic peer discovery: each node shares its peer list with neighbors.
// Converges to full mesh knowledge in O(log N) rounds.
// ─────────────────────────────────────────────────────────────────────────────

use std::collections::HashSet;
use std::time::Duration;
use tracing::debug;

/// Gossip message exchanged between mesh peers.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GossipMessage {
    pub sender_id: [u8; 32],
    pub known_peers: Vec<[u8; 32]>,
    pub timestamp_ms: u64,
    pub hop_count: u8,
    pub signature: [u8; 64],
}

/// Epidemic gossip protocol handler.
pub struct GossipProtocol {
    local_id: [u8; 32],
    fanout: usize,           // number of peers to gossip to per round
    ttl: u8,                 // max hop count
    seen_messages: parking_lot::Mutex<HashSet<[u8; 64]>>,
}

impl GossipProtocol {
    pub fn new(local_id: [u8; 32], fanout: usize, ttl: u8) -> Self {
        Self {
            local_id,
            fanout,
            ttl,
            seen_messages: parking_lot::Mutex::new(HashSet::new()),
        }
    }

    /// Create a gossip advertisement for the local node.
    pub fn create_advertisement(&self, known_peers: Vec<[u8; 32]>) -> GossipMessage {
        GossipMessage {
            sender_id: self.local_id,
            known_peers,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            hop_count: 0,
            signature: [0u8; 64],  // TODO: sign with device identity key
        }
    }

    /// Process an incoming gossip message. Returns the new peers to add,
    /// and a forwarded message (if TTL allows), or None if already seen.
    pub fn process(&self, mut msg: GossipMessage) -> Option<(Vec<[u8; 32]>, Option<GossipMessage>)> {
        // Dedup check
        if self.seen_messages.lock().contains(&msg.signature) {
            return None;
        }
        self.seen_messages.lock().insert(msg.signature);

        let new_peers = msg.known_peers
            .iter()
            .filter(|p| *p != &self.local_id)
            .copied()
            .collect::<Vec<_>>();

        // Forward if TTL allows
        let forward = if msg.hop_count < self.ttl {
            msg.hop_count += 1;
            Some(msg)
        } else {
            None
        };

        Some((new_peers, forward))
    }
}
