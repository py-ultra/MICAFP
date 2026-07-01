#!/usr/bin/env bash
# ══════════════════════════════════════════════════════════════
# post-start.sh — Runs on EVERY Codespace start (not just first).
# Lightweight checks and environment refresh only.
# ══════════════════════════════════════════════════════════════
set -euo pipefail

GREEN='\033[0;32m'; NC='\033[0m'
log() { echo -e "${GREEN}[START]${NC} $1"; }

export PATH="$HOME/.bun/bin:/opt/zig:$PATH"

log "UnifiedShield v8.0 — Codespace started"
log "Rust  : $(rustc --version 2>/dev/null || echo 'not found')"
log "Go    : $(go version 2>/dev/null | cut -d' ' -f3 || echo 'not found')"
log "Zig   : $(zig version 2>/dev/null || echo 'not found')"
log "Flutter: $(flutter --version 2>/dev/null | head -1 || echo 'not found')"
log "bun   : $(bun --version 2>/dev/null || echo 'not found')"
log ""
log "Quick commands:"
log "  make dev          → build daemon (debug)"
log "  make release      → full release build"
log "  make test         → run all tests"
log "  make android      → build Android library"
log "  make ios          → build iOS framework"
log "  make flutter      → build Flutter app"
log "  make zig-tun      → build Zig TUN module"
log "  make codespaces-help → show full help"
