//! io_uring Zero-Copy Buffer Management
//!
//! Manages a pool of registered buffers shared between userspace and
//! the kernel via io_uring's IORING_REGISTER_BUFFERS. Once registered,
//! the kernel has DMA-safe access to these buffers and can perform
//! I/O without copying data.
//!
//! ## Buffer Pool Design
//!
//! Each buffer is MTU-sized (65535 bytes for TUN, 1500 for Ethernet).
//! We maintain two pools:
//!   - RX pool: buffers used for incoming packets (TUN reads)
//!   - TX pool: buffers used for outgoing packets (TUN writes)
//!
//! Buffers are reference-counted. When a packet is being processed,
//! its buffer is "checked out" and returned to the pool when done.
//! This eliminates allocations in the hot packet-processing path.

/// Buffer state in the pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferState {
    Free,
    InUse,
    PendingKernel,
}

/// A single buffer in the zero-copy pool.
pub struct ZeroCopyBuffer {
    pub index: usize,
    pub data: Vec<u8>,
    pub state: BufferState,
    pub len: usize,
}

impl ZeroCopyBuffer {
    pub fn new(index: usize, size: usize) -> Self {
        Self {
            index,
            data: vec![0u8; size],
            state: BufferState::Free,
            len: 0,
        }
    }
    pub fn as_ptr(&self) -> *const u8 { self.data.as_ptr() }
    pub fn as_mut_ptr(&mut self) -> *mut u8 { self.data.as_mut_ptr() }
    pub fn capacity(&self) -> usize { self.data.len() }
}

/// Zero-copy buffer pool.
pub struct ZeroCopyPool {
    buffers: Vec<ZeroCopyBuffer>,
    free_list: Vec<usize>,
    buffer_size: usize,
}

impl ZeroCopyPool {
    pub fn new(count: usize, buffer_size: usize) -> Self {
        let buffers: Vec<ZeroCopyBuffer> = (0..count)
            .map(|i| ZeroCopyBuffer::new(i, buffer_size))
            .collect();
        let free_list: Vec<usize> = (0..count).collect();
        Self { buffers, free_list, buffer_size }
    }

    /// Check out a free buffer. Returns None if pool is exhausted.
    pub fn checkout(&mut self) -> Option<usize> {
        let idx = self.free_list.pop()?;
        self.buffers[idx].state = BufferState::InUse;
        self.buffers[idx].len = 0;
        Some(idx)
    }

    /// Return a buffer to the free pool.
    pub fn return_buf(&mut self, idx: usize) {
        self.buffers[idx].state = BufferState::Free;
        self.buffers[idx].len = 0;
        self.free_list.push(idx);
    }

    pub fn free_count(&self) -> usize { self.free_list.len() }
    pub fn total_count(&self) -> usize { self.buffers.len() }
    pub fn buffer_size(&self) -> usize { self.buffer_size }

    /// Get raw pointers to all buffers for io_uring REGISTER_BUFFERS.
    pub fn iovec_list(&self) -> Vec<(*const u8, usize)> {
        self.buffers.iter()
            .map(|b| (b.as_ptr(), b.capacity()))
            .collect()
    }
}

/// Separate safety note:
/// ZeroCopyPool contains raw pointers passed to the kernel via io_uring.
/// The buffers must not be moved (or dropped) while registered.
/// In production, use Pin<Box<[u8]>> or mmap'd memory for the buffers.
unsafe impl Send for ZeroCopyPool {}
unsafe impl Sync for ZeroCopyPool {}
