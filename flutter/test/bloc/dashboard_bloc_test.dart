// ─────────────────────────────────────────────────────────────────────────────
// MICAFP-UnifiedShield v10.0 — DashboardBloc Unit Tests (TASK-05)
//
// Tests the BLoC state machine without requiring the Rust daemon.
// All Rust interactions go through the stub bridge (shield_bridge_stub.dart).
// Run with: flutter test test/bloc/dashboard_bloc_test.dart
// ─────────────────────────────────────────────────────────────────────────────

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shield/src/bloc/dashboard_bloc.dart';
import 'package:shield/src/bridge/shield_bridge_stub.dart';

void main() {
  group('DashboardBloc', () {
    // ── Initial state ────────────────────────────────────────────────────
    test('initial state is idle with no snapshot', () {
      final bloc = DashboardBloc();
      expect(bloc.state.phase, ConnectionPhase.idle);
      expect(bloc.state.snapshot, isNull);
      bloc.close();
    });

    // ── StatusUpdate → connected ─────────────────────────────────────────
    blocTest<DashboardBloc, DashboardState>(
      'emits connected phase when StatusUpdate has connected=true',
      build: () => DashboardBloc(),
      act: (bloc) {
        injectTestEvent(StatusUpdateEvent(
          snapshot: const ShieldStatusSnapshotDto(
            connected: true,
            activeTransport: 'VLESS+REALITY',
            latencyMs: 42,
            bytesSent: 1024,
            bytesRecv: 2048,
            dpiThreatLevel: 0,
            failoverCount: 0,
            batteryPct: 90,
            healthScore: 0.95,
            activeCore: 'xray',
            ispName: 'MCI',
            threatLevel: 'Low',
            uptimeSecs: 120,
          ),
        ));
      },
      expect: () => [
        isA<DashboardState>().having(
          (s) => s.phase,
          'phase',
          ConnectionPhase.connected,
        ),
        isA<DashboardState>().having(
          (s) => s.snapshot?.activeTransport,
          'activeTransport',
          'VLESS+REALITY',
        ),
      ],
      wait: const Duration(milliseconds: 100),
    );

    // ── StatusUpdate → disconnected ──────────────────────────────────────
    blocTest<DashboardBloc, DashboardState>(
      'emits idle phase when StatusUpdate has connected=false',
      build: () => DashboardBloc(),
      act: (bloc) {
        injectTestEvent(StatusUpdateEvent(
          snapshot: const ShieldStatusSnapshotDto(connected: false),
        ));
      },
      expect: () => [
        isA<DashboardState>().having(
          (s) => s.phase,
          'phase',
          ConnectionPhase.idle,
        ),
      ],
      wait: const Duration(milliseconds: 100),
    );

    // ── TransportChanged → recentFailovers updated ───────────────────────
    blocTest<DashboardBloc, DashboardState>(
      'appends TransportChanged to recentFailovers',
      build: () => DashboardBloc(),
      act: (bloc) {
        injectTestEvent(TransportChangedEvent(
          from: 'VLESS',
          to: 'Hysteria2',
          reason: 'DPI detected',
          failoverLatencyMs: 85,
        ));
      },
      expect: () => [
        isA<DashboardState>().having(
          (s) => s.recentFailovers.first.to,
          'failover.to',
          'Hysteria2',
        ),
        isA<DashboardState>().having(
          (s) => s.recentFailovers.first.failoverLatencyMs,
          'failover.latencyMs',
          85,
        ),
      ],
      wait: const Duration(milliseconds: 100),
    );

    // ── recentFailovers capped at 20 ─────────────────────────────────────
    blocTest<DashboardBloc, DashboardState>(
      'caps recentFailovers list at 20 entries',
      build: () => DashboardBloc(),
      act: (bloc) {
        for (int i = 0; i < 25; i++) {
          injectTestEvent(TransportChangedEvent(
            from: 'A$i',
            to: 'B$i',
            reason: 'test',
            failoverLatencyMs: i,
          ));
        }
      },
      expect: () => [
        ...List.generate(
          25,
          (i) => isA<DashboardState>(),
        ),
      ],
      verify: (bloc) {
        expect(bloc.state.recentFailovers.length, lessThanOrEqualTo(20));
      },
      wait: const Duration(milliseconds: 200),
    );

    // ── DpiAlert → recentDpiAlerts updated ──────────────────────────────
    blocTest<DashboardBloc, DashboardState>(
      'appends DpiAlert to recentDpiAlerts',
      build: () => DashboardBloc(),
      act: (bloc) {
        injectTestEvent(DpiAlertEvent(
          threatLevel: 3,
          description: 'DPI signature: VLESS probe',
          ispName: 'Irancell',
        ));
      },
      expect: () => [
        isA<DashboardState>().having(
          (s) => s.recentDpiAlerts.first.threatLevel,
          'dpiAlert.threatLevel',
          3,
        ),
        isA<DashboardState>().having(
          (s) => s.recentDpiAlerts.first.ispName,
          'dpiAlert.ispName',
          'Irancell',
        ),
      ],
      wait: const Duration(milliseconds: 100),
    );

    // ── ErrorEvent → error phase ──────────────────────────────────────────
    blocTest<DashboardBloc, DashboardState>(
      'emits error phase on ErrorEvent',
      build: () => DashboardBloc(),
      act: (bloc) {
        injectTestEvent(ErrorEvent(code: 1001, message: 'TUN attach failed'));
      },
      expect: () => [
        isA<DashboardState>().having(
          (s) => s.phase,
          'phase',
          ConnectionPhase.error,
        ),
        isA<DashboardState>().having(
          (s) => s.lastError,
          'lastError',
          '[1001] TUN attach failed',
        ),
      ],
      wait: const Duration(milliseconds: 100),
    );

    // ── Connect command → connecting phase ─────────────────────────────
    blocTest<DashboardBloc, DashboardState>(
      'emits connecting phase when ConnectRequested',
      build: () => DashboardBloc(),
      act: (bloc) => bloc.add(const DashboardConnectRequested(
        preferredTransport: 'Hysteria2',
        preferredCore: 'sing-box',
      )),
      expect: () => [
        isA<DashboardState>().having(
          (s) => s.phase,
          'phase',
          ConnectionPhase.connecting,
        ),
      ],
    );

    // ── Disconnect command → disconnecting phase ───────────────────────
    blocTest<DashboardBloc, DashboardState>(
      'emits disconnecting phase when DisconnectRequested',
      build: () => DashboardBloc(),
      act: (bloc) => bloc.add(DashboardDisconnectRequested()),
      expect: () => [
        isA<DashboardState>().having(
          (s) => s.phase,
          'phase',
          ConnectionPhase.disconnecting,
        ),
      ],
    );

    // ── RotateIdentity → rotatingIdentity flag ────────────────────────
    blocTest<DashboardBloc, DashboardState>(
      'sets rotatingIdentity=true on RotateIdentity',
      build: () => DashboardBloc(),
      act: (bloc) => bloc.add(DashboardRotateIdentity()),
      expect: () => [
        isA<DashboardState>().having(
          (s) => s.rotatingIdentity,
          'rotatingIdentity',
          true,
        ),
      ],
    );

    // ── IdentityRotated event → rotatingIdentity cleared ─────────────
    blocTest<DashboardBloc, DashboardState>(
      'clears rotatingIdentity on IdentityRotated event',
      build: () => DashboardBloc(),
      act: (bloc) {
        bloc.add(DashboardRotateIdentity());
        injectTestEvent(
          IdentityRotatedEvent(newPublicKeyHex: 'deadbeef'),
        );
      },
      expect: () => [
        isA<DashboardState>().having(
          (s) => s.rotatingIdentity,
          'rotatingIdentity',
          true,
        ),
        isA<DashboardState>().having(
          (s) => s.rotatingIdentity,
          'rotatingIdentity',
          false,
        ),
      ],
      wait: const Duration(milliseconds: 100),
    );
  });
}
