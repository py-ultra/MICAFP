//! Zig TUN Helper — Standalone binary for OpenWrt
//!
//! A tiny standalone helper that opens a TUN device and passes packets
//! to/from the main Rust daemon via Unix socket or pipe.
//! Binary size: ~80KB (ReleaseSmall, musl, MIPS32)

const std = @import("std");
const tun = @import("tun.zig");

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);

    const iface_name = if (args.len > 1) args[1] else "tun0";
    const fd = tun.tun_open(iface_name.ptr);
    if (fd < 0) {
        std.debug.print("Failed to open TUN device '{}': error {}\n", .{iface_name, fd});
        std.process.exit(1);
    }
    defer _ = tun.tun_close(fd);

    std.debug.print("TUN device '{}' opened (fd={})\n", .{iface_name, fd});
    std.debug.print("Passing control to Rust daemon...\n", .{});

    // Production: fork/exec Rust daemon passing fd via SCM_RIGHTS socket
    // or write fd number to stdout for parent process to inherit
    const stdout = std.io.getStdOut().writer();
    try stdout.print("{}\n", .{fd});

    // Block until daemon signals shutdown
    std.time.sleep(std.math.maxInt(u64));
}
