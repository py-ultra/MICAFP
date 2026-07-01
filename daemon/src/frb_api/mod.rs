// ─────────────────────────────────────────────────────────────────────────────
// MICAFP-UnifiedShield v10.0 — flutter_rust_bridge v2 API Surface
//
// TASK-01 implementation. This is the SOLE FFI surface between the Rust daemon
// and the Flutter/Dart layer. All types and functions here are zero-copy via
// FRB v2's automatic code generation.
//
// Rules (enforced via CI invariants):
//   I-01  Rust daemon is the sole arbiter of all network decisions.
//   I-02  Only this file may contain #[no_mangle] / pub extern "C" symbols.
//   I-04  Every Err path must call publish(ShieldEvent::Error{…}).
// ─────────────────────────────────────────────────────────────────────────────

use futures::Stream;

/// Opaque status snapshot sent to the Flutter layer on every tick.
/// Mapped to a Dart @freezed class by the FRB code-generator.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShieldStatusSnapshot {
    /// Whether the daemon currently has an active transport connection.
    pub connected: bool,
    /// Human-readable name of the currently active transport.
    pub active_transport: String,
    /// Round-trip latency of the active transport in milliseconds.
    pub latency_ms: u32,
    /// Cumulative bytes sent since last connect.
    pub bytes_sent: u64,
    /// Cumulative bytes received since last connect.
    pub bytes_recv: u64,
    /// DPI threat level: 0 = none, 1 = low, 2 = medium, 3 = high.
    pub dpi_threat_level: u8,
    /// Number of automatic failovers executed in this session.
    pub failover_count: u32,
    /// Device battery percentage (0–100).
    pub battery_pct: u8,
    /// Composite health score from the load balancer EWMA (0.0–1.0).
    pub health_score: f32,
    /// NAIN (National Intranet) mode active flag.
    pub nain_active: bool,
    /// Name of the currently active VPN core (xray, sing-box, hiddify …).
    pub active_core: String,
    /// ISP name detected by the ISP classifier.
    pub isp_name: String,
    /// Current threat level string from the orchestrator.
    pub threat_level: String,
    /// Uptime seconds of the current daemon session.
    pub uptime_secs: u64,
}

impl Default for ShieldStatusSnapshot {
    fn default() -> Self {
        Self {
            connected: false,
            active_transport: "none".into(),
            latency_ms: 0,
            bytes_sent: 0,
            bytes_recv: 0,
            dpi_threat_level: 0,
            failover_count: 0,
            battery_pct: 100,
            health_score: 1.0,
            nain_active: false,
            active_core: "none".into(),
            isp_name: "unknown".into(),
            threat_level: "Low".into(),
            uptime_secs: 0,
        }
    }
}

/// One-shot commands sent from the Flutter UI to the daemon.
/// Mapped to a Dart sealed class by the FRB code-generator.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ShieldCommand {
    /// Request connection, optionally with a preferred transport name.
    Connect {
        preferred_transport: Option<String>,
        preferred_core: Option<String>,
    },
    /// Request graceful disconnection.
    Disconnect,
    /// Force a specific transport immediately (manual override).
    ForceTransport { name: String },
    /// Force a specific VPN core (xray, sing-box, hiddify, psiphon …).
    ForceCore { name: String },
    /// Emergency wipe — requires auth token to prevent accidental activation.
    EmergencyWipe { auth_token: String },
    /// Rotate identity (PQC key rotation + new circuit).
    RotateIdentity,
    /// Update a configuration key-value pair at runtime.
    ConfigUpdate { key: String, value: String },
}

/// Events pushed from the Rust daemon to the Flutter layer via FRB Stream.
/// Flutter/BLoC only observes these — it never modifies network state.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ShieldEvent {
    /// Periodic status tick (30-second cycle from health_monitor).
    StatusUpdate(ShieldStatusSnapshot),
    /// The AI engine executed an automatic transport failover.
    TransportChanged {
        from: String,
        to: String,
        reason: String,
        failover_latency_ms: u64,
    },
    /// The AI engine switched the active VPN core.
    CoreChanged {
        from: String,
        to: String,
        reason: String,
    },
    /// DPI classifier detected a censorship signature.
    DpiAlert {
        threat_level: u8,
        description: String,
        isp_name: String,
    },
    /// NAIN (National Intranet) mode status changed.
    NainStatusChanged {
        active: bool,
        mode: String,
    },
    /// License / subscription warning from the micafp-core validator.
    LicenseWarning { message: String },
    /// A watchdog subsystem reported a fatal error and was restarted.
    SubsystemRestarted { subsystem: String, reason: String },
    /// ISP detection completed — new ISP profile loaded.
    IspDetected {
        isp_name: String,
        country_code: String,
        censorship_level: u8,
    },
    /// Identity rotation (PQC key + circuit) completed.
    IdentityRotated { new_public_key_hex: String },
    /// Generic daemon error surfaced to the UI.
    Error { code: u32, message: String },
}

// ── Public async API (Dart-callable) ─────────────────────────────────────────

/// Initialise the daemon. Must be called exactly once before any other API.
///
/// `config_json` is a JSON-serialised `ShieldConfig`. Returns `Ok(())` when
/// all subsystems are fully started and the first health tick has fired.
pub async fn shield_init(config_json: String) -> anyhow::Result<()> {
    crate::daemon_start(config_json).await
}

/// Send a `ShieldCommand` to the daemon.
///
/// Fire-and-forget: errors are surfaced asynchronously via the event stream
/// as `ShieldEvent::Error` rather than as returned `Err` values, so the
/// Flutter layer need not await error handling.
pub async fn shield_command(cmd: ShieldCommand) -> anyhow::Result<()> {
    crate::dispatch_frb_command(cmd).await
}

/// Returns a Dart `Stream<ShieldEvent>` backed by the daemon's broadcast
/// channel. FRB v2 maps this Rust `impl Stream` to a Dart Stream automatically.
/// Never returns `None` — stream lives until the daemon shuts down.
pub fn shield_event_stream() -> impl Stream<Item = ShieldEvent> {
    crate::orchestrator::event_stream()
}

/// Synchronous: current status snapshot (for initial UI render, no await).
/// Safe to call from a Dart non-async context.
pub fn shield_status_sync() -> ShieldStatusSnapshot {
    crate::orchestrator::status_snapshot()
}

/// Returns the list of all available transport names known to the daemon.
/// Used by the protocol-switcher widget to populate its dropdown.
pub fn shield_available_transports() -> Vec<String> {
    crate::transport::available_transport_names()
}

/// Returns the list of all available VPN core names.
pub fn shield_available_cores() -> Vec<String> {
    crate::cores::available_core_names()
}

// ── Platform JNI entry points (Android only) ──────────────────────────────────

/// Called from `ShieldVpnService.nativeStartTunnel` via JNI.
/// Receives the TUN file descriptor transferred from the Kotlin VpnService
/// and starts the daemon packet engine. This is the only `#[no_mangle]`
/// symbol in the entire daemon (invariant I-02).
///
/// # Safety
/// `tun_fd` must be a valid open file descriptor. The Kotlin side must call
/// `ParcelFileDescriptor.detachFd()` before passing it here so that Java
/// no longer owns it.
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_micafp_unifiedshield_ShieldVpnService_nativeStartTunnel(
    env: jni::JNIEnv,
    _class: jni::objects::JClass,
    tun_fd: jni::sys::jint,
    config_json: jni::objects::JString,
) -> jni::sys::jboolean {
    let config_str: String = env
        .get_string(&config_json)
        .map(|s| s.into())
        .unwrap_or_default();

    match crate::platform::android_tun::start_tun(tun_fd as i32, &config_str) {
        Ok(_) => jni::sys::JNI_TRUE,
        Err(e) => {
            tracing::error!("nativeStartTunnel failed: {e:#}");
            jni::sys::JNI_FALSE
        }
    }
}

/// Called from `ShieldVpnService.nativeStopTunnel` via JNI.
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_micafp_unifiedshield_ShieldVpnService_nativeStopTunnel(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
) -> jni::sys::jboolean {
    match crate::platform::android_tun::stop_tun() {
        Ok(_) => jni::sys::JNI_TRUE,
        Err(e) => {
            tracing::error!("nativeStopTunnel failed: {e:#}");
            jni::sys::JNI_FALSE
        }
    }
}
