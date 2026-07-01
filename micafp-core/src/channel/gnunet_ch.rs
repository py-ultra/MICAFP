//! Channel 10 — GNUnet-no-central-bootstrap-pure-UDP
use async_trait::async_trait;
use tracing::debug;
use crate::MicafpError;
use super::{Channel, DpiLevel, RawToken, SignedToken, TransportType};

pub struct GnunetChannel;

#[async_trait]
impl Channel for GnunetChannel {
    fn id(&self) -> u8 { 10 }
    fn name(&self) -> &'static str { "GNUnet-no-central-bootstrap-pure-UDP" }
    fn dpi_resistance(&self) -> DpiLevel { DpiLevel::NearZero }
    fn transport(&self) -> TransportType { TransportType::UdpGnu }
    async fn fetch_token(&self) -> Result<Option<RawToken>, MicafpError> {
        debug!("Channel-10 {}: fetch", self.name());
        Ok(None)
    }
    async fn publish_token(&self, _t: &SignedToken) -> Result<(), MicafpError> {
        debug!("Channel-10 {}: publish", self.name());
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_ch_10() { assert_eq!(GnunetChannel.id(), 10); }
}
