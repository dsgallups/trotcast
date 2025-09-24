#![allow(unused)]
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use tracing::{Level, debug, info, warn};
use trotcast::prelude::*;

fn main() {
    let (tx, mut rx) = channel::<f32>(2);

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    info!("Starting deadlock debug example with capacity 2");

    // Shared counters to track what's being sent
    let sent_1 = Arc::new(Mutex::new(Vec::new()));
    let sent_2 = Arc::new(Mutex::new(Vec::new()));

    // Get debugger if available
    #[cfg(feature = "debug")]
    let debugger = tx.debugger();

    // sender 1 and 2 can send messages by cloning tx.
    let sender_1 = thread::spawn({
        let s1 = tx.clone();
        let sent = sent_1.clone();
        move || {
            let mut x = 0.;
            loop {
                match s1.send(x) {
                    Ok(_) => {
                        debug!("Sender 1: successfully sent {}", x);
                        sent.lock().unwrap().push(x);
                        x += 1.;
                        if x > 10. {
                            warn!("Sender 1: reached value {}, stopping", x);
                            break;
                        }
                    }
                    Err(SendError::Full(v)) => {
                        debug!("Sender 1: buffer full, failed to send {}", v);
                        // Add a small delay to avoid spinning too fast
                        thread::sleep(Duration::from_micros(10));
                    }
                    Err(SendError::Disconnected(v)) => {
                        warn!("Sender 1: disconnected, failed to send {}", v);
                        break;
                    }
                }
            }
            info!("Sender 1: exiting");
        }
    });

    let sender_2 = thread::spawn({
        let s2 = tx.clone();
        let sent = sent_2.clone();
        move || {
            let mut x = 990.;
            loop {
                match s2.send(x) {
                    Ok(_) => {
                        debug!("Sender 2: successfully sent {}", x);
                        sent.lock().unwrap().push(x);
                        x /= 2.;
                        if x < 10. {
                            warn!("Sender 2: reached value {}, stopping", x);
                            break;
                        }
                    }
                    Err(SendError::Full(v)) => {
                        debug!("Sender 2: buffer full, failed to send {}", v);
                        // Add a small delay to avoid spinning too fast
                        thread::sleep(Duration::from_micros(10));
                    }
                    Err(SendError::Disconnected(v)) => {
                        warn!("Sender 2: disconnected, failed to send {}", v);
                        break;
                    }
                }
            }
            info!("Sender 2: exiting");
        }
    });

    info!("Sleeping for 1 second before starting receiver...");
    std::thread::sleep(Duration::from_secs(1));

    // Print debug state before receiving
    #[cfg(feature = "debug")]
    {
        info!("State before receiving:\n{}", debugger.print_state());
    }

    let mut count = 0;
    let mut received = Vec::new();
    loop {
        match rx.try_recv() {
            Ok(msg) => {
                info!("RX1({count}) msg: {msg}");
                received.push(msg);
                count += 1;

                // Print state every 10 messages
                #[cfg(feature = "debug")]
                if count % 10 == 0 {
                    debug!(
                        "State after {} messages:\n{}",
                        count,
                        debugger.print_state()
                    );
                }
            }
            Err(TryRecvError::Disconnected) => {
                warn!("Receiver: disconnected");
                break;
            }
            Err(TryRecvError::Empty) => {
                // Check if senders are still alive
                if sender_1.is_finished() && sender_2.is_finished() {
                    info!("Both senders finished, checking for remaining messages...");
                    // Try one more time after a small delay
                    thread::sleep(Duration::from_millis(10));
                    if rx.try_recv().is_err() {
                        break;
                    }
                }
            }
        }
    }

    // Wait for senders to finish
    let _ = sender_1.join();
    let _ = sender_2.join();

    // Print final state
    #[cfg(feature = "debug")]
    {
        info!("Final state:\n{}", debugger.print_state());
    }

    // Analyze results
    let sent_1_vals = sent_1.lock().unwrap();
    let sent_2_vals = sent_2.lock().unwrap();

    info!("=== Analysis ===");
    info!(
        "Sender 1 sent {} messages: {:?}",
        sent_1_vals.len(),
        sent_1_vals
    );
    info!(
        "Sender 2 sent {} messages: {:?}",
        sent_2_vals.len(),
        sent_2_vals
    );
    info!("Receiver received {} messages", received.len());

    // Check for duplicate or corrupted values
    let mut unique_received: Vec<f32> = received.clone();
    unique_received.sort_by(|a, b| a.partial_cmp(b).unwrap());
    unique_received.dedup();

    info!("Unique values received: {}", unique_received.len());

    // Check for unexpected values
    for val in &received {
        if !sent_1_vals.contains(val) && !sent_2_vals.contains(val) {
            warn!("Received unexpected value: {}", val);
        }
    }
}
