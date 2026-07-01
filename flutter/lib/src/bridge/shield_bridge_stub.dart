// ─────────────────────────────────────────────────────────────────────────────
// MICAFP-UnifiedShield v10.0 — FRB Bridge Stub
//
// THIS FILE IS A HAND-WRITTEN STUB. In CI/CD (flutter-frb-codegen job) and
// in local dev after `make frb-gen`, this file is replaced by the output of:
//
//   flutter_rust_bridge_codegen generate \
//     --rust-input  daemon/src/frb_api/mod.rs \
//     --dart-output flutter/lib/src/bridge/shield_bridge.dart
//
// The stub mirrors all types and function signatures so that the BLoC and
// widget layers compile and are testable without Rust toolchain installed.
// ─────────────────────────────────────────────────────────────────────────────

import 'dart:async';

// ── DTO types (mirror daemon/src/frb_api/mod.rs) ──────────────────────────────

class ShieldStatusSnapshotDto {
  final bool connected;
  final String activeTransport;
  final int latencyMs;
  final int bytesSent;
  final int bytesRecv;
  final int dpiThreatLevel;
  final int failoverCount;
  final int batteryPct;
  final double healthScore;
  final bool nainActive;
  final String activeCore;
  final String ispName;
  final String threatLevel;
  final int uptimeSecs;

  const ShieldStatusSnapshotDto({
    required this.connected,
    this.activeTransport = 'none',
    this.latencyMs = 0,
    this.bytesSent = 0,
    this.bytesRecv = 0,
    this.dpiThreatLevel = 0,
    this.failoverCount = 0,
    this.batteryPct = 100,
    this.healthScore = 1.0,
    this.nainActive = false,
    this.activeCore = 'none',
    this.ispName = 'unknown',
    this.threatLevel = 'Low',
    this.uptimeSecs = 0,
  });
}

// ── ShieldEvent sealed hierarchy ─────────────────────────────────────────────

abstract class ShieldEventDto {}

class StatusUpdateEvent extends ShieldEventDto {
  final ShieldStatusSnapshotDto snapshot;
  StatusUpdateEvent({required this.snapshot});
}

class TransportChangedEvent extends ShieldEventDto {
  final String from;
  final String to;
  final String reason;
  final int failoverLatencyMs;
  TransportChangedEvent({
    required this.from,
    required this.to,
    required this.reason,
    required this.failoverLatencyMs,
  });
}

class CoreChangedEvent extends ShieldEventDto {
  final String from;
  final String to;
  final String reason;
  CoreChangedEvent({required this.from, required this.to, required this.reason});
}

class DpiAlertEvent extends ShieldEventDto {
  final int threatLevel;
  final String description;
  final String ispName;
  DpiAlertEvent({
    required this.threatLevel,
    required this.description,
    required this.ispName,
  });
}

class NainStatusChangedEvent extends ShieldEventDto {
  final bool active;
  final String mode;
  NainStatusChangedEvent({required this.active, required this.mode});
}

class LicenseWarningEvent extends ShieldEventDto {
  final String message;
  LicenseWarningEvent({required this.message});
}

class SubsystemRestartedEvent extends ShieldEventDto {
  final String subsystem;
  final String reason;
  SubsystemRestartedEvent({required this.subsystem, required this.reason});
}

class IspDetectedEvent extends ShieldEventDto {
  final String ispName;
  final String countryCode;
  final int censorshipLevel;
  IspDetectedEvent({
    required this.ispName,
    required this.countryCode,
    required this.censorshipLevel,
  });
}

class IdentityRotatedEvent extends ShieldEventDto {
  final String newPublicKeyHex;
  IdentityRotatedEvent({required this.newPublicKeyHex});
}

class ErrorEvent extends ShieldEventDto {
  final int code;
  final String message;
  ErrorEvent({required this.code, required this.message});
}

// ── ShieldCommand sealed hierarchy ───────────────────────────────────────────

abstract class ShieldCommandDto {
  const ShieldCommandDto();

  const factory ShieldCommandDto.connect({
    String? preferredTransport,
    String? preferredCore,
  }) = _ConnectCommand;

  const factory ShieldCommandDto.disconnect() = _DisconnectCommand;

  const factory ShieldCommandDto.forceTransport({required String name}) =
      _ForceTransportCommand;

  const factory ShieldCommandDto.forceCore({required String name}) =
      _ForceCoreCommand;

  const factory ShieldCommandDto.emergencyWipe({required String authToken}) =
      _EmergencyWipeCommand;

  const factory ShieldCommandDto.rotateIdentity() = _RotateIdentityCommand;

  const factory ShieldCommandDto.configUpdate({
    required String key,
    required String value,
  }) = _ConfigUpdateCommand;
}

class _ConnectCommand extends ShieldCommandDto {
  final String? preferredTransport;
  final String? preferredCore;
  const _ConnectCommand({this.preferredTransport, this.preferredCore});
}

class _DisconnectCommand extends ShieldCommandDto {
  const _DisconnectCommand();
}

class _ForceTransportCommand extends ShieldCommandDto {
  final String name;
  const _ForceTransportCommand({required this.name});
}

class _ForceCoreCommand extends ShieldCommandDto {
  final String name;
  const _ForceCoreCommand({required this.name});
}

class _EmergencyWipeCommand extends ShieldCommandDto {
  final String authToken;
  const _EmergencyWipeCommand({required this.authToken});
}

class _RotateIdentityCommand extends ShieldCommandDto {
  const _RotateIdentityCommand();
}

class _ConfigUpdateCommand extends ShieldCommandDto {
  final String key;
  final String value;
  const _ConfigUpdateCommand({required this.key, required this.value});
}

// ── Stub API functions — replaced by FRB codegen ──────────────────────────────

final StreamController<ShieldEventDto> _stubEventController =
    StreamController<ShieldEventDto>.broadcast();

/// Initialise the daemon from a JSON config string.
Future<void> shieldInit(String configJson) async {}

/// Send a command to the daemon.
Future<void> shieldCommand(ShieldCommandDto cmd) async {}

/// Returns a stream of `ShieldEventDto` items from the Rust daemon.
Stream<ShieldEventDto> shieldEventStream() => _stubEventController.stream;

/// Synchronous status snapshot for initial render.
ShieldStatusSnapshotDto shieldStatusSync() =>
    const ShieldStatusSnapshotDto(connected: false);

/// List of all available transport names from the daemon.
List<String> shieldAvailableTransports() => [
      'VLESS+REALITY',
      'VLESS+WS+TLS',
      'VMess+WS+TLS',
      'Trojan+TLS',
      'Hysteria2',
      'TUIC-v5',
      'ShadowTLS',
      'Shadowsocks',
      'Psiphon',
      'Tor+Obfs4',
      'Meek+CDN',
      'AmneziaWG',
      'Slipstream-QUIC',
      'DNSTT',
    ];

/// List of all available VPN core names from the daemon.
List<String> shieldAvailableCores() =>
    ['xray', 'sing-box', 'hiddify', 'psiphon', 'tor', 'custom-tunnel'];

/// Inject an event into the stub stream (for unit tests only).
void injectTestEvent(ShieldEventDto event) {
  _stubEventController.add(event);
}
