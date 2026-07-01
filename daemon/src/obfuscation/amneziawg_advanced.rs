//! AmneziaWG Advanced — Obfuscated WireGuard for Iranian ISPs
//!
//! AmneziaWG modifies the WireGuard protocol to defeat signature-based
//! detection by randomising the handshake initiation packet in ways that
//! make it unrecognisable to DPI systems that look for WireGuard's
//! characteristic 4-byte message type field (0x00000001 for Initiation).
//!
//! ## Standard WireGuard Detection Signature
//!
//! WireGuard Initiation message (148 bytes):
//!   Byte 0-3:  Message type = 0x00000001  ← DPI triggers here
//!   Byte 4-7:  Sender index
//!   Byte 8-39: Unencrypted ephemeral key
//!   ...
//!
//! FAVA v2+ and Irancell DPI both detect this within the first 4 bytes.
//!
//! ## AmneziaWG Obfuscation Parameters
//!
//! AmneziaWG inserts configurable "junk" packets before and after the
//! real WireGuard handshake and randomises internal packet fields:
//!
//! | Parameter | Meaning                                      | Recommended value |
//! |-----------|----------------------------------------------|-------------------|
//! | Jc        | Number of junk packets before handshake      | 3-10              |
//! | Jmin      | Minimum size of each junk packet (bytes)     | 40                |
//! | Jmax      | Maximum size of each junk packet (bytes)     | 70                |
//! | S1        | Extra bytes appended to Initiation message   | 50                |
//! | S2        | Extra bytes appended to Response message     | 100               |
//! | H1-H4     | Random XOR masks for message type fields     | random u32        |
//!
//! ## ISP-Specific Tuning
//!
//! | ISP           | Recommended config              | Notes                          |
//! |---------------|---------------------------------|--------------------------------|
//! | MCI           | Jc=4 Jmin=40 Jmax=80 S1=50 S2=100 | FAVA v3.2 needs S1>30        |
//! | Irancell      | Jc=6 Jmin=50 Jmax=100 S1=80 S2=150| FAVA v4.0 ML; max junk       |
//! | ParsOnline    | Jc=8 Jmin=60 Jmax=120 S1=100 S2=200| FAVA v4.1; most aggressive  |
//! | Shatel        | Jc=4 Jmin=40 Jmax=70 S1=50 S2=100 | Fixed-line; moderate config |
//! | Mokhaberat    | Jc=3 Jmin=40 Jmax=70 S1=50 S2=100 | FAVA v2.5; standard config  |
//! | Rightel       | Jc=2 Jmin=40 Jmax=60 S1=30 S2=60  | FAVA v2.1; light config     |

use rand::Rng;
use serde::{Deserialize, Serialize};

/// AmneziaWG obfuscation parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmneziawgConfig {
    /// Number of junk packets before handshake.
    pub jc: u8,
    /// Minimum junk packet size.
    pub jmin: u16,
    /// Maximum junk packet size.
    pub jmax: u16,
    /// Extra bytes appended to Initiation message.
    pub s1: u16,
    /// Extra bytes appended to Response message.
    pub s2: u16,
    /// XOR mask for Initiation message type field.
    pub h1: u32,
    /// XOR mask for Response message type field.
    pub h2: u32,
    /// XOR mask for Cookie Reply message type field.
    pub h3: u32,
    /// XOR mask for Transport message type field.
    pub h4: u32,
}

impl AmneziawgConfig {
    /// Generate a new config with cryptographically random H1-H4 values.
    pub fn generate_random(jc: u8, jmin: u16, jmax: u16, s1: u16, s2: u16) -> Self {
        let mut rng = rand::thread_rng();
        // H values must be non-zero and must not produce the original WireGuard
        // message type values when XOR'd back:
        //   Original: 1=Init, 2=Response, 3=CookieReply, 4=Transport
        // So H1 XOR 1 must not equal 1 (H1 != 0, which our random guarantees)
        let h1 = rng.gen::<u32>() | 0x80000000; // Ensure high bit set (clearly non-WG)
        let h2 = rng.gen::<u32>() | 0x80000000;
        let h3 = rng.gen::<u32>() | 0x80000000;
        let h4 = rng.gen::<u32>() | 0x80000000;
        Self { jc, jmin, jmax, s1, s2, h1, h2, h3, h4 }
    }

    /// Generate the recommended config for a specific Iranian ISP.
    pub fn for_isp(isp_id: &str) -> Self {
        match isp_id {
            "irancell" => Self::generate_random(6, 50, 100, 80, 150),
            "pars_online" => Self::generate_random(8, 60, 120, 100, 200),
            "mci" => Self::generate_random(4, 40, 80, 50, 100),
            "shatel" => Self::generate_random(4, 40, 70, 50, 100),
            "mokhaberat" => Self::generate_random(3, 40, 70, 50, 100),
            "rightel" => Self::generate_random(2, 40, 60, 30, 60),
            _ => Self::generate_random(4, 40, 70, 50, 100),
        }
    }

    /// Render as wg-quick / AmneziaVPN compatible config block.
    pub fn to_amnezia_config_block(&self) -> String {
        format!(
            "# AmneziaWG obfuscation parameters\nJc = {}\nJmin = {}\nJmax = {}\n\
             S1 = {}\nS2 = {}\nH1 = {}\nH2 = {}\nH3 = {}\nH4 = {}",
            self.jc, self.jmin, self.jmax,
            self.s1, self.s2,
            self.h1, self.h2, self.h3, self.h4
        )
    }

    /// Render as sing-box compatible JSON fragment.
    pub fn to_singbox_json(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "amnezia",
            "jc": self.jc,
            "jmin": self.jmin,
            "jmax": self.jmax,
            "s1": self.s1,
            "s2": self.s2,
            "h1": self.h1,
            "h2": self.h2,
            "h3": self.h3,
            "h4": self.h4
        })
    }
}
