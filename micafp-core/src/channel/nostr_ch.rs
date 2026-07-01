//! Channel 5 — Nostr-Protocol-WSS-50-relays
use async_trait::async_trait;
use tracing::debug;
use crate::MicafpError;
use super::{Channel, DpiLevel, RawToken, SignedToken, TransportType};

pub struct NostrChannel;

#[async_trait]
impl Channel for NostrChannel {
    fn id(&self) -> u8 { 5 }
    fn name(&self) -> &'static str { "Nostr-Protocol-WSS-50-relays" }
    fn dpi_resistance(&self) -> DpiLevel { DpiLevel::Low }
    fn transport(&self) -> TransportType { TransportType::Tcp443 }
    async fn fetch_token(&self) -> Result<Option<RawToken>, MicafpError> {
        debug!("Channel-5 {}: fetch", self.name());
        Ok(None)
    }
    async fn publish_token(&self, _t: &SignedToken) -> Result<(), MicafpError> {
        debug!("Channel-5 {}: publish", self.name());
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_ch_5() { assert_eq!(NostrChannel.id(), 5); }
}
