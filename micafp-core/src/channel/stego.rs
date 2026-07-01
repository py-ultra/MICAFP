//! Channel 7 — LSB-steganography-over-GitHub-Gist-PNG
use async_trait::async_trait;
use tracing::debug;
use crate::MicafpError;
use super::{Channel, DpiLevel, RawToken, SignedToken, TransportType};

pub struct StegoChannel;

#[async_trait]
impl Channel for StegoChannel {
    fn id(&self) -> u8 { 7 }
    fn name(&self) -> &'static str { "LSB-steganography-over-GitHub-Gist-PNG" }
    fn dpi_resistance(&self) -> DpiLevel { DpiLevel::Zero }
    fn transport(&self) -> TransportType { TransportType::Tcp443 }
    async fn fetch_token(&self) -> Result<Option<RawToken>, MicafpError> {
        debug!("Channel-7 {}: fetch", self.name());
        Ok(None)
    }
    async fn publish_token(&self, _t: &SignedToken) -> Result<(), MicafpError> {
        debug!("Channel-7 {}: publish", self.name());
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_ch_7() { assert_eq!(StegoChannel.id(), 7); }
}
