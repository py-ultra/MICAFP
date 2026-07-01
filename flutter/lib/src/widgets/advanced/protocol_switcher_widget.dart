// ─────────────────────────────────────────────────────────────────────────────
// MICAFP-UnifiedShield v10.0 — Protocol Switcher Widget
//
// Displays all available transports from the daemon and allows manual
// override. The Rust AI engine is the default selector; this widget gives
// users an explicit override when needed. Wired to DashboardBloc.
// ─────────────────────────────────────────────────────────────────────────────

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_animate/flutter_animate.dart';

import '../../bloc/dashboard_bloc.dart';

class ProtocolSwitcherWidget extends StatelessWidget {
  const ProtocolSwitcherWidget({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<DashboardBloc, DashboardState>(
      buildWhen: (prev, curr) =>
          prev.availableTransports != curr.availableTransports ||
          prev.snapshot?.activeTransport != curr.snapshot?.activeTransport ||
          prev.phase != curr.phase,
      builder: (context, state) {
        final activeTransport = state.snapshot?.activeTransport ?? 'none';
        final transports = state.availableTransports;
        final isConnected = state.phase == ConnectionPhase.connected;

        return Card(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    const Icon(
                      Icons.swap_horiz,
                      color: Color(0xFF5C6BC0),
                      size: 20,
                    ),
                    const SizedBox(width: 8),
                    Text(
                      'Protocol',
                      style: Theme.of(context).textTheme.titleMedium?.copyWith(
                            fontWeight: FontWeight.w600,
                          ),
                    ),
                    const Spacer(),
                    Container(
                      padding: const EdgeInsets.symmetric(
                          horizontal: 8, vertical: 4),
                      decoration: BoxDecoration(
                        color: const Color(0xFF5C6BC0).withOpacity(0.15),
                        borderRadius: BorderRadius.circular(8),
                      ),
                      child: Text(
                        'AI AUTO',
                        style: const TextStyle(
                          color: Color(0xFF7986CB),
                          fontSize: 10,
                          fontWeight: FontWeight.w700,
                          letterSpacing: 0.5,
                        ),
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 12),
                // Currently active transport indicator
                _ActiveTransportBadge(
                  name: activeTransport,
                  isAuto: !state.recentFailovers.isEmpty ||
                      state.phase == ConnectionPhase.connected,
                ),
                const SizedBox(height: 12),
                // Transport grid for manual override
                if (transports.isNotEmpty)
                  SizedBox(
                    height: 36,
                    child: ListView.separated(
                      scrollDirection: Axis.horizontal,
                      itemCount: transports.length,
                      separatorBuilder: (_, __) => const SizedBox(width: 8),
                      itemBuilder: (context, index) {
                        final t = transports[index];
                        final isActive = t == activeTransport;
                        return _TransportChip(
                          name: t,
                          isActive: isActive,
                          enabled: isConnected,
                          onTap: () => context
                              .read<DashboardBloc>()
                              .add(DashboardForceTransport(t)),
                        );
                      },
                    ),
                  ),
                if (!isConnected)
                  Padding(
                    padding: const EdgeInsets.only(top: 8),
                    child: Text(
                      'Connect first to enable manual override',
                      style: Theme.of(context).textTheme.bodySmall?.copyWith(
                            color: Colors.grey,
                          ),
                    ),
                  ),
              ],
            ),
          ),
        );
      },
    );
  }
}

class _ActiveTransportBadge extends StatelessWidget {
  final String name;
  final bool isAuto;

  const _ActiveTransportBadge({required this.name, required this.isAuto});

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: const Color(0xFF2E7D32).withOpacity(0.12),
        borderRadius: BorderRadius.circular(10),
        border: Border.all(
          color: const Color(0xFF2E7D32).withOpacity(0.4),
        ),
      ),
      child: Row(
        children: [
          const Icon(Icons.bolt, color: Color(0xFF66BB6A), size: 16),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              name,
              style: const TextStyle(
                color: Color(0xFF81C784),
                fontWeight: FontWeight.w600,
                fontSize: 13,
              ),
            ),
          ),
          if (isAuto)
            const Text(
              'AUTO',
              style: TextStyle(
                color: Color(0xFF66BB6A),
                fontSize: 10,
                fontWeight: FontWeight.w700,
              ),
            ),
        ],
      ),
    )
        .animate(onPlay: (c) => c.loop(reverse: true))
        .shimmer(
          duration: const Duration(seconds: 3),
          color: const Color(0xFF2E7D32).withOpacity(0.15),
        );
  }
}

class _TransportChip extends StatelessWidget {
  final String name;
  final bool isActive;
  final bool enabled;
  final VoidCallback onTap;

  const _TransportChip({
    required this.name,
    required this.isActive,
    required this.enabled,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: enabled ? onTap : null,
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 200),
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
        decoration: BoxDecoration(
          color: isActive
              ? const Color(0xFF2E7D32).withOpacity(0.25)
              : const Color(0xFF37474F).withOpacity(0.4),
          borderRadius: BorderRadius.circular(18),
          border: Border.all(
            color: isActive
                ? const Color(0xFF2E7D32)
                : Colors.transparent,
          ),
        ),
        child: Text(
          name,
          style: TextStyle(
            color: isActive
                ? const Color(0xFF81C784)
                : enabled
                    ? const Color(0xFFB0BEC5)
                    : Colors.grey.shade700,
            fontSize: 12,
            fontWeight:
                isActive ? FontWeight.w700 : FontWeight.normal,
          ),
        ),
      ),
    );
  }
}
