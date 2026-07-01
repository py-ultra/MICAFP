//! Build configuration for the Zig TUN module.
//! Supports cross-compilation for all Iranian router platforms.

const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // Shared library for FFI from Rust daemon
    const lib = b.addSharedLibrary(.{
        .name = "tun",
        .root_source_file = .{ .path = "src/tun.zig" },
        .target = target,
        .optimize = optimize,
    });
    b.installArtifact(lib);

    // Static library for embedding in the binary
    const static_lib = b.addStaticLibrary(.{
        .name = "tun_static",
        .root_source_file = .{ .path = "src/tun.zig" },
        .target = target,
        .optimize = optimize,
    });
    b.installArtifact(static_lib);

    // Standalone binary for OpenWrt deployment
    const exe = b.addExecutable(.{
        .name = "tun-helper",
        .root_source_file = .{ .path = "src/main.zig" },
        .target = target,
        .optimize = .ReleaseSmall, // Minimise binary size for router flash
    });
    b.installArtifact(exe);

    // Tests
    const unit_tests = b.addTest(.{
        .root_source_file = .{ .path = "src/tun.zig" },
        .target = target,
        .optimize = optimize,
    });
    const run_tests = b.addRunArtifact(unit_tests);
    const test_step = b.step("test", "Run TUN unit tests");
    test_step.dependOn(&run_tests.step);
}

// Cross-compilation targets for Iranian deployment
// Usage:
//   zig build -Dtarget=mipsel-linux-musl -Doptimize=ReleaseSmall   # TP-Link Archer
//   zig build -Dtarget=arm-linux-musleabihf -Doptimize=ReleaseSmall # Asus RT
//   zig build -Dtarget=aarch64-linux-musl -Doptimize=ReleaseFast    # Raspberry Pi / Apple M1
//   zig build -Dtarget=x86_64-linux-musl -Doptimize=ReleaseSafe     # x86 server
