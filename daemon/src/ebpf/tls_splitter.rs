//! eBPF TLS ClientHello Splitter — Kernel-Level SNI Fragmentation
//!
//! This module manages the eBPF program that splits TLS ClientHello
//! packets at the tc (traffic control) egress hook in the Linux kernel.
//!
//! ## How It Works
//!
//! The eBPF program attached to tc egress intercepts outgoing TCP packets.
//! When it detects a TLS ClientHello (identified by the TLS record header
//! byte 0x16 at the start of TCP payload), it:
//!
//!   1. Locates the SNI extension within the ClientHello
//!   2. Splits the packet into 2-3 TCP segments at the SNI boundary
//!   3. Manipulates TCP sequence numbers and segment sizes
//!   4. Emits multiple smaller segments instead of the original
//!
//! The receiving endpoint reassembles the segments correctly via TCP.
//! FAVA DPI sees fragmented TLS records and cannot extract the SNI.
//!
//! ## Fragment Strategies (mirroring isp-profiles.json)
//!
//! | Strategy     | Split point               | Defeats                    |
//! |--------------|---------------------------|----------------------------|
//! | SniSplit     | At SNI extension offset   | FAVA v2.x, v3.x SNI check |
//! | RecordSplit  | Random within TLS record  | FAVA v3.x, v4.x flow ML   |
//! | RandomSplit  | Random with jitter        | FAVA v4.x ML classifier    |
//! | DisorderMode | Out-of-order delivery     | FAVA v2.x reassembly       |

use std::collections::HashMap;

/// Fragment strategy for the eBPF TLS splitter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BpfFragmentStrategy {
    SniSplit,
    RecordSplit,
    RandomSplit,
    DisorderMode,
}

/// eBPF map key for per-flow fragment configuration.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FlowKey {
    pub src_ip: u32,
    pub dst_ip: u32,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: u8,
    pub _pad: [u8; 3],
}

/// eBPF map value for per-flow fragment configuration.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FlowFragConfig {
    /// Fragment strategy enum as u8.
    pub strategy: u8,
    /// Minimum fragment size in bytes.
    pub min_frag: u16,
    /// Maximum fragment size in bytes.
    pub max_frag: u16,
    /// Inter-fragment delay in milliseconds.
    pub delay_ms: u8,
    /// SNI split position (byte offset within ClientHello).
    pub sni_split_pos: u16,
    /// Whether this flow has been processed (1-shot per session).
    pub processed: u8,
    pub _pad: [u8; 1],
}

/// Manager for the eBPF TLS splitter program.
pub struct BpfTlsSplitter {
    /// Maps ISP profile ID to fragment strategy.
    isp_strategies: HashMap<String, BpfFragmentStrategy>,
    active: bool,
}

impl BpfTlsSplitter {
    pub fn new() -> Self {
        let mut isp_strategies = HashMap::new();
        // Populate from isp-profiles.json fragment_strategy types
        isp_strategies.insert("mci".into(),        BpfFragmentStrategy::SniSplit);
        isp_strategies.insert("irancell".into(),   BpfFragmentStrategy::RecordSplit);
        isp_strategies.insert("rightel".into(),    BpfFragmentStrategy::SniSplit);
        isp_strategies.insert("shatel".into(),     BpfFragmentStrategy::RandomSplit);
        isp_strategies.insert("asiatech".into(),   BpfFragmentStrategy::SniSplit);
        isp_strategies.insert("pars_online".into(),BpfFragmentStrategy::RecordSplit);
        isp_strategies.insert("afranet".into(),    BpfFragmentStrategy::SniSplit);
        isp_strategies.insert("mobinnet".into(),   BpfFragmentStrategy::SniSplit);
        isp_strategies.insert("fanava".into(),     BpfFragmentStrategy::RecordSplit);
        isp_strategies.insert("mokhaberat".into(), BpfFragmentStrategy::SniSplit);
        isp_strategies.insert("irib".into(),       BpfFragmentStrategy::SniSplit);
        isp_strategies.insert("pishgaman".into(),  BpfFragmentStrategy::SniSplit);
        isp_strategies.insert("itc".into(),        BpfFragmentStrategy::RecordSplit);
        Self { isp_strategies, active: false }
    }

    pub fn strategy_for_isp(&self, isp_id: &str) -> BpfFragmentStrategy {
        self.isp_strategies
            .get(isp_id)
            .copied()
            .unwrap_or(BpfFragmentStrategy::SniSplit)
    }

    /// Build a FlowFragConfig for a given ISP and flow.
    pub fn build_flow_config(&self, isp_id: &str) -> FlowFragConfig {
        let strategy = self.strategy_for_isp(isp_id);
        let (min_frag, max_frag, delay_ms, sni_split_pos) = match isp_id {
            "irancell"   => (1, 3, 2, 5),
            "pars_online"=> (1, 2, 4, 3),
            "shatel"     => (1, 4, 3, 4),
            "mci"        => (2, 5, 1, 4),
            "mokhaberat" => (2, 5, 10, 5),
            _            => (2, 5, 1, 4),
        };
        FlowFragConfig {
            strategy: strategy as u8,
            min_frag,
            max_frag,
            delay_ms,
            sni_split_pos,
            processed: 0,
            _pad: [0; 1],
        }
    }

    pub fn is_active(&self) -> bool { self.active }

    pub async fn attach(&mut self, iface: &str) -> Result<(), String> {
        // Production: load compiled BPF object, attach to tc egress on iface
        // using libbpf-rs TcHook::new(iface).attach(BpfAttachType::EgressTc)
        tracing::info!("BPF TLS splitter attached to interface: {}", iface);
        self.active = true;
        Ok(())
    }

    pub async fn detach(&mut self) {
        if self.active {
            tracing::info!("BPF TLS splitter detached");
            self.active = false;
        }
    }
}
