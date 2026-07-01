# MICAFP-UnifiedShield v10.0 — Smart Core / Dumb UI Architecture

> Zero features removed. All 16 source projects preserved.
> Flutter + flutter_rust_bridge v2 + BLoC layer added.

---

## What is New in v10.0

Version 10.0 implements the **Smart Core, Dumb UI** architecture across the full Flutter client.

The Rust daemon is the sole decision-maker for all network operations.
The Flutter layer observes daemon state and relays manual user commands.
It never participates in transport selection, failover timing, ping measurement,
or any network decision.

```
Flutter UI (Dart / BLoC)
  │  Renders state • Relays commands • NEVER makes network calls
  │
  │  flutter_rust_bridge v2 (zero-copy FFI + push event stream)
  │
Rust Daemon — Smart Core
  ├── AI Engine: UCB Bandit + RL selector + DPI classifier
  ├── Orchestrator: auto-failover in < 200 ms
  ├── 27 Transport protocols (VLESS·VMess·Trojan·Hysteria2·TUIC·…)
  ├── 9 VPN cores (xray, sing-box, hiddify, psiphon, tor, …)
  ├── Post-quantum cryptography (ML-KEM-768)
  ├── P2P mesh networking (libp2p)
  ├── NAIN / National Intranet detection
  └── Android VpnService TUN fd → all device traffic (no root)
```

---

## New Files in v10.0

| Path | Purpose |
|------|---------|
| `daemon/src/frb_api/mod.rs` | Sole FFI surface between Rust and Dart |
| `daemon/src/platform/android/android_tun.rs` | TUN fd JNI handler |
| `flutter/lib/src/bloc/dashboard_bloc.dart` | Core BLoC state machine |
| `flutter/lib/src/bloc/bloc_providers.dart` | Centralised BLoC registry |
| `flutter/lib/src/bridge/shield_bridge_stub.dart` | Stub for pre-codegen builds |
| `flutter/lib/src/screens/advanced_dashboard_screen.dart` | BLoC dashboard |
| `flutter/lib/src/widgets/advanced/protocol_switcher_widget.dart` | Transport override |
| `flutter/lib/src/widgets/advanced/dpi_alert_panel.dart` | DPI alert display |
| `flutter/lib/src/widgets/advanced/failover_history_widget.dart` | Failover log |
| `flutter/test/bloc/dashboard_bloc_test.dart` | 9 BLoC unit tests |
| `ENGINEERING-PROMPT.md` | Full v10.0 architecture specification |

---

## Modified Files (additions only — nothing removed)

| Path | Change summary |
|------|---------------|
| `daemon/Cargo.toml` | `flutter_rust_bridge = "2"` + `frb-mobile` feature gate |
| `daemon/src/lib.rs` | `pub mod frb_api`, `daemon_start`, `dispatch_frb_command` |
| `daemon/src/orchestrator/mod.rs` | `EVENT_TX`, `event_stream()`, `publish()`, `attach_tun()` |
| `daemon/src/orchestrator/failover.rs` | `record_failover_with_latency()` |
| `daemon/src/orchestrator/control_plane.rs` | `current_snapshot()`, `publish_status_tick()` |
| `daemon/src/ai/dpi_classifier.rs` | `emit_dpi_alert()` |
| `daemon/src/transport/mod.rs` | `available_transport_names()` |
| `daemon/src/cores/mod.rs` | `available_core_names()` |
| `android/ShieldVpnService.kt` | `startTunnel()`, `stopTunnel()`, JNI bindings |
| `flutter/lib/main.dart` | `MultiBlocProvider` + `_ShieldRoot` bottom nav |
| `flutter/pubspec.yaml` | FRB + BLoC + Freezed + mocktail dependencies |
| `.github/workflows/unified-build-release.yml` | `flutter-frb-codegen` + `flutter-build-apk` jobs |
| `setup.sh` | `install_flutter_deps()` function |
| `Makefile` | `frb-gen`, `flutter-all`, `validate-all`, `invariant-check` targets |

---

## Quick Start

```bash
# 1. Full environment setup (idempotent, fully non-interactive)
./setup.sh

# 2. Regenerate FRB bridge after any change to daemon/src/frb_api/mod.rs
make frb-gen

# 3. Full Flutter pipeline: codegen → analyze → test → APK build
make flutter-all

# 4. Validate all six architecture invariants
make validate-all
```

---

## Platform Support

| Platform | VPN Mechanism | Status |
|----------|--------------|--------|
| Android | `VpnService` — TUN fd transferred to Rust via JNI | v10.0 |
| iOS | `NetworkExtension` — NEPacketTunnelProvider | existing |
| Linux | `/dev/net/tun` | existing |
| Windows | WinTun driver | existing |
| OpenWrt | `netifd` protocol handler | existing |
| Chrome | Extension + WASM proxy | existing |
| Firefox | Extension + WASM proxy | existing |

---

## Architecture Invariants (CI-enforced)

| # | Invariant |
|---|-----------|
| I-01 | Flutter never references `transport::` functions |
| I-02 | `#[no_mangle]` symbols exist only in `frb_api/mod.rs` |
| I-03 | All UI updates flow through BLoC state transitions |
| I-04 | Every `Err` path in the daemon publishes `ShieldEvent::Error` |
| I-05 | Failover latency ≤ 200 ms, measured inside Rust |
| I-06 | Android TUN fd ownership transfers to Rust exactly once |
