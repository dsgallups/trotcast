#![allow(unused)]
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use tracing::{Level, debug, error, info, warn};
use trotcast::prelude::*;

fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let (tx, mut rx) = channel::<f32>(2);

    info!("Starting deadlock diagnosis with capacity 2 (ring size 3)");

    // Get debugger for state inspection
    #[cfg(feature = "debug")]
    let debugger = Arc::new(tx.debugger());

    // Track sender states
    let sender1_value = Arc::new(AtomicU64::new(0));
    let sender2_value = Arc::new(AtomicU64::new(f64::to_bits(990.0)));
    let sender1_stuck = Arc::new(AtomicBool::new(false));
    let sender2_stuck = Arc::new(AtomicBool::new(false));

    // Clone for senders
    let s1_val = sender1_value.clone();
    let s2_val = sender2_value.clone();
    let s1_stuck = sender1_stuck.clone();
    let s2_stuck = sender2_stuck.clone();

    // Sender 1: Uses the problematic pattern
    let sender_1 = thread::spawn({
        let s1 = tx.clone();
        move || {
            let mut x = 0.;
            let mut consecutive_failures = 0;

            loop {
                if s1.send(x).is_err() {
                    consecutive_failures += 1;

                    if consecutive_failures == 100 {
                        s1_val.store(x.to_bits() as u64, Ordering::Relaxed);
                        s1_stuck.store(true, Ordering::Relaxed);
                    }

                    if consecutive_failures > 10_000_000 {
                        error!("Sender 1: Giving up after 10M failures on value {}", x);
                        break;
                    }

                    // This continues WITHOUT incrementing x!
                    continue;
                } else {
                    consecutive_failures = 0;
                    s1_stuck.store(false, Ordering::Relaxed);
                    x += 1.;
                }
            }
        }
    });

    // Sender 2: Uses nested loop pattern
    let sender_2 = thread::spawn({
        let s2 = tx.clone();
        move || {
            let mut x = 990.;
            let mut total_attempts = 0;

            loop {
                let mut inner_attempts = 0;

                loop {
                    inner_attempts += 1;
                    total_attempts += 1;

                    if s2.send(x).is_err() {
                        if inner_attempts == 100 {
                            s2_val.store(x.to_bits() as u64, Ordering::Relaxed);
                            s2_stuck.store(true, Ordering::Relaxed);
                        }

                        if total_attempts > 10_000_000 {
                            error!("Sender 2: Giving up after 10M total attempts");
                            return;
                        }
                        continue;
                    } else {
                        s2_stuck.store(false, Ordering::Relaxed);
                        break;
                    }
                }

                x /= 2.;
                if x < 0.001 {
                    x = 990.;
                }
            }
        }
    });

    // Monitor thread that checks for deadlock
    #[cfg(feature = "debug")]
    let monitor = thread::spawn({
        let s1_stuck = sender1_stuck.clone();
        let s2_stuck = sender2_stuck.clone();
        let s1_val = sender1_value.clone();
        let s2_val = sender2_value.clone();
        let debugger = Arc::clone(&debugger);

        move || {
            let mut last_check = Instant::now();
            let mut deadlock_detected = false;

            loop {
                thread::sleep(Duration::from_millis(100));

                let s1_is_stuck = s1_stuck.load(Ordering::Relaxed);
                let s2_is_stuck = s2_stuck.load(Ordering::Relaxed);

                if s1_is_stuck && s2_is_stuck && !deadlock_detected {
                    deadlock_detected = true;
                    error!("=== DEADLOCK DETECTED ===");

                    let s1_trying = f32::from_bits(s1_val.load(Ordering::Relaxed) as u32);
                    let s2_trying = f32::from_bits(s2_val.load(Ordering::Relaxed) as u32);

                    error!("Sender 1 stuck trying to send: {}", s1_trying);
                    error!("Sender 2 stuck trying to send: {}", s2_trying);
                    error!("Ring buffer state:\n{}", debugger.print_state());

                    // Print detailed seat info
                    let shared = &debugger.shared;
                    let tail = shared.tail.load(Ordering::SeqCst);
                    error!("Current tail: {}", tail);

                    for i in 0..shared.len {
                        let seat = &shared.ring[i];
                        let num_reads = seat.num_reads.load(Ordering::SeqCst);
                        let check_writing = seat.check_writing.load(Ordering::SeqCst);
                        let state = unsafe { &*seat.state.get() };

                        error!(
                            "Seat[{}]: num_reads={}, required_reads={}, check_writing={}, has_value={}",
                            i,
                            num_reads,
                            state.required_reads,
                            check_writing,
                            state.val.is_some()
                        );

                        if let Some(val) = &state.val {
                            error!("  value: {}", val);
                        }
                    }

                    // Check fence positions
                    let fence = (tail + 1) % shared.len;
                    error!("Fence position: {} (checking if this is clear)", fence);

                    let fence_reads = shared.ring[fence].num_reads.load(Ordering::SeqCst);
                    let fence_state = unsafe { &*shared.ring[fence].state.get() };
                    let fence_required = fence_state.required_reads;

                    error!(
                        "Fence status: {} reads completed out of {} required",
                        fence_reads, fence_required
                    );

                    if fence_required > fence_reads {
                        error!("FENCE BLOCKED: The fence seat hasn't been fully read!");
                    }
                }

                if last_check.elapsed() > Duration::from_secs(10) {
                    info!("Monitor: Still running...");
                    last_check = Instant::now();
                }
            }
        }
    });

    // Sleep before starting receiver (reproduces the issue)
    info!("Sleeping for 1 second before starting receiver...");
    thread::sleep(Duration::from_secs(1));

    // Print initial state before receiver starts
    #[cfg(feature = "debug")]
    {
        info!("State before receiver starts:\n{}", debugger.print_state());
    }

    info!("Starting receiver with head at position {}", rx.head);

    // Receiver loop
    let mut count = 0;
    let mut last_value = None;
    let mut same_value_count = 0;
    let start = Instant::now();

    loop {
        match rx.try_recv() {
            Ok(msg) => {
                count += 1;

                // Detect if we're receiving the same value repeatedly
                if last_value == Some(msg) {
                    same_value_count += 1;
                    if same_value_count == 100 {
                        warn!("Received same value {} 100 times in a row!", msg);
                    }
                } else {
                    if same_value_count > 10 {
                        info!(
                            "Value {} was received {} times",
                            last_value.unwrap(),
                            same_value_count
                        );
                    }
                    same_value_count = 1;
                    last_value = Some(msg);
                }

                if count % 10000 == 0 {
                    info!("Received {} messages, latest: {}", count, msg);
                }
            }
            Err(TryRecvError::Disconnected) => {
                warn!("Receiver disconnected");
                break;
            }
            Err(TryRecvError::Empty) => {
                thread::yield_now();
            }
        }

        // Run for limited time
        if start.elapsed() > Duration::from_secs(5) {
            break;
        }
    }

    info!(
        "Receiver exiting after {} messages in {:.1}s",
        count,
        start.elapsed().as_secs_f32()
    );

    // Give threads time to detect final state
    thread::sleep(Duration::from_millis(500));

    // Final state
    #[cfg(feature = "debug")]
    {
        info!("Final state:\n{}", debugger.print_state());
        info!("Receiver head position: {}", rx.head);
    }
}
