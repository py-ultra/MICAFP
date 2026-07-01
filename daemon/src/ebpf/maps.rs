//! eBPF Maps — Shared State Between Kernel and Userspace
//!
//! eBPF maps are key-value stores shared between eBPF programs
//! running in the kernel and userspace (this Rust daemon).
//! The daemon writes configuration (per-ISP fragment strategies,
//! blocklists, etc.) into maps; the eBPF programs read them at
//! packet-processing time with near-zero overhead.

/// Available eBPF map types used by UnifiedShield.
#[derive(Debug, Clone, Copy)]
pub enum BpfMapType {
    /// Per-flow fragment configuration (HashMap).
    FlowFragConfig,
    /// IP blocklist for active probing defence (LPM trie).
    BlockedIpTrie,
    /// ISP-to-strategy mapping (array indexed by ISP ID hash).
    IspStrategyArray,
    /// Per-CPU packet counters for metrics (PercpuArray).
    PacketCounters,
    /// Ring buffer for events from BPF to userspace.
    EventRingBuf,
}

/// Map descriptor.
#[derive(Debug)]
pub struct BpfMapDesc {
    pub map_type: BpfMapType,
    pub name: &'static str,
    pub max_entries: u32,
    pub fd: Option<i32>,
}

impl BpfMapDesc {
    pub fn new(map_type: BpfMapType, name: &'static str, max_entries: u32) -> Self {
        Self { map_type, name, max_entries, fd: None }
    }
    pub fn is_open(&self) -> bool { self.fd.is_some() }
}

/// All eBPF maps used by UnifiedShield.
pub fn all_maps() -> Vec<BpfMapDesc> {
    vec![
        BpfMapDesc::new(BpfMapType::FlowFragConfig,   "flow_frag_config",   65536),
        BpfMapDesc::new(BpfMapType::BlockedIpTrie,    "blocked_ips",        4096),
        BpfMapDesc::new(BpfMapType::IspStrategyArray, "isp_strategies",     64),
        BpfMapDesc::new(BpfMapType::PacketCounters,   "pkt_counters",       256),
        BpfMapDesc::new(BpfMapType::EventRingBuf,     "events",             1),
    ]
}
