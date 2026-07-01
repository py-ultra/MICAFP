// ─────────────────────────────────────────────────────────────────────────────
// MICAFP-UnifiedShield v10.0 — Advanced Dashboard Screen
//
// Fully BLoC-driven. The Rust daemon's event stream drives all state changes.
// This screen only renders state and relays user commands to the BLoC.
// It never makes network calls or transport decisions directly.
// ─────────────────────────────────────────────────────────────────────────────

import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../bloc/dashboard_bloc.dart';
import '../widgets/advanced/protocol_switcher_widget.dart';
import '../widgets/advanced/dpi_alert_panel.dart';
import '../widgets/advanced/failover_history_widget.dart';

class AdvancedDashboardScreen extends StatelessWidget {
  const AdvancedDashboardScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Shield'),
        actions: [
          BlocBuilder<DashboardBloc, DashboardState>(
            buildWhen: (p, c) => p.snapshot?.ispName != c.snapshot?.ispName,
            builder: (context, state) {
              final isp = state.snapshot?.ispName ?? '...';
              return Padding(
                padding: const EdgeInsets.only(right: 12),
                child: Center(
                  child: Text(
                    isp,
                    style: const TextStyle(
                      fontSize: 11,
                      color: Color(0xFF78909C),
                    ),
                  ),
                ),
              );
            },
          ),
        ],
      ),
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: Column(
            children: [
              // ── Main connection button + status ──────────────────────
              const _ConnectionCard(),
              const SizedBox(height: 12),

              // ── DPI alerts (only visible when active) ───────────────
              const DpiAlertPanel(),

              // ── Network metrics row ──────────────────────────────────
              const _MetricsRow(),
              const SizedBox(height: 12),

              // ── Protocol switcher (manual override) ─────────────────
              const ProtocolSwitcherWidget(),
              const SizedBox(height: 12),

              // ── NAIN / Internet status indicator ────────────────────
              const _NainStatusCard(),
              const SizedBox(height: 12),

              // ── Auto-failover history ────────────────────────────────
              const FailoverHistoryWidget(),
              const SizedBox(height: 12),

              // ── Identity rotation button ─────────────────────────────
              const _IdentityRotateButton(),
              const SizedBox(height: 20),
            ],
          ),
        ),
      ),
    );
  }
}

// ── Connection Card ────────────────────────────────────────────────────────

class _ConnectionCard extends StatelessWidget {
  const _ConnectionCard();

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<DashboardBloc, DashboardState>(
      buildWhen: (prev, curr) =>
          prev.phase != curr.phase ||
          prev.snapshot != curr.snapshot,
      builder: (context, state) {
        final phase = state.phase;
        final snap = state.snapshot;
        final isConnected = phase == ConnectionPhase.connected;
        final isLoading = phase == ConnectionPhase.connecting ||
            phase == ConnectionPhase.disconnecting;

        return Card(
          child: Padding(
            padding: const EdgeInsets.all(20),
            child: Column(
              children: [
                // Status orb
                _StatusOrb(phase: phase),
                const SizedBox(height: 16),

                // Status text
                Text(
                  _phaseLabel(phase),
                  style: TextStyle(
                    fontSize: 22,
                    fontWeight: FontWeight.w700,
                    color: _phaseColor(phase),
                  ),
                ),

                if (isConnected && snap != null) ...[
                  const SizedBox(height: 4),
                  Text(
                    '${snap.activeCore} · ${snap.activeTransport}',
                    style: const TextStyle(
                      color: Color(0xFF78909C),
                      fontSize: 13,
                    ),
                  ),
                  const SizedBox(height: 4),
                  Text(
                    '${snap.latencyMs} ms  ·  ${snap.failoverCount} failovers',
                    style: const TextStyle(
                      color: Color(0xFF546E7A),
                      fontSize: 12,
                    ),
                  ),
                ],

                const SizedBox(height: 20),

                // Connect / Disconnect button
                SizedBox(
                  width: double.infinity,
                  child: ElevatedButton(
                    onPressed: isLoading
                        ? null
                        : () => isConnected
                            ? context
                                .read<DashboardBloc>()
                                .add(DashboardDisconnectRequested())
                            : context
                                .read<DashboardBloc>()
                                .add(const DashboardConnectRequested()),
                    style: ElevatedButton.styleFrom(
                      backgroundColor:
                          isConnected ? const Color(0xFFC62828) : null,
                      padding: const EdgeInsets.symmetric(vertical: 16),
                    ),
                    child: isLoading
                        ? const SizedBox(
                            height: 20,
                            width: 20,
                            child: CircularProgressIndicator(
                              strokeWidth: 2,
                              color: Colors.white,
                            ),
                          )
                        : Text(isConnected ? 'Disconnect' : 'Connect'),
                  ),
                ),

                // Error message
                if (phase == ConnectionPhase.error &&
                    state.lastError != null) ...[
                  const SizedBox(height: 12),
                  Text(
                    state.lastError!,
                    style: const TextStyle(
                      color: Color(0xFFEF5350),
                      fontSize: 12,
                    ),
                    textAlign: TextAlign.center,
                  ),
                ],
              ],
            ),
          ),
        );
      },
    );
  }

  String _phaseLabel(ConnectionPhase phase) => switch (phase) {
        ConnectionPhase.idle => 'Disconnected',
        ConnectionPhase.connecting => 'Connecting...',
        ConnectionPhase.connected => 'Protected',
        ConnectionPhase.disconnecting => 'Disconnecting...',
        ConnectionPhase.error => 'Error',
      };

  Color _phaseColor(ConnectionPhase phase) => switch (phase) {
        ConnectionPhase.connected => const Color(0xFF4CAF50),
        ConnectionPhase.connecting ||
        ConnectionPhase.disconnecting =>
          const Color(0xFFFFA726),
        ConnectionPhase.error => const Color(0xFFEF5350),
        _ => const Color(0xFF78909C),
      };
}

// ── Animated status orb ────────────────────────────────────────────────────

class _StatusOrb extends StatelessWidget {
  final ConnectionPhase phase;
  const _StatusOrb({required this.phase});

  @override
  Widget build(BuildContext context) {
    final color = switch (phase) {
      ConnectionPhase.connected => const Color(0xFF4CAF50),
      ConnectionPhase.connecting ||
      ConnectionPhase.disconnecting =>
        const Color(0xFFFFA726),
      ConnectionPhase.error => const Color(0xFFEF5350),
      _ => const Color(0xFF37474F),
    };

    final isPulsing = phase == ConnectionPhase.connected;
    final isSpinning = phase == ConnectionPhase.connecting ||
        phase == ConnectionPhase.disconnecting;

    return Container(
      width: 80,
      height: 80,
      decoration: BoxDecoration(
        shape: BoxShape.circle,
        color: color.withOpacity(0.12),
        border: Border.all(color: color.withOpacity(0.4), width: 2),
      ),
      child: Center(
        child: Icon(
          phase == ConnectionPhase.connected
              ? Icons.shield
              : Icons.shield_outlined,
          color: color,
          size: 36,
        ),
      ),
    )
        .animate(target: isPulsing ? 1 : 0)
        .scaleXY(begin: 1.0, end: 1.06, duration: 1200.ms)
        .then()
        .scaleXY(begin: 1.06, end: 1.0, duration: 1200.ms)
        .animate(target: isSpinning ? 1 : 0)
        .rotate(duration: 1500.ms);
  }
}

// ── Network Metrics Row ────────────────────────────────────────────────────

class _MetricsRow extends StatelessWidget {
  const _MetricsRow();

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<DashboardBloc, DashboardState>(
      buildWhen: (p, c) =>
          p.snapshot?.bytesSent != c.snapshot?.bytesSent ||
          p.snapshot?.bytesRecv != c.snapshot?.bytesRecv ||
          p.snapshot?.healthScore != c.snapshot?.healthScore,
      builder: (context, state) {
        final snap = state.snapshot;
        if (snap == null) return const SizedBox.shrink();
        return Row(
          children: [
            Expanded(
              child: _MetricCard(
                icon: Icons.arrow_upward,
                label: '↑',
                value: _fmt(snap.bytesSent),
                color: const Color(0xFF5C6BC0),
              ),
            ),
            const SizedBox(width: 8),
            Expanded(
              child: _MetricCard(
                icon: Icons.arrow_downward,
                label: '↓',
                value: _fmt(snap.bytesRecv),
                color: const Color(0xFF26A69A),
              ),
            ),
            const SizedBox(width: 8),
            Expanded(
              child: _MetricCard(
                icon: Icons.favorite_border,
                label: 'Health',
                value: '${(snap.healthScore * 100).round()}%',
                color: snap.healthScore > 0.7
                    ? const Color(0xFF4CAF50)
                    : const Color(0xFFFFA726),
              ),
            ),
          ],
        );
      },
    );
  }

  String _fmt(int bytes) {
    if (bytes < 1024) return '$bytes B';
    if (bytes < 1048576) return '${(bytes / 1024).toStringAsFixed(1)} KB';
    if (bytes < 1073741824) return '${(bytes / 1048576).toStringAsFixed(1)} MB';
    return '${(bytes / 1073741824).toStringAsFixed(2)} GB';
  }
}

class _MetricCard extends StatelessWidget {
  final IconData icon;
  final String label;
  final String value;
  final Color color;

  const _MetricCard({
    required this.icon,
    required this.label,
    required this.value,
    required this.color,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 12, horizontal: 8),
        child: Column(
          children: [
            Icon(icon, color: color, size: 16),
            const SizedBox(height: 4),
            Text(
              value,
              style: TextStyle(
                color: color,
                fontWeight: FontWeight.w700,
                fontSize: 13,
              ),
            ),
            Text(
              label,
              style: const TextStyle(color: Colors.grey, fontSize: 10),
            ),
          ],
        ),
      ),
    );
  }
}

// ── NAIN Status Card ────────────────────────────────────────────────────────

class _NainStatusCard extends StatelessWidget {
  const _NainStatusCard();

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<DashboardBloc, DashboardState>(
      buildWhen: (p, c) => p.snapshot?.nainActive != c.snapshot?.nainActive,
      builder: (context, state) {
        final nainActive = state.snapshot?.nainActive ?? false;
        return Card(
          color: nainActive
              ? const Color(0xFFE65100).withOpacity(0.08)
              : null,
          child: ListTile(
            leading: Icon(
              nainActive ? Icons.domain : Icons.language,
              color: nainActive
                  ? const Color(0xFFFF7043)
                  : const Color(0xFF2E7D32),
            ),
            title: Text(
              nainActive ? 'National Intranet Mode' : 'Full Internet',
              style: TextStyle(
                color: nainActive
                    ? const Color(0xFFFF7043)
                    : const Color(0xFF81C784),
                fontWeight: FontWeight.w600,
                fontSize: 14,
              ),
            ),
            subtitle: Text(
              nainActive
                  ? 'Mesh and alternative channels active'
                  : 'All channels operational',
              style: const TextStyle(fontSize: 12),
            ),
            trailing: nainActive
                ? const Icon(Icons.warning_amber, color: Color(0xFFFFA726))
                : null,
          ),
        );
      },
    );
  }
}

// ── Identity Rotation Button ────────────────────────────────────────────────

class _IdentityRotateButton extends StatelessWidget {
  const _IdentityRotateButton();

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<DashboardBloc, DashboardState>(
      buildWhen: (p, c) => p.rotatingIdentity != c.rotatingIdentity,
      builder: (context, state) {
        return OutlinedButton.icon(
          onPressed: state.rotatingIdentity
              ? null
              : () => context
                  .read<DashboardBloc>()
                  .add(DashboardRotateIdentity()),
          icon: state.rotatingIdentity
              ? const SizedBox(
                  width: 16,
                  height: 16,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : const Icon(Icons.refresh, size: 18),
          label: Text(
            state.rotatingIdentity ? 'Rotating...' : 'Rotate Identity (PQC)',
          ),
          style: OutlinedButton.styleFrom(
            minimumSize: const Size(double.infinity, 48),
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(12),
            ),
          ),
        );
      },
    );
  }
}
