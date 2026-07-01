// ─────────────────────────────────────────────────────────────────────────────
// MICAFP-UnifiedShield v10.0 — DPI Alert Panel Widget
//
// Displays real-time DPI detection alerts from the Rust AI classifier.
// By the time this widget renders, the daemon has already switched transports
// autonomously. This widget is purely informational.
// ─────────────────────────────────────────────────────────────────────────────

import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../bloc/dashboard_bloc.dart';

class DpiAlertPanel extends StatelessWidget {
  const DpiAlertPanel({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<DashboardBloc, DashboardState>(
      buildWhen: (prev, curr) =>
          prev.recentDpiAlerts != curr.recentDpiAlerts,
      builder: (context, state) {
        if (state.recentDpiAlerts.isEmpty) {
          return const SizedBox.shrink();
        }

        final latest = state.recentDpiAlerts.first;

        return Card(
          color: _threatColor(latest.threatLevel).withOpacity(0.08),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(16),
            side: BorderSide(
              color: _threatColor(latest.threatLevel).withOpacity(0.4),
            ),
          ),
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Icon(
                      Icons.shield_outlined,
                      color: _threatColor(latest.threatLevel),
                      size: 18,
                    ),
                    const SizedBox(width: 8),
                    Text(
                      'DPI Detection',
                      style: TextStyle(
                        color: _threatColor(latest.threatLevel),
                        fontWeight: FontWeight.w700,
                        fontSize: 13,
                      ),
                    ),
                    const Spacer(),
                    _ThreatBadge(level: latest.threatLevel),
                  ],
                ),
                const SizedBox(height: 8),
                Text(
                  latest.description,
                  style: const TextStyle(
                    color: Color(0xFFB0BEC5),
                    fontSize: 12,
                  ),
                ),
                const SizedBox(height: 4),
                Row(
                  children: [
                    const Icon(Icons.business, size: 12, color: Colors.grey),
                    const SizedBox(width: 4),
                    Text(
                      latest.ispName,
                      style: const TextStyle(
                        color: Colors.grey,
                        fontSize: 11,
                      ),
                    ),
                    const Spacer(),
                    const Text(
                      '✓ Auto-switched',
                      style: TextStyle(
                        color: Color(0xFF66BB6A),
                        fontSize: 11,
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                  ],
                ),
                if (state.recentDpiAlerts.length > 1) ...[
                  const SizedBox(height: 8),
                  Text(
                    '+${state.recentDpiAlerts.length - 1} earlier alerts',
                    style: const TextStyle(
                      color: Colors.grey,
                      fontSize: 11,
                    ),
                  ),
                ],
              ],
            ),
          ),
        ).animate().fadeIn(duration: 300.ms).slideY(begin: -0.1, end: 0);
      },
    );
  }

  Color _threatColor(int level) {
    switch (level) {
      case 1:
        return const Color(0xFFFDD835);
      case 2:
        return const Color(0xFFFFA726);
      case 3:
        return const Color(0xFFEF5350);
      default:
        return const Color(0xFF78909C);
    }
  }
}

class _ThreatBadge extends StatelessWidget {
  final int level;
  const _ThreatBadge({required this.level});

  @override
  Widget build(BuildContext context) {
    final label = switch (level) {
      1 => 'LOW',
      2 => 'MED',
      3 => 'HIGH',
      _ => '?',
    };
    final color = switch (level) {
      1 => const Color(0xFFFDD835),
      2 => const Color(0xFFFFA726),
      3 => const Color(0xFFEF5350),
      _ => Colors.grey,
    };
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
      decoration: BoxDecoration(
        color: color.withOpacity(0.15),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: color.withOpacity(0.5)),
      ),
      child: Text(
        label,
        style: TextStyle(
          color: color,
          fontSize: 10,
          fontWeight: FontWeight.w800,
        ),
      ),
    );
  }
}
