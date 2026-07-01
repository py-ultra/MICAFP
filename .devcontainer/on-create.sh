#!/usr/bin/env bash
# ══════════════════════════════════════════════════════════════
# on-create.sh — Runs ONCE when the Codespace container is built
# Installs ALL system dependencies automatically.
# ══════════════════════════════════════════════════════════════
set -euo pipefail

GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
log() { echo -e "${GREEN}[SETUP]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

log "═══ UnifiedShield v8.0 — Codespace bootstrap starting ═══"

# ── 1. System packages ─────────────────────────────────────────
log "Installing system packages..."
export DEBIAN_FRONTEND=noninteractive
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends \
    build-essential curl wget git pkg-config \
    clang llvm lld \
    libelf-dev libssl-dev libpcap-dev \
    linux-headers-generic libbpf-dev \
    protobuf-compiler libprotobuf-dev \
    cmake ninja-build \
    libsqlite3-dev \
    android-tools-adb \
    unzip zip tar \
    jq yq python3-pip \
    iproute2 iptables nftables \
    netcat-openbsd tcpdump \
    2>/dev/null
log "  ✓ System packages installed"

# ── 2. Zig (not in apt — download directly) ────────────────────
log "Installing Zig 0.13.0..."
ZIG_VERSION="0.13.0"
ZIG_ARCH="x86_64"
ZIG_URL="https://ziglang.org/download/${ZIG_VERSION}/zig-linux-${ZIG_ARCH}-${ZIG_VERSION}.tar.xz"
wget -q "$ZIG_URL" -O /tmp/zig.tar.xz
sudo mkdir -p /opt/zig
sudo tar -xJf /tmp/zig.tar.xz -C /opt/zig --strip-components=1
sudo ln -sf /opt/zig/zig /usr/local/bin/zig
rm /tmp/zig.tar.xz
zig version
log "  ✓ Zig $(zig version) installed"

# ── 3. Rust targets for cross-compilation ──────────────────────
log "Adding Rust cross-compilation targets..."
rustup target add \
    aarch64-linux-android \
    armv7-linux-androideabi \
    x86_64-linux-android \
    x86_64-unknown-linux-musl \
    aarch64-unknown-linux-musl \
    x86_64-pc-windows-gnu
rustup component add clippy rustfmt rust-src
log "  ✓ Rust targets added"

# ── 4. cargo tools ────────────────────────────────────────────
log "Installing cargo tools..."
cargo install --locked \
    cross \
    cargo-deny \
    cargo-audit \
    cargo-expand \
    cargo-watch \
    cargo-outdated \
    2>/dev/null || true
log "  ✓ cargo tools installed"

# ── 5. Android NDK (for Android builds) ───────────────────────
log "Setting up Android NDK..."
NDK_VERSION="r26d"
NDK_URL="https://dl.google.com/android/repository/android-ndk-${NDK_VERSION}-linux.zip"
if [ ! -d "/opt/android-ndk" ]; then
    wget -q "$NDK_URL" -O /tmp/ndk.zip
    sudo unzip -q /tmp/ndk.zip -d /opt/
    sudo mv /opt/android-ndk-${NDK_VERSION} /opt/android-ndk
    rm /tmp/ndk.zip
fi
export ANDROID_NDK_HOME=/opt/android-ndk
echo 'export ANDROID_NDK_HOME=/opt/android-ndk' >> ~/.bashrc
echo 'export PATH=$PATH:/opt/android-ndk/toolchains/llvm/prebuilt/linux-x86_64/bin' >> ~/.bashrc
log "  ✓ Android NDK installed at /opt/android-ndk"

# ── 6. bun (JavaScript runtime for workers/dashboard) ─────────
log "Installing bun..."
curl -fsSL https://bun.sh/install | bash 2>/dev/null
export PATH="$HOME/.bun/bin:$PATH"
echo 'export PATH="$HOME/.bun/bin:$PATH"' >> ~/.bashrc
log "  ✓ bun installed"

# ── 7. Protocol Buffers ───────────────────────────────────────
log "Installing protoc plugins..."
go install google.golang.org/protobuf/cmd/protoc-gen-go@latest 2>/dev/null || true
go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest 2>/dev/null || true
log "  ✓ protoc plugins installed"

# ── 8. eBPF clang target headers ─────────────────────────────
log "Setting up eBPF build environment..."
sudo apt-get install -y linux-libc-dev 2>/dev/null
# Install libbpf headers for eBPF program compilation
sudo apt-get install -y libbpf-dev 2>/dev/null || true
log "  ✓ eBPF build tools ready"

log "═══ on-create.sh complete ═══"

# ── v10.0: flutter_rust_bridge v2 codegen ────────────────────────────────────
log "Installing flutter_rust_bridge_codegen v2..."
cargo install flutter_rust_bridge_codegen --version "^2" --locked 2>/dev/null || \
    warn "FRB codegen install failed — run manually: cargo install flutter_rust_bridge_codegen"
log "  ✅ flutter_rust_bridge_codegen installed"
