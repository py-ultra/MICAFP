// ─────────────────────────────────────────────────────────────────────────────
// MICAFP-UnifiedShield v10.0 — BLoC Providers Registry
//
// All BLoCs are registered here. Never scatter BlocProvider calls across
// widget trees — add new BLoCs to this single registry.
// ─────────────────────────────────────────────────────────────────────────────

import 'package:flutter_bloc/flutter_bloc.dart';
import 'dashboard_bloc.dart';

/// Returns all BLoC providers for the application.
/// lazy: false on DashboardBloc ensures Rust events are never missed —
/// the BLoC subscribes to the stream before any widget builds.
List<BlocProvider> buildBlocProviders() => [
  BlocProvider<DashboardBloc>(
    create: (_) => DashboardBloc()..add(DashboardStarted()),
    lazy: false,
  ),
];
