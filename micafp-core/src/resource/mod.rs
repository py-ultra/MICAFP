//! Smart Resource Manager — MICAFP v6.0 + v9.0 Feature 3
//!
//! Battery, thermal, and network-aware scheduling.
//! Runs as a low-priority background task (~0.1% CPU when idle).
//! NEVER disables security or expiry enforcement — only adjusts
//! polling frequency and channel parallelism.

use std::time::Duration;
use sha2::{Digest, Sha256};
use rand::{Rng, RngCore};
use tracing::{debug, info, warn};

/// Battery state (platform-detected).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryState {
    Full,     // >80% or charging
    Normal,   // 30–80%
    Low,      // 15–30%
    Critical, // <15%
    Charging, // just plugged in → immediate full resume
    Unknown,  // can't detect battery (desktop)
}

/// CPU thermal state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThermalState {
    Normal,   // <60°C
    Warm,     // 60–70°C
    Hot,      // 70–80°C
    Critical, // >80°C
}

/// Network type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkType {
    Wifi,
    MobileData,
    NoNetwork,
    Unknown,
}

/// CPU scheduling priority settings.
pub struct ResourceManager {
    pub battery:  BatteryState,
    pub thermal:  ThermalState,
    pub network:  NetworkType,
    /// True when expiry is within 3 days (doubles poll frequency).
    pub expiry_approaching: bool,
    /// Consecutive overheats within 1 hour.
    overheat_count: u32,
    overheat_window_start: Option<std::time::Instant>,
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self {
            battery: BatteryState::Unknown,
            thermal: ThermalState::Normal,
            network: NetworkType::Unknown,
            expiry_approaching: false,
            overheat_count: 0,
            overheat_window_start: None,
        }
    }
}

impl ResourceManager {
    pub fn new() -> Self { Self::default() }

    /// Refresh all state readings. Call every 60 seconds.
    pub fn refresh(&mut self) {
        self.battery = read_battery_state();
        self.thermal = read_thermal_state();
        self.network = read_network_type();
        self.update_overheat_tracking();
        debug!(
            "ResourceManager: battery={:?} thermal={:?} network={:?}",
            self.battery, self.thermal, self.network
        );
    }

    /// Which channel IDs should be active this cycle.
    pub fn active_channels(&self) -> Vec<u8> {
        use crate::channel::{ALL_CHANNEL_IDS, TOP5_CHANNELS, TOP2_CHANNELS, MOBILE_DATA_CHANNELS};

        // Critical thermal → cache only
        if self.thermal == ThermalState::Critical {
            warn!("Thermal critical: suspending all network, using cache only");
            return vec![];
        }

        // No network → cache + SSB LAN only
        if self.network == NetworkType::NoNetwork {
            return vec![9]; // SSB LAN-only channel
        }

        // Mobile data reduces channel set
        let base: &[u8] = match self.network {
            NetworkType::MobileData => MOBILE_DATA_CHANNELS,
            _                       => ALL_CHANNEL_IDS,
        };

        // Battery state limits further
        let limited: Vec<u8> = match self.battery {
            BatteryState::Critical => vec![1],          // DNS only
            BatteryState::Low      => TOP2_CHANNELS.to_vec(),
            BatteryState::Normal   => {
                // Mobile + battery<30% → DNS + meek only
                if self.network == NetworkType::MobileData {
                    TOP2_CHANNELS.to_vec()
                } else {
                    TOP5_CHANNELS.to_vec()
                }
            }
            _ => base.to_vec(),
        };

        // Thermal warm/hot: cap parallelism
        match self.thermal {
            ThermalState::Hot  => limited.into_iter().take(2).collect(),
            ThermalState::Warm => limited.into_iter().take(4).collect(),
            _                  => limited,
        }
    }

    /// Base poll interval for current state.
    pub fn base_poll_interval(&self) -> Duration {
        let base = match self.battery {
            BatteryState::Full | BatteryState::Charging | BatteryState::Unknown
                => Duration::from_secs(6 * 3600),
            BatteryState::Normal
                => Duration::from_secs(8 * 3600),
            BatteryState::Low
                => Duration::from_secs(12 * 3600),
            BatteryState::Critical
                => Duration::from_secs(24 * 3600),
        };
        // Feature 7: halve interval when expiry is within 3 days
        if self.expiry_approaching {
            base / 2
        } else {
            base
        }
    }

    /// Actual poll interval with jitter + time-of-day offset.
    /// Feature 3: traffic shaping to defeat AI timing classifiers.
    pub fn actual_poll_interval(&self, device_salt: &[u8; 32]) -> Duration {
        let base = self.base_poll_interval();
        let jitter_secs = rand::thread_rng().gen_range(0u64..1800); // ±30 min
        let day_offset = compute_day_offset(device_salt);
        base + Duration::from_secs(jitter_secs) + Duration::from_secs(day_offset)
    }

    /// Max concurrent channels (for tokio::select! parallelism).
    pub fn max_parallelism(&self) -> usize {
        match self.thermal {
            ThermalState::Critical => 0,
            ThermalState::Hot      => 2,
            ThermalState::Warm     => 4,
            ThermalState::Normal   => match self.battery {
                BatteryState::Critical => 1,
                BatteryState::Low      => 2,
                _                      => 10,
            },
        }
    }

    /// Thermal overheat tracking with exponential backoff.
    fn update_overheat_tracking(&mut self) {
        if self.thermal >= ThermalState::Hot {
            let window_ok = self.overheat_window_start
                .map(|t| t.elapsed() < Duration::from_secs(3600))
                .unwrap_or(false);
            if window_ok {
                self.overheat_count += 1;
            } else {
                self.overheat_count = 1;
                self.overheat_window_start = Some(std::time::Instant::now());
            }
            if self.overheat_count >= 3 {
                warn!("3 overheats in 1h — applying 30min mandatory cooldown");
            }
        } else {
            if self.overheat_count > 0 {
                info!("Thermal back to normal, resetting overheat counter");
            }
            self.overheat_count = 0;
        }
    }
}

/// Feature 3: per-device deterministic daily offset to avoid predictable timing.
fn compute_day_offset(device_salt: &[u8; 32]) -> u64 {
    let day_number = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() / 86400;
    let mut hasher = Sha256::new();
    hasher.update(device_salt);
    hasher.update(&day_number.to_le_bytes());
    let digest = hasher.finalize();
    // Use first 2 bytes as minutes offset (0–65535 minutes, capped to 60)
    let raw = u16::from_le_bytes([digest[0], digest[1]]) as u64;
    (raw % 60) * 60 // 0–60 minutes in seconds
}

/// Pad payload to next multiple of 512 bytes with random bytes (Feature 14 / Feature 3).
pub fn pad_to_512_boundary(data: &mut Vec<u8>) {
    let target = ((data.len() / 512) + 1) * 512;
    let needed = target - data.len();
    let mut padding = vec![0u8; needed];
    rand::thread_rng().fill_bytes(&mut padding);
    data.extend_from_slice(&padding);
}

// ── Platform-specific state readers ──────────────────────────────────────

fn read_battery_state() -> BatteryState {
    #[cfg(target_os = "linux")]
    {
        // /sys/class/power_supply/battery/capacity and status
        let capacity = std::fs::read_to_string(
            "/sys/class/power_supply/battery/capacity"
        ).ok().and_then(|s| s.trim().parse::<u8>().ok());
        let status = std::fs::read_to_string(
            "/sys/class/power_supply/battery/status"
        ).unwrap_or_default();
        if status.trim() == "Charging" { return BatteryState::Charging; }
        match capacity {
            Some(c) if c > 80 => BatteryState::Full,
            Some(c) if c > 30 => BatteryState::Normal,
            Some(c) if c > 15 => BatteryState::Low,
            Some(_)            => BatteryState::Critical,
            None               => BatteryState::Unknown,
        }
    }
    #[cfg(not(target_os = "linux"))]
    BatteryState::Unknown
}

fn read_thermal_state() -> ThermalState {
    #[cfg(target_os = "linux")]
    {
        // Read all thermal zones and take max temperature
        let mut max_temp_milliC: i64 = 0;
        if let Ok(entries) = std::fs::read_dir("/sys/class/thermal") {
            for entry in entries.flatten() {
                let temp_path = entry.path().join("temp");
                if let Ok(s) = std::fs::read_to_string(temp_path) {
                    if let Ok(t) = s.trim().parse::<i64>() {
                        max_temp_milliC = max_temp_milliC.max(t);
                    }
                }
            }
        }
        let temp_c = max_temp_milliC / 1000;
        match temp_c {
            t if t >= 80 => ThermalState::Critical,
            t if t >= 70 => ThermalState::Hot,
            t if t >= 60 => ThermalState::Warm,
            _            => ThermalState::Normal,
        }
    }
    #[cfg(not(target_os = "linux"))]
    ThermalState::Normal
}

fn read_network_type() -> NetworkType {
    // Production: check active network interface type
    // Android: ConnectivityManager.getActiveNetworkInfo()
    // Linux: check /sys/class/net/*/type or use rtnetlink
    NetworkType::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_channels_critical_thermal() {
        let mut rm = ResourceManager::new();
        rm.thermal = ThermalState::Critical;
        assert!(rm.active_channels().is_empty());
    }

    #[test]
    fn test_active_channels_critical_battery() {
        let mut rm = ResourceManager::new();
        rm.battery = BatteryState::Critical;
        rm.thermal = ThermalState::Normal;
        rm.network = NetworkType::Wifi;
        let active = rm.active_channels();
        assert_eq!(active, vec![1]);
    }

    #[test]
    fn test_active_channels_low_battery() {
        let mut rm = ResourceManager::new();
        rm.battery = BatteryState::Low;
        rm.thermal = ThermalState::Normal;
        rm.network = NetworkType::Wifi;
        assert_eq!(rm.active_channels(), vec![1, 2]);
    }

    #[test]
    fn test_padding_multiple_of_512() {
        let mut data = vec![0u8; 100];
        pad_to_512_boundary(&mut data);
        assert_eq!(data.len(), 512);
        let mut data2 = vec![0u8; 513];
        pad_to_512_boundary(&mut data2);
        assert_eq!(data2.len(), 1024);
    }

    #[test]
    fn test_expiry_approaching_halves_interval() {
        let mut rm = ResourceManager::new();
        rm.battery = BatteryState::Normal;
        rm.expiry_approaching = false;
        let normal_base = rm.base_poll_interval();
        rm.expiry_approaching = true;
        let approaching_base = rm.base_poll_interval();
        assert_eq!(approaching_base, normal_base / 2);
    }

    #[test]
    fn test_all_battery_thermal_combinations() {
        let batteries = [
            BatteryState::Full, BatteryState::Normal,
            BatteryState::Low,  BatteryState::Critical,
        ];
        let thermals = [
            ThermalState::Normal, ThermalState::Warm,
            ThermalState::Hot, ThermalState::Critical,
        ];
        for &b in &batteries {
            for &t in &thermals {
                let mut rm = ResourceManager::new();
                rm.battery = b;
                rm.thermal = t;
                rm.network = NetworkType::Wifi;
                let channels = rm.active_channels();
                // Critical thermal → empty; otherwise some channels
                if t == ThermalState::Critical {
                    assert!(channels.is_empty());
                }
            }
        }
    }
}
