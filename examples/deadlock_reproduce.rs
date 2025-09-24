#![allow(unused)]
use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use tracing::{Level, debug, error, info, warn};
use trotcast::prelude::*;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let (tx, mut rx) = channel::<f32>(2);

    info!("Starting deadlock reproduction with capacity 2");

    // Shared state to track what's happening
    let sender1_attempts = Arc::new(AtomicU64::new(0));
    let sender1_successes = Arc::new(AtomicU64::new(0));
    let sender2_attempts = Arc::new(AtomicU64::new(0));
    let sender2_successes = Arc::new(AtomicU64::new(0));
    let should_stop = Arc::new(AtomicBool::new(false));

    // Clone for monitoring thread
    let s1_att = sender1_attempts.clone();
    let s1_succ = sender1_successes.clone();
    let s2_att = sender2_attempts.clone();
    let s2_succ = sender2_successes.clone();
    let stop = should_stop.clone();

    // Monitor thread to report progress
    let monitor = thread::spawn(move || {
        let start = Instant::now();
        let mut last_s1_succ = 0;
        let mut last_s2_succ = 0;

        while !stop.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_secs(1));

            let s1_att_val = s1_att.load(Ordering::Relaxed);
            let s1_succ_val = s1_succ.load(Ordering::Relaxed);
            let s2_att_val = s2_att.load(Ordering::Relaxed);
            let s2_succ_val = s2_succ.load(Ordering::Relaxed);

            let s1_progress = s1_succ_val - last_s1_succ;
            let s2_progress = s2_succ_val - last_s2_succ;

            info!(
                "[{:.1}s] S1: {}/{} attempts (+{} msgs/sec), S2: {}/{} attempts (+{} msgs/sec)",
                start.elapsed().as_secs_f32(),
                s1_succ_val,
                s1_att_val,
                s1_progress,
                s2_succ_val,
                s2_att_val,
                s2_progress
            );

            // Detect potential deadlock
            if s1_progress == 0 && s2_progress == 0 && s1_att_val > 1000 && s2_att_val > 1000 {
                error!("POTENTIAL DEADLOCK DETECTED: No progress from either sender!");
            }

            last_s1_succ = s1_succ_val;
            last_s2_succ = s2_succ_val;
        }
    });

    // Sender 1: The problematic pattern from original deadlock.rs
    let sender_1 = thread::spawn({
        let s1 = tx.clone();
        let attempts = sender1_attempts.clone();
        let successes = sender1_successes.clone();
        let stop = should_stop.clone();

        move || {
            let mut x = 0.;
            let mut consecutive_failures = 0;

            while !stop.load(Ordering::Relaxed) {
                attempts.fetch_add(1, Ordering::Relaxed);

                if s1.send(x).is_err() {
                    consecutive_failures += 1;

                    if consecutive_failures > 1_000_000 {
                        warn!(
                            "Sender 1: {} consecutive failures trying to send {}",
                            consecutive_failures, x
                        );
                        consecutive_failures = 0;
                    }

                    // CRITICAL: This continues WITHOUT incrementing x!
                    continue;
                } else {
                    if consecutive_failures > 1000 {
                        debug!(
                            "Sender 1: succeeded after {} failures",
                            consecutive_failures
                        );
                    }
                    consecutive_failures = 0;
                    successes.fetch_add(1, Ordering::Relaxed);
                    x += 1.;
                }
            }

            info!("Sender 1: exiting");
        }
    });

    // Sender 2: Uses nested loop pattern from original
    let sender_2 = thread::spawn({
        let s2 = tx.clone();
        let attempts = sender2_attempts.clone();
        let successes = sender2_successes.clone();
        let stop = should_stop.clone();

        move || {
            let mut x = 990.;

            while !stop.load(Ordering::Relaxed) {
                let mut inner_attempts = 0;

                // Keep trying until successful
                loop {
                    attempts.fetch_add(1, Ordering::Relaxed);
                    inner_attempts += 1;

                    if s2.send(x).is_err() {
                        if inner_attempts > 1_000_000 {
                            warn!("Sender 2: {} attempts trying to send {}", inner_attempts, x);
                            inner_attempts = 0;
                        }
                        continue;
                    } else {
                        if inner_attempts > 1000 {
                            debug!("Sender 2: succeeded after {} attempts", inner_attempts);
                        }
                        successes.fetch_add(1, Ordering::Relaxed);
                        break;
                    }
                }

                x /= 2.;

                // Reset to avoid underflow
                if x < 0.001 {
                    x = 990.;
                }
            }

            info!("Sender 2: exiting");
        }
    });

    // Wait before starting receiver (like original)
    info!("Waiting 1 second before starting receiver...");
    thread::sleep(Duration::from_secs(1));

    // Receiver
    let start_time = Instant::now();
    let mut count = 0;
    let mut last_report = Instant::now();
    let mut values_seen = std::collections::HashMap::new();

    loop {
        match rx.try_recv() {
            Ok(msg) => {
                count += 1;
                *values_seen.entry(msg.to_bits()).or_insert(0) += 1;

                // Report every second
                if last_report.elapsed() > Duration::from_secs(1) {
                    info!("Receiver: {} total messages received", count);

                    // Show most common values
                    let mut freq: Vec<_> = values_seen.iter().collect();
                    freq.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

                    if freq.len() > 5 {
                        info!("Top 5 values received:");
                        for (bits, count) in freq.iter().take(5) {
                            let val = f32::from_bits(**bits);
                            info!("  {}: {} times", val, count);
                        }
                    }

                    last_report = Instant::now();
                }
            }
            Err(TryRecvError::Disconnected) => {
                warn!("Receiver: disconnected");
                break;
            }
            Err(TryRecvError::Empty) => {
                // Don't spin too fast
                thread::yield_now();
            }
        }

        // Run for 10 seconds total
        if start_time.elapsed() > Duration::from_secs(10) {
            info!("Test duration reached, stopping...");
            break;
        }
    }

    // Signal threads to stop
    should_stop.store(true, Ordering::Relaxed);

    // Give threads time to exit cleanly
    thread::sleep(Duration::from_millis(100));

    // Join threads
    let _ = sender_1.join();
    let _ = sender_2.join();
    let _ = monitor.join();

    info!(
        "Final stats: Received {} messages in {:.1} seconds",
        count,
        start_time.elapsed().as_secs_f32()
    );
}
