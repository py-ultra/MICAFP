# eBPF Program Build Instructions

## Prerequisites

```bash
# Ubuntu/Debian
sudo apt install clang llvm libelf-dev linux-headers-$(uname -r) libbpf-dev

# Fedora/RHEL
sudo dnf install clang llvm elfutils-libelf-devel kernel-devel libbpf-devel
```

## Compile eBPF Programs

```bash
# TLS splitter (tc egress)
clang -O2 -g -target bpf \
      -D__TARGET_ARCH_x86_64 \
      -I/usr/include/x86_64-linux-gnu \
      -c tls_splitter.bpf.c \
      -o tls_splitter.bpf.o

# Verify (optional)
llvm-objdump -d tls_splitter.bpf.o
```

## Embed in Rust Binary

The compiled `.bpf.o` files are embedded via `build.rs`:

```rust
// build.rs
println!("cargo:rerun-if-changed=bpf/tls_splitter.bpf.c");
// Compile BPF program here using cc crate or Command::new("clang")
```

Then in Rust:
```rust
const BPF_TLS_SPLITTER: &[u8] = include_bytes!("../bpf/tls_splitter.bpf.o");
```

## Cross-Architecture BPF

eBPF bytecode is architecture-independent — the kernel JIT compiles it
to native machine code at load time. The same `.bpf.o` works on:
- x86_64 (desktop/server)
- aarch64 (ARM64 / Android)
- arm32 (older routers with eBPF support)
