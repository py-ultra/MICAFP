# ══════════════════════════════════════════════════════════════
# UnifiedShield v8.0 — Custom Codespaces Dockerfile
# Use this instead of devcontainer.json image field
# for full reproducible environment control.
# ══════════════════════════════════════════════════════════════

FROM ubuntu:24.04

ARG ZIG_VERSION=0.13.0
ARG RUST_VERSION=stable
ARG GO_VERSION=1.22.3
ARG NDK_VERSION=r26d

ENV DEBIAN_FRONTEND=noninteractive
ENV RUST_BACKTRACE=1
ENV CARGO_TERM_COLOR=always
ENV ANDROID_NDK_HOME=/opt/android-ndk
ENV PATH=/opt/zig:/opt/android-ndk/toolchains/llvm/prebuilt/linux-x86_64/bin:${PATH}

# ── System dependencies ────────────────────────────────────────
RUN apt-get update -qq && apt-get install -y --no-install-recommends \
    build-essential curl wget git pkg-config sudo ca-certificates \
    clang-18 llvm-18 lld-18 \
    libelf-dev libssl-dev libpcap-dev libbpf-dev \
    linux-libc-dev \
    protobuf-compiler libprotobuf-dev \
    cmake ninja-build \
    unzip zip tar jq \
    iproute2 iptables nftables \
    python3 python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Create symlinks for clang-18
RUN ln -sf /usr/bin/clang-18 /usr/bin/clang && \
    ln -sf /usr/bin/llvm-objdump-18 /usr/bin/llvm-objdump

# ── Rust ──────────────────────────────────────────────────────
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain ${RUST_VERSION}
ENV PATH=/root/.cargo/bin:${PATH}
RUN rustup target add \
    aarch64-linux-android armv7-linux-androideabi x86_64-linux-android \
    x86_64-unknown-linux-musl aarch64-unknown-linux-musl \
    x86_64-pc-windows-gnu && \
    rustup component add clippy rustfmt rust-src

# ── Go ────────────────────────────────────────────────────────
RUN wget -q https://go.dev/dl/go${GO_VERSION}.linux-amd64.tar.gz -O /tmp/go.tar.gz && \
    tar -xzf /tmp/go.tar.gz -C /usr/local && rm /tmp/go.tar.gz
ENV PATH=/usr/local/go/bin:${PATH}

# ── Zig ───────────────────────────────────────────────────────
RUN wget -q https://ziglang.org/download/${ZIG_VERSION}/zig-linux-x86_64-${ZIG_VERSION}.tar.xz \
    -O /tmp/zig.tar.xz && \
    mkdir -p /opt/zig && \
    tar -xJf /tmp/zig.tar.xz -C /opt/zig --strip-components=1 && \
    rm /tmp/zig.tar.xz && \
    ln -sf /opt/zig/zig /usr/local/bin/zig

# ── bun ───────────────────────────────────────────────────────
RUN curl -fsSL https://bun.sh/install | bash
ENV PATH=/root/.bun/bin:${PATH}

# ── Android NDK ───────────────────────────────────────────────
RUN wget -q https://dl.google.com/android/repository/android-ndk-${NDK_VERSION}-linux.zip \
    -O /tmp/ndk.zip && \
    unzip -q /tmp/ndk.zip -d /opt/ && \
    mv /opt/android-ndk-${NDK_VERSION} /opt/android-ndk && \
    rm /tmp/ndk.zip

# ── cargo tools ───────────────────────────────────────────────
RUN cargo install --locked cross cargo-deny cargo-audit cargo-watch 2>/dev/null || true

# ── Python tools ──────────────────────────────────────────────
RUN pip3 install --quiet --break-system-packages \
    scapy dpkt cryptography requests aiohttp || true

WORKDIR /workspace
CMD ["/bin/bash"]
