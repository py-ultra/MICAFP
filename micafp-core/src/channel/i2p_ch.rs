//! Channel 8 — I2P-pure-UDP-overlay-network
use async_trait::async_trait;
use tracing::debug;
use crate::MicafpError;
use super::{Channel, DpiLevel, RawToken, SignedToken, TransportType};

pub struct I2pChannel;

#[async_trait]
impl Channel for I2pChannel {
    fn id(&self) -> u8 { 8 }
    fn name(&self) -> &'static str { "I2P-pure-UDP-overlay-network" }
    fn dpi_resistance(&self) -> DpiLevel { DpiLevel::NearZero }
    fn transport(&self) -> TransportType { TransportType::UdpI2p }
    async fn fetch_token(&self) -> Result<Option<RawToken>, MicafpError> {
        debug!("Channel-8 {}: fetch", self.name());
        Ok(None)
    }
    async fn publish_token(&self, _t: &SignedToken) -> Result<(), MicafpError> {
        debug!("Channel-8 {}: publish", self.name());
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_ch_8() { assert_eq!(I2pChannel.id(), 8); }
}
