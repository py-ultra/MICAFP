// ─────────────────────────────────────────────────────────────────────────────
// MICAFP-UnifiedShield v10.0 — Failover History Widget
//
// Displays the chronological list of automatic transport failovers executed
// by the Rust AI engine. Purely informational — the failovers have already
// occurred by the time this widget renders.
// ─────────────────────────────────────────────────────────────────────────────

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../bloc/dashboard_bloc.dart';

class FailoverHistoryWidget extends StatelessWidget {
  const FailoverHistoryWidget({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<DashboardBloc, DashboardState>(
      buildWhen: (prev, curr) =>
          prev.recentFailovers != curr.recentFailovers,
      builder: (context, state) {
        if (state.recentFailovers.isEmpty) {
          return Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Row(
                children: [
                  const Icon(Icons.check_circle_outline,
                      color: Color(0xFF2E7D32), size: 18),
                  const SizedBox(width: 8),
                  Text(
                    'No failovers — AI engine running stable',
                    style: Theme.of(context).textTheme.bodySmall?.copyWith(
                          color: const Color(0xFF81C784),
                        ),
                  ),
                ],
              ),
            ),
          );
        }

        return Card(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    const Icon(Icons.history, color: Color(0xFF5C6BC0), size: 18),
                    const SizedBox(width: 8),
                    Text(
                      'Auto-Failover Log',
                      style: Theme.of(context).textTheme.titleSmall?.copyWith(
                            fontWeight: FontWeight.w700,
                          ),
                    ),
                    const Spacer(),
                    Text(
                      '${state.recentFailovers.length} events',
                      style: Theme.of(context).textTheme.bodySmall?.copyWith(
                            color: Colors.grey,
                          ),
                    ),
                  ],
                ),
                const SizedBox(height: 12),
                ...state.recentFailovers.take(5).map(
                      (r) => _FailoverRow(record: r),
                    ),
                if (state.recentFailovers.length > 5)
                  Padding(
                    padding: const EdgeInsets.only(top: 8),
                    child: Text(
                      '+${state.recentFailovers.length - 5} older events',
                      style: const TextStyle(color: Colors.grey, fontSize: 12),
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

class _FailoverRow extends StatelessWidget {
  final TransportChangeRecord record;
  const _FailoverRow({required this.record});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          // Left: from → to
          Expanded(
            child: Row(
              children: [
                _TransportLabel(record.from, color: const Color(0xFFEF9A9A)),
                const Padding(
                  padding: EdgeInsets.symmetric(horizontal: 4),
                  child: Icon(Icons.arrow_forward,
                      size: 14, color: Color(0xFF78909C)),
                ),
                _TransportLabel(record.to, color: const Color(0xFF81C784)),
              ],
            ),
          ),
          // Right: latency badge
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
            decoration: BoxDecoration(
              color: _latencyColor(record.failoverLatencyMs).withOpacity(0.12),
              borderRadius: BorderRadius.circular(6),
            ),
            child: Text(
              '${record.failoverLatencyMs}ms',
              style: TextStyle(
                color: _latencyColor(record.failoverLatencyMs),
                fontSize: 10,
                fontWeight: FontWeight.w700,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Color _latencyColor(int ms) {
    if (ms < 100) return const Color(0xFF66BB6A);
    if (ms < 200) return const Color(0xFFFFA726);
    return const Color(0xFFEF5350);
  }
}

class _TransportLabel extends StatelessWidget {
  final String name;
  final Color color;
  const _TransportLabel(this.name, {required this.color});

  @override
  Widget build(BuildContext context) {
    return Flexible(
      child: Text(
        name,
        overflow: TextOverflow.ellipsis,
        style: TextStyle(
          color: color,
          fontSize: 11,
          fontWeight: FontWeight.w600,
        ),
      ),
    );
  }
}
