#!/usr/bin/env bash
# ══════════════════════════════════════════════════════════════
# post-create.sh — Runs after on-create.sh, inside the workspace.
# Installs project-level dependencies (cargo, flutter, go, bun).
# ══════════════════════════════════════════════════════════════
set -euo pipefail

GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
log() { echo -e "${GREEN}[POST-CREATE]${NC} $1"; }

cd /workspace

# ── 1. Rust — fetch all crates ────────────────────────────────
log "Fetching Rust crates (daemon)..."
cd daemon && cargo fetch 2>/dev/null && cd ..
log "  ✓ Rust crates fetched"

# ── 2. Go dependencies ────────────────────────────────────────
log "Fetching Go modules..."
if [ -f go-bridge/go.mod ]; then
    cd go-bridge && go mod download && cd ..
    log "  ✓ Go modules downloaded"
fi

# ── 3. Flutter dependencies ───────────────────────────────────
log "Getting Flutter packages..."
for dir in flutter flutter_app; do
    if [ -f "$dir/pubspec.yaml" ]; then
        cd "$dir"
        flutter pub get --no-example 2>/dev/null
        cd ..
        log "  ✓ Flutter packages for $dir"
    fi
done

# ── 4. JS/TS (workers, dashboard, extensions) ────────────────
log "Installing JS dependencies..."
export PATH="$HOME/.bun/bin:$PATH"
for dir in workers dashboard extensions; do
    if [ -f "$dir/package.json" ]; then
        cd "$dir" && bun install --frozen-lockfile 2>/dev/null || bun install
        cd ..
        log "  ✓ bun install for $dir"
    fi
done
# Root package.json
if [ -f package.json ]; then
    bun install 2>/dev/null || true
fi

# ── 5. Python tools ───────────────────────────────────────────
log "Installing Python tools..."
pip3 install --quiet --break-system-packages \
    scapy \
    dpkt \
    cryptography \
    requests \
    aiohttp \
    2>/dev/null || true
log "  ✓ Python tools installed"

# ── 6. Verify zig-tun builds ─────────────────────────────────
log "Verifying Zig TUN module..."
if command -v zig &>/dev/null; then
    cd zig-tun && zig build 2>/dev/null && cd ..
    log "  ✓ zig-tun builds OK"
else
    log "  ⚠ zig not found, skipping zig-tun build"
fi

# ── 7. Git hooks ─────────────────────────────────────────────
log "Installing git hooks..."
cat > .git/hooks/pre-commit << 'HOOK'
#!/bin/bash
cd daemon && cargo clippy -- -D warnings 2>/dev/null
cd ..
HOOK
chmod +x .git/hooks/pre-commit
log "  ✓ Git hooks installed"

log "═══ post-create.sh complete — workspace ready ═══"

# ── MICAFP workspace dependencies ────────────────────────────────────────────
log "Fetching MICAFP workspace crates..."
if [ -f "Cargo.toml" ] && grep -q '\[workspace\]' Cargo.toml; then
    cargo fetch 2>/dev/null && log "  ✓ workspace crates fetched" || true
fi
