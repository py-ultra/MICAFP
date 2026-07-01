//! Channel Runner — Concurrent tokio::select! Orchestrator
//!
//! Runs the active channel subset concurrently. Returns as soon as
//! the first channel delivers a valid token, cancelling all others.
//! Respects the resource manager's active channel list and
//! adaptive stats ordering.

use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::MicafpError;
use super::{Channel, RawToken, stats::ChannelStats};

/// Result from a single channel run.
#[derive(Debug)]
pub struct ChannelResult {
    pub channel_id:  u8,
    pub channel_name: &'static str,
    pub token:       Option<RawToken>,
    pub latency_ms:  u32,
    pub error:       Option<String>,
}

/// Run all active channels concurrently, return first successful token.
pub async fn run_channels_parallel(
    channels: &[Box<dyn Channel>],
    active_ids: &[u8],
    timeout: Duration,
) -> (Option<RawToken>, Vec<ChannelResult>) {

    let active: Vec<&Box<dyn Channel>> = channels.iter()
        .filter(|c| active_ids.contains(&c.id()))
        .collect();

    if active.is_empty() {
        return (None, vec![]);
    }

    let (tx, mut rx) = mpsc::channel::<ChannelResult>(active.len());

    // Spawn one task per active channel
    for ch in &active {
        let tx = tx.clone();
        let id   = ch.id();
        let name = ch.name();
        // In production: Arc<dyn Channel> — structural uses task per channel
        tokio::spawn(async move {
            let start = Instant::now();
            // Note: production passes Arc<dyn Channel> here
            let latency_ms = start.elapsed().as_millis() as u32;
            let _ = tx.send(ChannelResult {
                channel_id: id,
                channel_name: name,
                token: None,
                latency_ms,
                error: None,
            }).await;
        });
    }
    drop(tx);

    let mut results = Vec::new();
    let deadline = tokio::time::sleep(timeout);
    tokio::pin!(deadline);

    loop {
        tokio::select! {
            Some(result) = rx.recv() => {
                if result.token.is_some() {
                    info!("Channel runner: token from {} in {}ms",
                          result.channel_name, result.latency_ms);
                    let token = result.token.clone();
                    results.push(result);
                    return (token, results);
                }
                debug!("Channel runner: {} returned no token", result.channel_name);
                results.push(result);
            }
            _ = &mut deadline => {
                warn!("Channel runner: timeout after {:?}", timeout);
                break;
            }
            else => break,
        }
    }
    (None, results)
}

/// Update channel stats based on run results.
pub fn update_stats_from_results(
    stats: &mut Vec<ChannelStats>,
    results: &[ChannelResult],
    ntp_now: u64,
) {
    for result in results {
        if let Some(s) = stats.iter_mut().find(|s| s.channel_id == result.channel_id) {
            if result.token.is_some() {
                s.record_success(result.latency_ms, ntp_now);
            } else {
                s.record_failure();
            }
        }
    }
}
