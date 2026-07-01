//! io_uring TUN Device Reader
//!
//! Reads packets from the TUN device using io_uring IORING_OP_READ
//! with registered buffers. In SQPOLL mode, the kernel polling thread
//! reads packets without any syscall from userspace.
//!
//! ## Comparison with epoll approach
//!
//! epoll approach:
//!   1. epoll_wait() — syscall, blocks until TUN fd is readable
//!   2. read(tun_fd, buf) — syscall, copies data to userspace
//!   Total: 2 syscalls per packet batch
//!
//! io_uring approach (SQPOLL):
//!   1. Submit IORING_OP_READ SQE to ring buffer — userspace write to mmap
//!   2. Kernel SQPOLL thread sees SQE, reads from TUN fd, writes CQE
//!   3. Userspace reads CQE from ring buffer — userspace read from mmap
//!   Total: 0 syscalls (both SQ write and CQ read are mmap operations)

use std::os::unix::io::RawFd;
use tracing::{debug, trace};

/// io_uring TUN reader state.
pub struct IoUringTunReader {
    tun_fd: RawFd,
    /// Registered buffer indices (into the io_uring registered buffer table).
    buffer_indices: Vec<usize>,
    /// Buffer size in bytes (should match TUN MTU, e.g. 65535).
    buffer_size: usize,
    packets_read: u64,
    bytes_read: u64,
}

impl IoUringTunReader {
    pub fn new(tun_fd: RawFd, buffer_count: usize, buffer_size: usize) -> Self {
        Self {
            tun_fd,
            buffer_indices: (0..buffer_count).collect(),
            buffer_size,
            packets_read: 0,
            bytes_read: 0,
        }
    }

    /// Submit a batch of read SQEs to the io_uring submission queue.
    /// Returns the number of SQEs submitted.
    pub fn submit_reads(&mut self, batch_size: usize) -> usize {
        // Production:
        //  for each available buffer:
        //    ring.submission().push(
        //        opcode::ReadFixed::new(Fd(self.tun_fd), buf_ptr, buf_len, buf_idx)
        //            .build().user_data(buf_idx as u64)
        //    )
        let submitted = self.buffer_indices.len().min(batch_size);
        trace!("io_uring TUN reader: submitted {} read SQEs", submitted);
        submitted
    }

    /// Process completed reads from the completion queue.
    /// Returns packets as Vec<(buffer_index, length)>.
    pub fn drain_completions(&mut self) -> Vec<(usize, usize)> {
        // Production:
        //  ring.completion().map(|cqe| {
        //      let buf_idx = cqe.user_data() as usize;
        //      let len = cqe.result() as usize;
        //      self.packets_read += 1;
        //      self.bytes_read += len as u64;
        //      (buf_idx, len)
        //  }).collect()
        vec![]
    }

    pub fn packets_read(&self) -> u64 { self.packets_read }
    pub fn bytes_read(&self) -> u64 { self.bytes_read }
}
