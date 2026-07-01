use std::collections::HashMap;
use rand::Rng;

/// Core selector arm for UCB1 bandit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreArm {
    Hiddify,
    Xray,
    Singbox,
    AmneziaVpn,
    DefyX,
    Moav,
    Lantern,
    Mahsang,
    Psiphon,
}

impl CoreArm {
    pub fn all() -> &'static [CoreArm] {
        &[
            CoreArm::Hiddify,
            CoreArm::Xray,
            CoreArm::Singbox,
            CoreArm::AmneziaVpn,
            CoreArm::DefyX,
            CoreArm::Moav,
            CoreArm::Lantern,
            CoreArm::Mahsang,
            CoreArm::Psiphon,
        ]
    }
}

impl std::fmt::Display for CoreArm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreArm::Hiddify => write!(f, "hiddify"),
            CoreArm::Xray => write!(f, "xray"),
            CoreArm::Singbox => write!(f, "singbox"),
            CoreArm::AmneziaVpn => write!(f, "amneziavpn"),
            CoreArm::DefyX => write!(f, "defyx"),
            CoreArm::Moav => write!(f, "moav"),
            CoreArm::Lantern => write!(f, "lantern"),
            CoreArm::Mahsang => write!(f, "mahsang"),
            CoreArm::Psiphon => write!(f, "psiphon"),
        }
    }
}

pub struct ArmStats {
    pub pulls: u64,
    pub reward_sum: f64,
    pub recent_rewards: Vec<f64>,
    pub decay_factor: f64,
}

pub struct UCBBandit {
    arms: HashMap<String, ArmStats>,
    total_pulls: u64,
    alpha: f64,
}

impl UCBBandit {
    pub fn new(alpha: f64) -> Self {
        Self { arms: HashMap::new(), total_pulls: 0, alpha }
    }

    pub fn add_arm(&mut self, id: &str) {
        self.arms.insert(id.to_string(), ArmStats {
            pulls: 0, reward_sum: 0.0, recent_rewards: Vec::new(), decay_factor: 0.95,
        });
    }

    pub fn select(&mut self) -> SelectionResult {
        let mut best = None;
        let mut best_score = f64::NEG_INFINITY;
        for (id, stats) in &self.arms {
            let score = if stats.pulls == 0 {
                f64::INFINITY
            } else {
                let avg = stats.reward_sum / stats.pulls as f64;
                let exploration = self.alpha * ((2.0 * (self.total_pulls as f64).ln() / stats.pulls as f64).sqrt());
                avg + exploration
            };
            if score > best_score {
                best_score = score;
                best = Some(id.clone());
            }
        }
        SelectionResult {
            arm: parse_arm(&best),
            score: best_score,
        }
    }

    pub fn select_arm(&mut self) -> Option<String> {
        let mut best = None;
        let mut best_score = f64::NEG_INFINITY;
        for (id, stats) in &self.arms {
            let score = if stats.pulls == 0 {
                f64::INFINITY
            } else {
                let avg = stats.reward_sum / stats.pulls as f64;
                let exploration = self.alpha * ((2.0 * (self.total_pulls as f64).ln() / stats.pulls as f64).sqrt());
                avg + exploration
            };
            if score > best_score {
                best_score = score;
                best = Some(id.clone());
            }
        }
        best
    }

    pub fn update_reward(&mut self, arm_id: &str, reward: f64) {
        if let Some(stats) = self.arms.get_mut(arm_id) {
            stats.reward_sum = stats.reward_sum * stats.decay_factor + reward;
            stats.pulls += 1;
            stats.recent_rewards.push(reward);
            if stats.recent_rewards.len() > 100 { stats.recent_rewards.remove(0); }
        }
        self.total_pulls += 1;
    }

    pub fn get_scores(&self) -> Vec<(String, f64)> {
        self.arms.iter().map(|(id, stats)| {
            let score = if stats.pulls == 0 { 0.0 } else {
                stats.reward_sum / stats.pulls as f64 + self.alpha * ((2.0 * (self.total_pulls as f64).ln() / stats.pulls as f64).sqrt())
            };
            (id.clone(), score)
        }).collect()
    }
}

pub struct SelectionResult {
    pub arm: CoreArm,
    pub score: f64,
}

fn parse_arm(arm_str: &Option<String>) -> CoreArm {
    match arm_str.as_deref() {
        Some("hiddify") => CoreArm::Hiddify,
        Some("xray") => CoreArm::Xray,
        Some("singbox") => CoreArm::Singbox,
        Some("amneziavpn") => CoreArm::AmneziaVpn,
        Some("defyx") => CoreArm::DefyX,
        Some("moav") => CoreArm::Moav,
        Some("lantern") => CoreArm::Lantern,
        Some("mahsang") => CoreArm::Mahsang,
        Some("psiphon") => CoreArm::Psiphon,
        _ => CoreArm::Xray, // Default fallback
    }
}
