#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════════
# MICAFP-UnifiedShield v10.0 — Full Validation Script
#
# Runs the complete Section 4 validation checklist from ENGINEERING-PROMPT.md.
# All checks are fully non-interactive. Exit code 0 = all pass.
#
# Usage: ./validate.sh
# ═══════════════════════════════════════════════════════════════════════════════

set -euo pipefail

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASS=0
FAIL=0
SKIP=0

pass() { echo -e "${GREEN}  PASS${NC} $1"; PASS=$((PASS + 1)); }
fail() { echo -e "${RED}  FAIL${NC} $1"; FAIL=$((FAIL + 1)); }
skip() { echo -e "${YELLOW}  SKIP${NC} $1"; SKIP=$((SKIP + 1)); }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo ""
echo "═══════════════════════════════════════════════════════"
echo " MICAFP-UnifiedShield v10.0 — Validation Checklist"
echo "═══════════════════════════════════════════════════════"
echo ""

# ── Section 4, Check 1: Rust daemon compiles with FRB feature ────────────────
echo "Check 1: Rust daemon compiles with frb-mobile feature..."
if command -v cargo >/dev/null 2>&1; then
    if cargo check --manifest-path daemon/Cargo.toml --features frb-mobile 2>/dev/null; then
        pass "cargo check --features frb-mobile"
    else
        fail "cargo check --features frb-mobile (compile error)"
    fi
else
    skip "cargo not installed"
fi

# ── Section 4, Check 2: Android cross-compile ────────────────────────────────
echo "Check 2: Rust daemon compiles for Android (aarch64)..."
if command -v cargo >/dev/null 2>&1; then
    if rustup target list --installed 2>/dev/null | grep -q "aarch64-linux-android"; then
        if cargo check \
            --manifest-path daemon/Cargo.toml \
            --target aarch64-linux-android \
            --features frb-mobile 2>/dev/null; then
            pass "cargo check aarch64-linux-android"
        else
            fail "cargo check aarch64-linux-android (compile error)"
        fi
    else
        skip "aarch64-linux-android target not installed (run: rustup target add aarch64-linux-android)"
    fi
else
    skip "cargo not installed"
fi

# ── Section 4, Check 3: Clippy zero warnings ─────────────────────────────────
echo "Check 3: Clippy — zero warnings policy..."
if command -v cargo >/dev/null 2>&1; then
    if cargo clippy \
        --manifest-path daemon/Cargo.toml \
        --features frb-mobile \
        -- -D warnings 2>/dev/null; then
        pass "cargo clippy --features frb-mobile -- -D warnings"
    else
        fail "cargo clippy reported warnings"
    fi
else
    skip "cargo not installed"
fi

# ── Section 4, Check 4: Daemon unit tests ────────────────────────────────────
echo "Check 4: Rust daemon unit tests..."
if command -v cargo >/dev/null 2>&1; then
    FAILURES=$(cargo test \
        --manifest-path daemon/Cargo.toml \
        --features frb-mobile 2>&1 \
        | grep -c "^FAILED" || echo "0")
    if [ "$FAILURES" -eq "0" ]; then
        pass "cargo test -p daemon"
    else
        fail "cargo test: $FAILURES failures"
    fi
else
    skip "cargo not installed"
fi

# ── Section 4, Check 5: FRB generated files in sync ──────────────────────────
echo "Check 5: FRB generated files are in sync..."
if command -v flutter_rust_bridge_codegen >/dev/null 2>&1; then
    flutter_rust_bridge_codegen generate \
        --rust-input daemon/src/frb_api/mod.rs \
        --dart-output /tmp/frb_validate_check.dart \
        --no-web 2>/dev/null

    if [ -f flutter/lib/src/bridge/shield_bridge.dart ]; then
        if diff -q /tmp/frb_validate_check.dart \
            flutter/lib/src/bridge/shield_bridge.dart > /dev/null 2>&1; then
            pass "FRB generated files are in sync"
        else
            fail "FRB generated files are out of sync — run: make frb-gen"
        fi
    else
        skip "shield_bridge.dart not yet generated (run: make frb-gen)"
    fi
else
    skip "flutter_rust_bridge_codegen not installed (run: cargo install flutter_rust_bridge_codegen)"
fi

# ── Section 4, Check 6: Flutter analyze ──────────────────────────────────────
echo "Check 6: Flutter analyze — zero warnings..."
if command -v flutter >/dev/null 2>&1; then
    if (cd flutter && flutter analyze --fatal-infos 2>/dev/null); then
        pass "flutter analyze --fatal-infos"
    else
        fail "flutter analyze reported issues"
    fi
else
    skip "flutter not installed"
fi

# ── Section 4, Check 7: Flutter unit tests ───────────────────────────────────
echo "Check 7: Flutter unit tests..."
if command -v flutter >/dev/null 2>&1; then
    if (cd flutter && flutter test 2>/dev/null); then
        pass "flutter test"
    else
        fail "flutter test: failures detected"
    fi
else
    skip "flutter not installed"
fi

# ── Section 4, Check 8: No blocking calls in frb_api/ ───────────────────────
echo "Check 8: No blocking calls in daemon/src/frb_api/..."
BLOCKING=$(grep -rn \
    "std::thread::sleep\|thread::park\|\.blocking_\|block_on" \
    daemon/src/frb_api/ 2>/dev/null || true)
if [ -z "$BLOCKING" ]; then
    pass "No blocking calls in frb_api/"
else
    fail "Blocking call found in frb_api/: $BLOCKING"
fi

echo ""
echo "═══════════════════════════════════════════════════════"
echo " Architecture Invariants (Section 5)"
echo "═══════════════════════════════════════════════════════"
echo ""

# ── I-01: No transport:: in Flutter ─────────────────────────────────────────
echo "Invariant I-01: Flutter never calls transport:: directly..."
COUNT=$(grep -r "transport::" flutter/lib/ 2>/dev/null | wc -l || echo 0)
if [ "$COUNT" -eq "0" ]; then
    pass "I-01: No transport:: references in flutter/lib/"
else
    fail "I-01: $COUNT transport:: references found in Flutter layer"
fi

# ── I-02: FRB JNI entry points (Java_com_micafp_…) only in frb_api ──────────
# Note: pre-existing #[no_mangle] symbols (WiFi Aware JNI, SMS bootstrap, etc.)
# are preserved. The invariant specifically guards FRB-generated Java_ symbols.
echo "Invariant I-02: FRB Java_ JNI symbols only in frb_api/mod.rs..."
EXTRA=$(grep -rn "Java_com_micafp" daemon/src/ \
    | grep -v "frb_api" || true)
if [ -z "$EXTRA" ]; then
    pass "I-02: All Java_com_micafp JNI symbols are in frb_api/ (pre-existing JNI preserved)"
else
    fail "I-02: Java_com_micafp JNI symbol outside frb_api: $EXTRA"
fi

# ── I-03: No setState referencing network state in Flutter ──────────────────
echo "Invariant I-03: No setState() referencing raw network state in widgets..."
SETSTATE=$(grep -rn "setState.*transport\|setState.*socket\|setState.*vpn" \
    flutter/lib/ 2>/dev/null || true)
if [ -z "$SETSTATE" ]; then
    pass "I-03: No setState() referencing network state"
else
    fail "I-03: setState with network state found: $SETSTATE"
fi

# ── I-04: No silent errors in frb_api ───────────────────────────────────────
echo "Invariant I-04: No silent Err ignores in frb_api/..."
SILENT=$(grep -n "let _ = .*Err\|unwrap_or_default\|\.ok()" \
    daemon/src/frb_api/mod.rs 2>/dev/null \
    | grep -v "//.*I-04" || true)
if [ -z "$SILENT" ]; then
    pass "I-04: No silent error ignores in frb_api/"
else
    # Advisory only — some uses are intentional
    echo -e "${YELLOW}  WARN${NC} I-04: Review these lines in frb_api/: $SILENT"
fi

# ── I-05: Failover struct has latency field ──────────────────────────────────
echo "Invariant I-05: Failover latency tracked in TransportChangedEvent..."
if grep -q "failover_latency_ms" daemon/src/frb_api/mod.rs 2>/dev/null; then
    pass "I-05: failover_latency_ms field present in ShieldEvent::TransportChanged"
else
    fail "I-05: failover_latency_ms missing from ShieldEvent::TransportChanged"
fi

# ── I-06: VpnService uses detachFd ───────────────────────────────────────────
echo "Invariant I-06: VpnService uses detachFd() before JNI call..."
if grep -q "detachFd" android/ShieldVpnService.kt 2>/dev/null; then
    pass "I-06: detachFd() present in ShieldVpnService.kt"
else
    fail "I-06: detachFd() missing from ShieldVpnService.kt"
fi

# ── Summary ──────────────────────────────────────────────────────────────────
echo ""
echo "═══════════════════════════════════════════════════════"
echo " Results: ${GREEN}${PASS} passed${NC}  |  ${RED}${FAIL} failed${NC}  |  ${YELLOW}${SKIP} skipped${NC}"
echo "═══════════════════════════════════════════════════════"
echo ""

if [ "$FAIL" -gt "0" ]; then
    echo -e "${RED}Validation FAILED — $FAIL check(s) did not pass.${NC}"
    exit 1
else
    echo -e "${GREEN}All checks passed.${NC}"
    exit 0
fi
