# Engineering Prompt — MICAFP-UnifiedShield v10.0
## Flutter + flutter_rust_bridge v2 + BLoC + Smart Core (Android VpnService Architecture)
### Target Audience: Agentic AI Coding Systems (OpenHands · Aider · Claude Code)

---

## 0. Directive

You are a senior systems engineer extending **MICAFP-UnifiedShield v9.0** with
a production-grade Flutter UI layer connected to the existing Rust daemon via
**flutter_rust_bridge v2** (zero-copy FFI), managed by a **BLoC** state machine,
and backed by an on-device Rust AI decision engine that operates with **zero user
interaction** during failover events.

**Non-negotiable constraints:**

- Never delete, rename, or disable any existing module, transport, protocol, file,
  or feature. Only add, fix, or improve.
- Every build step must complete without errors or warnings.
- All automation must be **fully non-interactive** — no `read`, no `pause`, no prompts.
- The Rust daemon (`daemon/`) is the sole brain for all network decisions. The
  Flutter/Dart layer is a **monitor and remote control only**.
- Android target: **VpnService** (system-wide, no root required).

---

## 1. Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│           Flutter UI (Dart / BLoC) — Monitor + Remote Control   │
│   Renders state • Sends user commands • NEVER makes net calls   │
└──────────────────────────┬──────────────────────────────────────┘
                           │ flutter_rust_bridge v2 (zero-copy FFI)
                           │ + Broadcast channel (push events)
┌──────────────────────────▼──────────────────────────────────────┐
│                Rust Daemon — Smart Core                         │
│  AI Engine: UCBBandit · RlTransportSelector · DpiClassifier     │
│  Orchestrator: health_monitor → failover (<200 ms)              │
│  27 Transports · 9 VPN Cores · PQC · P2P · NAIN detection      │
│  Android VpnService fd ──► TUN device ──► all device traffic    │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. v10.0 Changes

### New files added

| Path | Purpose |
|------|---------|
| `daemon/src/frb_api/mod.rs` | FRB v2 API surface — sole FFI boundary (TASK-01) |
| `daemon/src/platform/android/android_tun.rs` | TUN fd JNI handler (TASK-06) |
| `flutter/lib/src/bloc/dashboard_bloc.dart` | DashboardBloc state machine (TASK-04) |
| `flutter/lib/src/bloc/bloc_providers.dart` | BLoC providers registry (TASK-04) |
| `flutter/lib/src/bridge/shield_bridge_stub.dart` | FRB stub for pre-codegen builds |
| `flutter/lib/src/screens/advanced_dashboard_screen.dart` | BLoC-powered dashboard |
| `flutter/lib/src/widgets/advanced/protocol_switcher_widget.dart` | Protocol switcher |
| `flutter/lib/src/widgets/advanced/dpi_alert_panel.dart` | DPI alert display |
| `flutter/lib/src/widgets/advanced/failover_history_widget.dart` | Failover log |
| `flutter/test/bloc/dashboard_bloc_test.dart` | BLoC unit tests (TASK-05) |

### Modified files (additions only, nothing removed)

| Path | Change |
|------|--------|
| `daemon/Cargo.toml` | Added `flutter_rust_bridge = "2"` + `frb-mobile` feature |
| `daemon/src/lib.rs` | Added `pub mod frb_api`, `daemon_start`, `dispatch_frb_command` |
| `daemon/src/orchestrator/mod.rs` | Added `EVENT_TX`, `event_stream()`, `publish()`, `attach_tun()` |
| `daemon/src/orchestrator/failover.rs` | Added `record_failover_with_latency()` |
| `daemon/src/orchestrator/control_plane.rs` | Added `current_snapshot()`, `publish_status_tick()` |
| `daemon/src/ai/dpi_classifier.rs` | Added `emit_dpi_alert()` |
| `daemon/src/transport/mod.rs` | Added `available_transport_names()` |
| `daemon/src/cores/mod.rs` | Added `available_core_names()` |
| `android/ShieldVpnService.kt` | Added `startTunnel()`, `stopTunnel()`, JNI bindings |
| `flutter/lib/main.dart` | Added BLoC providers + bottom nav with `_ShieldRoot` |
| `flutter/pubspec.yaml` | Added FRB + BLoC + Freezed dependencies |
| `.github/workflows/unified-build-release.yml` | Added `flutter-frb-codegen` + `flutter-build-apk` jobs |
| `setup.sh` | Added `install_flutter_deps()` function |
| `Makefile` | Added FRB/Flutter targets |

---

## 3. Invariants

| # | Invariant | Enforcement |
|---|-----------|-------------|
| I-01 | Flutter never calls `transport::` functions directly | `grep -r "transport::" flutter/` must return empty |
| I-02 | Only `frb_api/mod.rs` has `#[no_mangle]` | `grep -r "#[no_mangle]" daemon/src/ | grep -v frb_api` must return empty |
| I-03 | All UI updates via BLoC state | No `setState()` referencing network state |
| I-04 | Every `Err(_)` must surface via `publish(ShieldEvent::Error{…})` | `cargo clippy` enforced in CI |
| I-05 | Failover latency ≤ 200 ms | Measured in Rust; `cargo test failover_latency_test` |
| I-06 | VpnService fd ownership transfers to Rust exactly once | `detachFd()` in Kotlin, `OwnedFd` in Rust |

---

## 4. Quick Start

```bash
# 1. Install all dependencies and run FRB codegen
./setup.sh

# 2. Regenerate FRB bridge after any frb_api change
make frb-gen

# 3. Analyze + test Flutter layer
make flutter-all

# 4. Full invariant validation
make validate-all
```

---

## 5. Glossary

| Term | Meaning |
|------|---------|
| **FRB** | flutter_rust_bridge v2 — zero-copy Rust↔Dart FFI code generator |
| **BLoC** | Business Logic Component — Flutter state management pattern |
| **Smart Core** | The Rust daemon: owns all network decisions autonomously |
| **Dumb UI** | Flutter layer: renders state, relays commands, never decides |
| **HotSwap** | Sub-200 ms transport failover by the Rust AI engine |
| **DPI** | Deep Packet Inspection — censorship mechanism detected by the AI |
| **NAIN** | National Intranet — Iran's disconnected intranet mode |
| **TUN fd** | Linux TUN device file descriptor from VpnService → Rust |
| **ShieldEvent** | Union type pushed from Rust to Dart via FRB Stream (read-only) |
| **ShieldCommand** | Union type sent from Dart to Rust via FRB async call (write-only) |
