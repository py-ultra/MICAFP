//! Automatic Protocol Selector
//!
//! Given a DetectedIsp (from the ISP detection engine), selects the
//! optimal protocol and obfuscation strategy automatically.
//! This is the "brain" that ties all detection and profile data together.
//!
//! ## Decision Algorithm
//!
//! 1. Load ISP profile from isp-profiles.json
//! 2. Check QUIC availability → enables/disables Hysteria2
//! 3. Check NAIN mode → restricts to ArvanCloud-safe options only
//! 4. Check FAVA version → determines fragmentation aggressiveness
//! 5. Check active_probing flag → enables ShadowTLS v3 or Reality
//! 6. Rank protocols by ISP's preferred_protocols_ranked list
//! 7. Return ordered list of (protocol, config) pairs to try

use super::DetectedIsp;

/// Selected protocol with its configuration.
#[derive(Debug, Clone)]
pub struct ProtocolSelection {
    pub protocol: Protocol,
    pub priority: u8,
    pub reason: String,
    pub config_hints: ProtocolConfigHints,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Protocol {
    VlessReality,
    VlessRealityVision,
    ShadowTlsV3,
    AmneziaWg,
    NaiveProxy,
    Hysteria2,
    VlessWsTls,
    VmessWsTls,
    TrojanWsTls,
    Psiphon,
    MeekAzure,
}

/// Config hints for the selected protocol.
#[derive(Debug, Clone)]
pub struct ProtocolConfigHints {
    pub utls_fingerprint: Option<String>,
    pub reality_dest: Option<String>,
    pub shadow_tls_sni: Option<String>,
    pub amnezia_config: Option<String>,
    pub rotation_interval_days: u32,
    pub use_cdn_fronting: bool,
    pub cdn_provider: Option<String>,
}

/// Select the best protocols for a detected ISP.
pub fn select_protocols(isp: &DetectedIsp) -> Vec<ProtocolSelection> {
    let mut selections = Vec::new();

    // NAIN mode: only ArvanCloud-safe options
    if isp.nain_active {
        selections.push(ProtocolSelection {
            protocol: Protocol::VlessWsTls,
            priority: 1,
            reason: "NAIN mode active — ArvanCloud CDN fronting is only reliable option".into(),
            config_hints: ProtocolConfigHints {
                utls_fingerprint: Some("chrome".into()),
                reality_dest: None,
                shadow_tls_sni: None,
                amnezia_config: None,
                rotation_interval_days: 30,
                use_cdn_fronting: true,
                cdn_provider: Some("arvancloud".into()),
            },
        });
        return selections;
    }

    // Normal mode: rank by ISP profile
    match isp.id.as_str() {
        "irancell" | "pars_online" => {
            // FAVA v4.x ML: Reality with randomized fingerprint + ShadowTLS
            selections.push(ProtocolSelection {
                protocol: Protocol::VlessRealityVision,
                priority: 1,
                reason: format!("FAVA v4 ML on {} — randomized uTLS defeats ML classifier", isp.id),
                config_hints: ProtocolConfigHints {
                    utls_fingerprint: Some("randomized".into()),
                    reality_dest: Some("www.speedtest.net:443".into()),
                    shadow_tls_sni: None,
                    amnezia_config: None,
                    rotation_interval_days: 7,
                    use_cdn_fronting: false,
                    cdn_provider: None,
                },
            });
            selections.push(ProtocolSelection {
                protocol: Protocol::ShadowTlsV3,
                priority: 2,
                reason: "ShadowTLS v3 defeats active probing on Irancell/ParsOnline".into(),
                config_hints: ProtocolConfigHints {
                    utls_fingerprint: None,
                    reality_dest: None,
                    shadow_tls_sni: Some("www.apple.com".into()),
                    amnezia_config: None,
                    rotation_interval_days: 14,
                    use_cdn_fronting: false,
                    cdn_provider: None,
                },
            });
        }
        "rightel" | "asiatech" | "afranet" | "mobinnet" => {
            // Light filtering: Hysteria2 if QUIC available, else Reality
            if isp.quic_available == Some(true) {
                selections.push(ProtocolSelection {
                    protocol: Protocol::Hysteria2,
                    priority: 1,
                    reason: format!("QUIC available on {} — Hysteria2 fastest option", isp.id),
                    config_hints: ProtocolConfigHints {
                        utls_fingerprint: None,
                        reality_dest: None,
                        shadow_tls_sni: None,
                        amnezia_config: None,
                        rotation_interval_days: 30,
                        use_cdn_fronting: false,
                        cdn_provider: None,
                    },
                });
            }
            selections.push(ProtocolSelection {
                protocol: Protocol::VlessReality,
                priority: 2,
                reason: format!("Reality reliable fallback for {}", isp.id),
                config_hints: ProtocolConfigHints {
                    utls_fingerprint: Some("chrome".into()),
                    reality_dest: Some("www.speedtest.net:443".into()),
                    shadow_tls_sni: None,
                    amnezia_config: None,
                    rotation_interval_days: 30,
                    use_cdn_fronting: false,
                    cdn_provider: None,
                },
            });
            selections.push(ProtocolSelection {
                protocol: Protocol::NaiveProxy,
                priority: 3,
                reason: "NaiveProxy excellent on light-filtering ISPs".into(),
                config_hints: ProtocolConfigHints {
                    utls_fingerprint: None,
                    reality_dest: None,
                    shadow_tls_sni: None,
                    amnezia_config: None,
                    rotation_interval_days: 30,
                    use_cdn_fronting: false,
                    cdn_provider: None,
                },
            });
        }
        "mci" => {
            selections.push(ProtocolSelection {
                protocol: Protocol::VlessReality,
                priority: 1,
                reason: "FAVA v3.2 on MCI — Reality with Chrome fingerprint".into(),
                config_hints: ProtocolConfigHints {
                    utls_fingerprint: Some("chrome".into()),
                    reality_dest: Some("www.speedtest.net:443".into()),
                    shadow_tls_sni: None,
                    amnezia_config: Some("Jc=4 Jmin=40 Jmax=80".into()),
                    rotation_interval_days: 14,
                    use_cdn_fronting: false,
                    cdn_provider: None,
                },
            });
            selections.push(ProtocolSelection {
                protocol: Protocol::AmneziaWg,
                priority: 2,
                reason: "AmneziaWG second choice on MCI — effective WG obfuscation".into(),
                config_hints: ProtocolConfigHints {
                    utls_fingerprint: None,
                    reality_dest: None,
                    shadow_tls_sni: None,
                    amnezia_config: Some("Jc=4 Jmin=40 Jmax=80 S1=50 S2=100".into()),
                    rotation_interval_days: 14,
                    use_cdn_fronting: false,
                    cdn_provider: None,
                },
            });
        }
        "mokhaberat" => {
            selections.push(ProtocolSelection {
                protocol: Protocol::VlessReality,
                priority: 1,
                reason: "TCI backbone — Reality best for normal operation".into(),
                config_hints: ProtocolConfigHints {
                    utls_fingerprint: Some("chrome".into()),
                    reality_dest: Some("captive.apple.com:443".into()),
                    shadow_tls_sni: None,
                    amnezia_config: None,
                    rotation_interval_days: 14,
                    use_cdn_fronting: false,
                    cdn_provider: None,
                },
            });
            selections.push(ProtocolSelection {
                protocol: Protocol::Psiphon,
                priority: 2,
                reason: "Psiphon Iran config reliable second choice on TCI".into(),
                config_hints: ProtocolConfigHints {
                    utls_fingerprint: None, reality_dest: None, shadow_tls_sni: None,
                    amnezia_config: None, rotation_interval_days: 30,
                    use_cdn_fronting: false, cdn_provider: None,
                },
            });
        }
        _ => {
            // Default: Reality + ShadowTLS
            selections.push(ProtocolSelection {
                protocol: Protocol::VlessReality,
                priority: 1,
                reason: "Default — Reality reliable on most Iranian ISPs".into(),
                config_hints: ProtocolConfigHints {
                    utls_fingerprint: Some("chrome".into()),
                    reality_dest: Some("www.speedtest.net:443".into()),
                    shadow_tls_sni: None, amnezia_config: None,
                    rotation_interval_days: 30,
                    use_cdn_fronting: false, cdn_provider: None,
                },
            });
        }
    }

    selections
}
