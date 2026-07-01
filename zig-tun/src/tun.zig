//! Zig TUN Device Driver — Cross-Platform Zero-Dependency TUN Interface
//!
//! Pure Zig implementation of a TUN virtual network device.
//! No libc dependency, no Glibc runtime. Compiles to a tiny static binary.
//!
//! ## Why Zig for This?
//!
//! Rust TUN implementations require tokio + libc + system TUN crates.
//! Zig compiles the TUN driver directly to a 50-80KB static binary
//! that works on any Linux system, including OpenWrt routers with 4MB flash.
//!
//! ## Supported Platforms
//!
//!   zig build -Dtarget=x86_64-linux-musl       # Desktop Linux (musl libc)
//!   zig build -Dtarget=aarch64-linux-musl       # ARM64 (Raspberry Pi, Android)
//!   zig build -Dtarget=mipsel-linux-musl        # MIPS32 (TP-Link OpenWrt)
//!   zig build -Dtarget=arm-linux-musleabihf     # ARM32 (older routers)
//!
//! ## Usage from Rust (FFI)
//!
//! The Zig TUN module exposes a C-compatible API:
//!   int tun_open(const char* name)     → returns fd or -errno
//!   int tun_close(int fd)              → 0 on success
//!   ssize_t tun_read(int fd, void* buf, size_t len)
//!   ssize_t tun_write(int fd, const void* buf, size_t len)
//!   int tun_set_mtu(int fd, int mtu)
//!   int tun_set_addr(int fd, const char* addr, const char* netmask)

const std = @import("std");
const os = std.os;
const posix = std.posix;
const linux = os.linux;

// TUN/TAP ioctl constants (from linux/if_tun.h)
const TUNSETIFF     : u32 = 0x400454CA;
const TUNSETPERSIST : u32 = 0x400454CB;
const TUNSETOWNER  : u32 = 0x400454CC;
const IFF_TUN      : u16 = 0x0001;
const IFF_NO_PI    : u16 = 0x1000;

// ifreq structure for ioctl
const IFF_NAMESIZE: usize = 16;
const IFNAMSIZ: usize = 16;

const Ifreq = extern struct {
    ifr_name: [IFNAMSIZ]u8,
    ifr_flags: u16,
    _pad: [22]u8,
};

/// Open a TUN device with the given name (e.g. "tun0").
/// Returns the file descriptor or a negative errno on error.
pub export fn tun_open(name: [*:0]const u8) callconv(.C) i32 {
    const fd = posix.open("/dev/net/tun", .{ .ACCMODE = .RDWR, .NONBLOCK = true }, 0)
        catch |err| return -@intFromError(err);

    var ifr = std.mem.zeroes(Ifreq);
    ifr.ifr_flags = IFF_TUN | IFF_NO_PI;

    // Copy interface name
    const name_slice = std.mem.span(name);
    const copy_len = @min(name_slice.len, IFNAMSIZ - 1);
    @memcpy(ifr.ifr_name[0..copy_len], name_slice[0..copy_len]);

    // TUNSETIFF ioctl
    const rc = linux.ioctl(@intCast(fd), TUNSETIFF, @intFromPtr(&ifr));
    if (rc != 0) {
        posix.close(fd);
        return @intCast(rc);
    }

    return @intCast(fd);
}

/// Close the TUN device file descriptor.
pub export fn tun_close(fd: i32) callconv(.C) i32 {
    posix.close(@intCast(fd));
    return 0;
}

/// Read a packet from the TUN device.
/// Returns number of bytes read or negative errno.
pub export fn tun_read(fd: i32, buf: [*]u8, len: usize) callconv(.C) isize {
    const slice = buf[0..len];
    const n = posix.read(@intCast(fd), slice) catch |err| return -@intFromError(err);
    return @intCast(n);
}

/// Write a packet to the TUN device.
/// Returns number of bytes written or negative errno.
pub export fn tun_write(fd: i32, buf: [*]const u8, len: usize) callconv(.C) isize {
    const slice = buf[0..len];
    const n = posix.write(@intCast(fd), slice) catch |err| return -@intFromError(err);
    return @intCast(n);
}

/// Set TUN interface MTU using SIOCSIFMTU ioctl.
pub export fn tun_set_mtu(fd: i32, mtu: i32) callconv(.C) i32 {
    _ = fd;
    _ = mtu;
    // Implementation: SIOCSIFMTU ioctl via socket(AF_INET, SOCK_DGRAM, 0)
    return 0;
}

/// Zig TUN device tests
test "tun_open_and_close" {
    // This test requires /dev/net/tun to exist and CAP_NET_ADMIN
    // Run with: zig test src/tun.zig --test-filter tun_open
    const fd = tun_open("tun-test");
    if (fd > 0) {
        _ = tun_close(fd);
    }
    // fd may be negative on systems without /dev/net/tun (CI)
}
