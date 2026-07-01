//! Channel 6 — IPFS-fixed-CID-hardcoded-gateways
use async_trait::async_trait;
use tracing::debug;
use crate::MicafpError;
use super::{Channel, DpiLevel, RawToken, SignedToken, TransportType};

pub struct IpfsChannel;

#[async_trait]
impl Channel for IpfsChannel {
    fn id(&self) -> u8 { 6 }
    fn name(&self) -> &'static str { "IPFS-fixed-CID-hardcoded-gateways" }
    fn dpi_resistance(&self) -> DpiLevel { DpiLevel::Low }
    fn transport(&self) -> TransportType { TransportType::Tcp443 }
    async fn fetch_token(&self) -> Result<Option<RawToken>, MicafpError> {
        debug!("Channel-6 {}: fetch", self.name());
        Ok(None)
    }
    async fn publish_token(&self, _t: &SignedToken) -> Result<(), MicafpError> {
        debug!("Channel-6 {}: publish", self.name());
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_ch_6() { assert_eq!(IpfsChannel.id(), 6); }
}
