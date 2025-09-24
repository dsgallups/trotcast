use std::thread;

use trotcast::prelude::*;

fn main() {
    // Create a broadcast channel with a capacity of 2
    let tx = Channel::new(2);

    // Clone the channel and receiver for multiple producers/consumers
    let tx2 = tx.clone();
    let mut rx1 = tx.spawn_rx();
    let mut rx2 = rx1.clone();

    thread::spawn(move || {
        tx.blocking_send(1).unwrap();
        tx.blocking_send(2).unwrap();
    });

    thread::spawn(move || {
        tx2.blocking_send(3).unwrap();
        tx2.blocking_send(4).unwrap();
    });

    let (tx_vals, receiver_vals) = crossbeam_channel::unbounded();

    let recv_1 = thread::spawn({
        let tx = tx_vals.clone();
        move || {
            let mut count = 0;
            // Collect messages from first receiver
            while let Ok(msg) = rx1.recv() {
                tx.send((1, msg)).unwrap();
                count += 1;
                if count == 4 {
                    break;
                }
            }
        }
    });

    let recv_2 = thread::spawn({
        let tx = tx_vals.clone();
        move || {
            let mut count = 0;
            // Collect messages from first receiver
            while let Ok(msg) = rx2.recv() {
                tx.send((2, msg)).unwrap();
                count += 1;
                if count == 4 {
                    break;
                }
            }
        }
    });
    loop {
        if [&recv_1, &recv_2].iter().all(|handle| handle.is_finished()) {
            break;
        }
    }
    let mut messages1 = vec![];
    let mut messages2 = vec![];
    while let Ok((thread_no, val)) = receiver_vals.try_recv() {
        if thread_no == 1 {
            messages1.push(val)
        } else {
            messages2.push(val)
        }
    }
    // Both receivers got all messages
    assert_eq!(messages1.len(), 4);
    assert_eq!(messages2.len(), 4);

    for i in [1, 2, 3, 4] {
        assert!(messages1.contains(&i));
        assert!(messages2.contains(&i));
    }
}
