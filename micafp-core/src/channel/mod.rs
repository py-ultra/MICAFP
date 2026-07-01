//! MICAFP v10.0 — 10 Redundant Distribution Channels
pub mod dns_txt;
pub mod meek_lite;
pub mod snowflake;
pub mod dht_bep44;
pub mod nostr_ch;
pub mod ipfs_ch;
pub mod stego;
pub mod i2p_ch;
pub mod ssb_ch;
pub mod gnunet_ch;
pub mod stats;
pub mod runner;

use async_trait::async_trait;
use crate::MicafpError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DpiLevel { Zero, Low, NearZero }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportType { UdpDns, Tcp443, WebRtc, UdpDht, UdpI2p, TcpLan, UdpGnu }

pub type RawToken = String;
pub type SignedToken = String;

#[async_trait]
pub trait Channel: Send + Sync {
    fn id(&self) -> u8;
    fn name(&self) -> &'static str;
    fn dpi_resistance(&self) -> DpiLevel;
    fn transport(&self) -> TransportType;
    async fn fetch_token(&self) -> Result<Option<RawToken>, MicafpError>;
    async fn publish_token(&self, token: &SignedToken) -> Result<(), MicafpError>;
}

pub const ALL_CHANNEL_IDS:    &[u8] = &[1,2,3,4,5,6,7,8,9,10];
pub const TOP5_CHANNELS:      &[u8] = &[1,2,3,4,7];
pub const TOP2_CHANNELS:      &[u8] = &[1,2];
pub const MOBILE_DATA_CHANNELS:&[u8]= &[1,2,3,4,5,6,7];
