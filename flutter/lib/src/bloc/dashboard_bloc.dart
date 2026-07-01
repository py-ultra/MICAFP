// ─────────────────────────────────────────────────────────────────────────────
// MICAFP-UnifiedShield v10.0 — DashboardBloc
//
// TASK-04 implementation.
// The Flutter/BLoC layer is a Monitor and Remote Control only.
// It NEVER participates in transport selection, failover timing,
// ping measurement, or any network decision. The Rust daemon is sole brain.
//
// Architecture:
//   Rust daemon emits ShieldEvent (via FRB stream)
//       ↓  sub-millisecond, zero-copy
//   DashboardBloc receives → updates DashboardState
//       ↓
//   Flutter widgets rebuild → user sees result
// ─────────────────────────────────────────────────────────────────────────────

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:equatable/equatable.dart';

import '../bridge/shield_bridge_stub.dart';

// ═══════════════════════════════════════════════════════════════════════════
// Events
// ═══════════════════════════════════════════════════════════════════════════

abstract class DashboardEvent extends Equatable {
  const DashboardEvent();
  @override
  List<Object?> get props => [];
}

/// Bootstrap: subscribe to Rust event stream and seed initial state.
class DashboardStarted extends DashboardEvent {}

/// User tapped Connect (optional transport/core preference).
class DashboardConnectRequested extends DashboardEvent {
  final String? preferredTransport;
  final String? preferredCore;
  const DashboardConnectRequested({this.preferredTransport, this.preferredCore});
  @override
  List<Object?> get props => [preferredTransport, preferredCore];
}

/// User tapped Disconnect.
class DashboardDisconnectRequested extends DashboardEvent {}

/// User manually selected a transport from the protocol switcher.
class DashboardForceTransport extends DashboardEvent {
  final String transportName;
  const DashboardForceTransport(this.transportName);
  @override
  List<Object?> get props => [transportName];
}

/// User manually selected a VPN core.
class DashboardForceCore extends DashboardEvent {
  final String coreName;
  const DashboardForceCore(this.coreName);
  @override
  List<Object?> get props => [coreName];
}

/// User triggered emergency wipe.
class DashboardEmergencyWipe extends DashboardEvent {
  final String authToken;
  const DashboardEmergencyWipe(this.authToken);
  @override
  List<Object?> get props => [authToken];
}

/// User triggered identity rotation.
class DashboardRotateIdentity extends DashboardEvent {}

/// Internal: incoming ShieldEvent from the Rust daemon stream.
/// Never dispatched by user code — only by the BLoC's own stream subscription.
class _RustEvent extends DashboardEvent {
  final ShieldEventDto event;
  const _RustEvent(this.event);
  @override
  List<Object?> get props => [event];
}

// ═══════════════════════════════════════════════════════════════════════════
// State
// ═══════════════════════════════════════════════════════════════════════════

enum ConnectionPhase { idle, connecting, connected, disconnecting, error }

/// Immutable record of a single automatic transport failover event.
class TransportChangeRecord extends Equatable {
  final String from;
  final String to;
  final String reason;
  final int failoverLatencyMs;
  final DateTime at;

  const TransportChangeRecord({
    required this.from,
    required this.to,
    required this.reason,
    required this.failoverLatencyMs,
    required this.at,
  });

  @override
  List<Object?> get props => [from, to, reason, failoverLatencyMs, at];
}

/// Immutable record of a DPI alert event.
class DpiAlertRecord extends Equatable {
  final int threatLevel;
  final String description;
  final String ispName;
  final DateTime at;

  const DpiAlertRecord({
    required this.threatLevel,
    required this.description,
    required this.ispName,
    required this.at,
  });

  @override
  List<Object?> get props => [threatLevel, description, ispName, at];
}

/// Full observable state emitted by DashboardBloc.
class DashboardState extends Equatable {
  final ConnectionPhase phase;
  final ShieldStatusSnapshotDto? snapshot;
  final String? lastError;

  /// Last 20 automatic failover events (newest first).
  final List<TransportChangeRecord> recentFailovers;

  /// Last 10 DPI alert events (newest first).
  final List<DpiAlertRecord> recentDpiAlerts;

  /// List of all transport names available for the protocol switcher.
  final List<String> availableTransports;

  /// List of all VPN core names available for the core switcher.
  final List<String> availableCores;

  /// Whether an identity rotation is in progress.
  final bool rotatingIdentity;

  const DashboardState({
    required this.phase,
    this.snapshot,
    this.lastError,
    this.recentFailovers = const [],
    this.recentDpiAlerts = const [],
    this.availableTransports = const [],
    this.availableCores = const [],
    this.rotatingIdentity = false,
  });

  DashboardState copyWith({
    ConnectionPhase? phase,
    ShieldStatusSnapshotDto? snapshot,
    String? lastError,
    List<TransportChangeRecord>? recentFailovers,
    List<DpiAlertRecord>? recentDpiAlerts,
    List<String>? availableTransports,
    List<String>? availableCores,
    bool? rotatingIdentity,
    bool clearError = false,
  }) =>
      DashboardState(
        phase: phase ?? this.phase,
        snapshot: snapshot ?? this.snapshot,
        lastError: clearError ? null : (lastError ?? this.lastError),
        recentFailovers: recentFailovers ?? this.recentFailovers,
        recentDpiAlerts: recentDpiAlerts ?? this.recentDpiAlerts,
        availableTransports: availableTransports ?? this.availableTransports,
        availableCores: availableCores ?? this.availableCores,
        rotatingIdentity: rotatingIdentity ?? this.rotatingIdentity,
      );

  @override
  List<Object?> get props => [
        phase,
        snapshot,
        lastError,
        recentFailovers,
        recentDpiAlerts,
        availableTransports,
        availableCores,
        rotatingIdentity,
      ];
}

// ═══════════════════════════════════════════════════════════════════════════
// BLoC
// ═══════════════════════════════════════════════════════════════════════════

class DashboardBloc extends Bloc<DashboardEvent, DashboardState> {
  StreamSubscription<ShieldEventDto>? _rustSub;

  DashboardBloc()
      : super(const DashboardState(phase: ConnectionPhase.idle)) {
    on<DashboardStarted>(_onStarted);
    on<DashboardConnectRequested>(_onConnect);
    on<DashboardDisconnectRequested>(_onDisconnect);
    on<DashboardForceTransport>(_onForceTransport);
    on<DashboardForceCore>(_onForceCore);
    on<DashboardEmergencyWipe>(_onEmergencyWipe);
    on<DashboardRotateIdentity>(_onRotateIdentity);
    on<_RustEvent>(_onRustEvent);
  }

  // ── Handlers ──────────────────────────────────────────────────────────────

  Future<void> _onStarted(
    DashboardStarted _,
    Emitter<DashboardState> emit,
  ) async {
    // Seed initial state from synchronous snapshot — no async round-trip.
    final snap = shieldStatusSync();
    final transports = shieldAvailableTransports();
    final cores = shieldAvailableCores();

    emit(state.copyWith(
      snapshot: snap,
      phase: snap.connected ? ConnectionPhase.connected : ConnectionPhase.idle,
      availableTransports: transports,
      availableCores: cores,
      clearError: true,
    ));

    // Subscribe to the Rust event stream.
    // Events are pushed by the daemon AI engine without any Dart involvement.
    _rustSub = shieldEventStream().listen(
      (event) => add(_RustEvent(event)),
      onError: (Object e) => add(
        _RustEvent(ShieldEventDto.error(code: 9999, message: e.toString())),
      ),
    );
  }

  Future<void> _onConnect(
    DashboardConnectRequested event,
    Emitter<DashboardState> emit,
  ) async {
    emit(state.copyWith(phase: ConnectionPhase.connecting, clearError: true));
    await shieldCommand(ShieldCommandDto.connect(
      preferredTransport: event.preferredTransport,
      preferredCore: event.preferredCore,
    ));
    // State transitions happen automatically via the Rust event stream.
  }

  Future<void> _onDisconnect(
    DashboardDisconnectRequested _,
    Emitter<DashboardState> emit,
  ) async {
    emit(state.copyWith(phase: ConnectionPhase.disconnecting));
    await shieldCommand(const ShieldCommandDto.disconnect());
  }

  Future<void> _onForceTransport(
    DashboardForceTransport event,
    Emitter<DashboardState> emit,
  ) async {
    await shieldCommand(
      ShieldCommandDto.forceTransport(name: event.transportName),
    );
  }

  Future<void> _onForceCore(
    DashboardForceCore event,
    Emitter<DashboardState> emit,
  ) async {
    await shieldCommand(ShieldCommandDto.forceCore(name: event.coreName));
  }

  Future<void> _onEmergencyWipe(
    DashboardEmergencyWipe event,
    Emitter<DashboardState> emit,
  ) async {
    await shieldCommand(
      ShieldCommandDto.emergencyWipe(authToken: event.authToken),
    );
  }

  Future<void> _onRotateIdentity(
    DashboardRotateIdentity _,
    Emitter<DashboardState> emit,
  ) async {
    emit(state.copyWith(rotatingIdentity: true));
    await shieldCommand(const ShieldCommandDto.rotateIdentity());
  }

  void _onRustEvent(
    _RustEvent event,
    Emitter<DashboardState> emit,
  ) {
    final e = event.event;

    if (e is StatusUpdateEvent) {
      emit(state.copyWith(
        snapshot: e.snapshot,
        phase: e.snapshot.connected
            ? ConnectionPhase.connected
            : ConnectionPhase.idle,
        clearError: true,
      ));
    } else if (e is TransportChangedEvent) {
      final record = TransportChangeRecord(
        from: e.from,
        to: e.to,
        reason: e.reason,
        failoverLatencyMs: e.failoverLatencyMs,
        at: DateTime.now(),
      );
      final updated = [record, ...state.recentFailovers].take(20).toList();
      emit(state.copyWith(recentFailovers: updated));
    } else if (e is DpiAlertEvent) {
      final record = DpiAlertRecord(
        threatLevel: e.threatLevel,
        description: e.description,
        ispName: e.ispName,
        at: DateTime.now(),
      );
      final updated = [record, ...state.recentDpiAlerts].take(10).toList();
      emit(state.copyWith(recentDpiAlerts: updated));
    } else if (e is IdentityRotatedEvent) {
      emit(state.copyWith(rotatingIdentity: false));
    } else if (e is ErrorEvent) {
      emit(state.copyWith(
        phase: ConnectionPhase.error,
        lastError: '[${e.code}] ${e.message}',
      ));
    }
    // NainStatusChanged, CoreChanged, LicenseWarning, etc. are surfaced via
    // the snapshot in StatusUpdate — no separate state fields needed.
  }

  @override
  Future<void> close() {
    _rustSub?.cancel();
    return super.close();
  }
}
