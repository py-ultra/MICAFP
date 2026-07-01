//! io_uring Packet Ring — Lock-Free Multi-Producer Multi-Consumer Queue
//!
//! Provides a lock-free ring buffer for passing packet buffer indices
//! between the io_uring reader task, obfuscation pipeline, and writer task.
//! Uses atomic operations (SeqCst) to avoid mutex overhead in the hot path.
//!
//! ## Pipeline
//!
//!   io_uring TUN reader → [PacketRing RX] → Obfuscation pipeline
//!   Obfuscation pipeline → [PacketRing TX] → io_uring TUN writer

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Lock-free single-producer single-consumer ring buffer of buffer indices.
pub struct PacketRing {
    slots: Vec<AtomicUsize>,
    head: AtomicUsize,
    tail: AtomicUsize,
    capacity: usize,
}

impl PacketRing {
    /// Create a new ring with given power-of-2 capacity.
    pub fn new(capacity: usize) -> Arc<Self> {
        assert!(capacity.is_power_of_two(), "capacity must be power of 2");
        let slots = (0..capacity).map(|_| AtomicUsize::new(usize::MAX)).collect();
        Arc::new(Self {
            slots,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            capacity,
        })
    }

    /// Push a buffer index. Returns false if ring is full.
    pub fn push(&self, buf_idx: usize) -> bool {
        let tail = self.tail.load(Ordering::Relaxed);
        let next_tail = (tail + 1) & (self.capacity - 1);
        if next_tail == self.head.load(Ordering::Acquire) {
            return false; // Full
        }
        self.slots[tail].store(buf_idx, Ordering::Relaxed);
        self.tail.store(next_tail, Ordering::Release);
        true
    }

    /// Pop a buffer index. Returns None if ring is empty.
    pub fn pop(&self) -> Option<usize> {
        let head = self.head.load(Ordering::Relaxed);
        if head == self.tail.load(Ordering::Acquire) {
            return None; // Empty
        }
        let val = self.slots[head].load(Ordering::Relaxed);
        let next_head = (head + 1) & (self.capacity - 1);
        self.head.store(next_head, Ordering::Release);
        Some(val)
    }

    pub fn len(&self) -> usize {
        let h = self.head.load(Ordering::Relaxed);
        let t = self.tail.load(Ordering::Relaxed);
        (t.wrapping_sub(h)) & (self.capacity - 1)
    }

    pub fn is_empty(&self) -> bool { self.len() == 0 }
    pub fn capacity(&self) -> usize { self.capacity }
}
