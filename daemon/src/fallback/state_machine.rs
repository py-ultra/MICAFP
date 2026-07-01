//! Fallback State Machine
//!
//! Tracks the lifecycle of the FallbackEngine with explicit states
//! and valid transitions. Used for telemetry, UI display, and
//! ensuring the engine never enters an inconsistent state.
//!
//! ## State Diagram
//!
//! ```text
//!  [Initializing]
//!       │ first connect ok
//!       ▼
//!  [Connected] ◄──────────────────────────────────┐
//!       │ block signal(s)                          │ fallback ok
//!       ▼                                          │
//!  [FallingBack] ──────────────────────────────────┘
//!       │ all protocols tried, none worked
//!       ▼
//!  [AllProtocolsFailed]
//!       │ user retries / ISP unblocks
//!       ▼
//!  [Reconnecting] ──► [Connected]
//! ```

use std::time::Instant;
use tracing::{info, warn, error};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackState {
    /// Engine created, not yet connected.
    Initializing,
    /// Active tunnel healthy, traffic flowing.
    Connected,
    /// Block detected, trying next protocol.
    FallingBack,
    /// All protocols tried, none working.
    AllProtocolsFailed,
    /// Attempting to reconnect after failure.
    Reconnecting,
}

impl std::fmt::Display for FallbackState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initializing        => write!(f, "Initializing"),
            Self::Connected           => write!(f, "Connected"),
            Self::FallingBack         => write!(f, "FallingBack"),
            Self::AllProtocolsFailed  => write!(f, "AllProtocolsFailed"),
            Self::Reconnecting        => write!(f, "Reconnecting"),
        }
    }
}

pub struct FallbackStateMachine {
    state: FallbackState,
    entered_at: Instant,
    transition_count: u32,
    fallback_count: u32,
}

impl FallbackStateMachine {
    pub fn new() -> Self {
        Self {
            state: FallbackState::Initializing,
            entered_at: Instant::now(),
            transition_count: 0,
            fallback_count: 0,
        }
    }

    pub fn current(&self) -> FallbackState { self.state }
    pub fn fallback_count(&self) -> u32 { self.fallback_count }
    pub fn time_in_state(&self) -> std::time::Duration { self.entered_at.elapsed() }

    /// Transition to a new state, validating the transition is legal.
    pub fn transition_to(&mut self, new_state: FallbackState) {
        if !self.is_valid_transition(new_state) {
            warn!(
                "StateMachine: invalid transition {} → {} (ignored)",
                self.state, new_state
            );
            return;
        }

        let old = self.state;
        self.state = new_state;
        self.entered_at = Instant::now();
        self.transition_count += 1;

        if new_state == FallbackState::FallingBack {
            self.fallback_count += 1;
        }

        match new_state {
            FallbackState::Connected => {
                info!("StateMachine: {} → Connected (fallbacks so far: {})",
                      old, self.fallback_count);
            }
            FallbackState::FallingBack => {
                warn!("StateMachine: {} → FallingBack (attempt #{})",
                      old, self.fallback_count);
            }
            FallbackState::AllProtocolsFailed => {
                error!("StateMachine: ALL PROTOCOLS FAILED after {} fallback attempts",
                       self.fallback_count);
            }
            _ => {
                info!("StateMachine: {} → {}", old, new_state);
            }
        }
    }

    fn is_valid_transition(&self, to: FallbackState) -> bool {
        matches!(
            (self.state, to),
            (FallbackState::Initializing,       FallbackState::Connected)
            | (FallbackState::Connected,        FallbackState::FallingBack)
            | (FallbackState::FallingBack,      FallbackState::Connected)
            | (FallbackState::FallingBack,      FallbackState::AllProtocolsFailed)
            | (FallbackState::AllProtocolsFailed,FallbackState::Reconnecting)
            | (FallbackState::Reconnecting,     FallbackState::Connected)
            | (FallbackState::Reconnecting,     FallbackState::AllProtocolsFailed)
        )
    }
}
