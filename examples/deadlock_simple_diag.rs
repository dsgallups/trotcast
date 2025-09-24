#![allow(unused)]
use std::{
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
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let (tx, mut rx) = channel::<f32>(2);

    info!("Starting deadlock diagnosis with capacity 2");

    // Get debugger for state inspection
    #[cfg(feature = "debug")]
    let debugger = Arc::new(tx.debugger());

    // Track when senders get stuck
    let deadlock_detected = Arc::new(AtomicBool::new(false));
    let sender1_attempts = Arc::new(AtomicU64::new(0));
    let sender2_attempts = Arc::new(AtomicU64::new(0));

    // Sender 1: The problematic pattern from original deadlock.rs
    let sender_1 = thread::spawn({
        let s1 = tx.clone();
        let attempts = sender1_attempts.clone();
        let deadlock = deadlock_detected.clone();

        move || {
            let mut x = 0.;
            let mut consecutive_failures = 0;

            loop {
                if deadlock.load(Ordering::Relaxed) {
                    break;
                }

                attempts.fetch_add(1, Ordering::Relaxed);

                if s1.send(x).is_err() {
                    consecutive_failures += 1;
                    // CRITICAL: This continues WITHOUT incrementing x!
                    continue;
                } else {
                    consecutive_failures = 0;
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
        let deadlock = deadlock_detected.clone();

        move || {
            let mut x = 990.;

            loop {
                if deadlock.load(Ordering::Relaxed) {
                    break;
                }

                // Keep trying until successful
                loop {
                    attempts.fetch_add(1, Ordering::Relaxed);

                    if s2.send(x).is_err() {
                        continue;
                    } else {
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

    // Monitor thread
    let monitor = thread::spawn({
        let s1_att = sender1_attempts.clone();
        let s2_att = sender2_attempts.clone();
        let deadlock = deadlock_detected.clone();
        #[cfg(feature = "debug")]
        let dbg = Arc::clone(&debugger);

        move || {
            let mut last_s1 = 0;
            let mut last_s2 = 0;
            let mut stuck_count = 0;

            loop {
                thread::sleep(Duration::from_millis(100));

                if deadlock.load(Ordering::Relaxed) {
                    break;
                }

                let s1_curr = s1_att.load(Ordering::Relaxed);
                let s2_curr = s2_att.load(Ordering::Relaxed);

                // Check if both are making attempts but not progressing
                if s1_curr > last_s1 + 1000 && s2_curr > last_s2 + 1000 {
                    let s1_rate = s1_curr - last_s1;
                    let s2_rate = s2_curr - last_s2;

                    info!(
                        "S1: {} attempts/100ms, S2: {} attempts/100ms",
                        s1_rate, s2_rate
                    );

                    stuck_count += 1;

                    if stuck_count > 10 {
                        // Stuck for 1 second
                        error!("=== DEADLOCK DETECTED ===");
                        error!("Both senders spinning without progress!");

                        #[cfg(feature = "debug")]
                        {
                            error!("Current ring buffer state:");
                            error!("{}", dbg.print_state());
                        }

                        deadlock.store(true, Ordering::Relaxed);
                        break;
                    }
                } else {
                    stuck_count = 0;
                }

                last_s1 = s1_curr;
                last_s2 = s2_curr;
            }
        }
    });

    // Sleep before starting receiver (reproduces the issue)
    info!("Sleeping for 1 second before starting receiver...");
    thread::sleep(Duration::from_secs(1));

    // Print state before receiving
    #[cfg(feature = "debug")]
    {
        info!("State before receiver starts:");
        info!("{}", debugger.print_state());
        info!("Receiver head position: {}", rx.head);
    }

    // Receiver loop
    let mut count = 0;
    let mut values_received = std::collections::HashMap::new();
    let start = Instant::now();
    let mut last_report = Instant::now();

    loop {
        match rx.try_recv() {
            Ok(msg) => {
                count += 1;
                *values_received.entry(msg.to_bits()).or_insert(0) += 1;

                if count % 10000 == 0 {
                    info!("Received {} messages so far", count);
                }
            }
            Err(TryRecvError::Disconnected) => {
                warn!("Receiver disconnected");
                break;
            }
            Err(TryRecvError::Empty) => {
                // Check if deadlock was detected
                if deadlock_detected.load(Ordering::Relaxed) {
                    warn!("Stopping receiver due to deadlock detection");
                    break;
                }

                // Report periodically
                if last_report.elapsed() > Duration::from_secs(1) {
                    info!("Receiver: {} total messages received", count);

                    // Show most common values
                    let mut freq: Vec<_> = values_received.iter().collect();
                    freq.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

                    if !freq.is_empty() {
                        info!("Most common values:");
                        for (bits, count) in freq.iter().take(5) {
                            let val = f32::from_bits(**bits);
                            info!("  {}: {} times", val, count);
                        }
                    }

                    #[cfg(feature = "debug")]
                    {
                        info!("Current state:");
                        info!("{}", debugger.print_state());
                        info!("Receiver head: {}", rx.head);
                    }

                    last_report = Instant::now();
                }

                thread::yield_now();
            }
        }

        // Run for limited time
        if start.elapsed() > Duration::from_secs(10) && !deadlock_detected.load(Ordering::Relaxed) {
            info!("Time limit reached, stopping...");
            break;
        }
    }

    // Signal threads to stop
    deadlock_detected.store(true, Ordering::Relaxed);

    // Wait for threads
    let _ = sender_1.join();
    let _ = sender_2.join();
    let _ = monitor.join();

    info!("Final results:");
    info!("  Total messages received: {}", count);
    info!("  Unique values: {}", values_received.len());

    // Final state
    #[cfg(feature = "debug")]
    {
        info!("Final state:");
        info!("{}", debugger.print_state());
        info!("Final receiver head: {}", rx.head);
    }
}
