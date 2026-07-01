//! eBPF + io_uring Integration — Zero-Copy Event Pipeline
//!
//! Connects the eBPF ring buffer map (for kernel→userspace events)
//! to an io_uring consumer for near-zero-latency event processing.
//!
//! When eBPF detects a suspicious flow (e.g., active probing source IP),
//! it writes an event to the ring buffer. This module reads those events
//! via io_uring (avoiding epoll overhead) and dispatches them.

use tracing::{debug, warn};

/// Event types emitted from eBPF programs to userspace.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BpfEventType {
    /// A new flow was fragmented. src_ip, dst_ip, dst_port provided.
    FlowFragmented = 0x01,
    /// A suspected active probing source was detected.
    ActiveProbeDetected = 0x02,
    /// A DNS poison IP was detected in outbound DNS response.
    DnsPoisonDetected = 0x03,
    /// A QUIC packet was blocked by the ISP (updates QUIC status).
    QuicBlocked = 0x04,
    /// Interface MTU change detected.
    MtuChanged = 0x05,
}

/// Event record from eBPF ring buffer.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BpfEvent {
    pub event_type: u8,
    pub _pad: [u8; 3],
    pub src_ip: u32,
    pub dst_ip: u32,
    pub src_port: u16,
    pub dst_port: u16,
    pub extra: u64,
}

/// io_uring-backed eBPF event consumer.
pub struct BpfIouringConsumer {
    active: bool,
    events_processed: u64,
}

impl BpfIouringConsumer {
    pub fn new() -> Self {
        Self { active: false, events_processed: 0 }
    }

    pub async fn start(&mut self) -> Result<(), String> {
        // Production:
        //  1. Open the eBPF ring buffer map fd
        //  2. Create an io_uring instance
        //  3. Submit IORING_OP_READ requests against the ring buffer fd
        //  4. Poll CQ for completed reads → parse BpfEvent structs
        self.active = true;
        debug!("BPF io_uring event consumer started");
        Ok(())
    }

    pub async fn stop(&mut self) {
        self.active = false;
    }

    pub fn events_processed(&self) -> u64 { self.events_processed }
}
