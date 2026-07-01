//! Channel 9 — SSB-LAN-gossip-survives-internet-blackout
use async_trait::async_trait;
use tracing::debug;
use crate::MicafpError;
use super::{Channel, DpiLevel, RawToken, SignedToken, TransportType};

pub struct SsbChannel;

#[async_trait]
impl Channel for SsbChannel {
    fn id(&self) -> u8 { 9 }
    fn name(&self) -> &'static str { "SSB-LAN-gossip-survives-internet-blackout" }
    fn dpi_resistance(&self) -> DpiLevel { DpiLevel::Zero }
    fn transport(&self) -> TransportType { TransportType::TcpLan }
    async fn fetch_token(&self) -> Result<Option<RawToken>, MicafpError> {
        debug!("Channel-9 {}: fetch", self.name());
        Ok(None)
    }
    async fn publish_token(&self, _t: &SignedToken) -> Result<(), MicafpError> {
        debug!("Channel-9 {}: publish", self.name());
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_ch_9() { assert_eq!(SsbChannel.id(), 9); }
}
