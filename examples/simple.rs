use std::{thread, time::Duration};

use trotcast::prelude::*;

fn main() {
    let (tx, rx) = channel::<f32>(20);

    // sender 1 and 2 can send messages by cloning tx.
    let sender_1 = thread::spawn({
        let s1 = tx.clone();
        move || {
            let mut x = 0.;
            loop {
                std::thread::sleep(Duration::from_millis(200));
                s1.send(x);
                x += 1.;
            }
        }
    });

    let sender_2 = thread::spawn({
        let s2 = tx.clone();
        move || {
            let mut x = 990.;
            loop {
                std::thread::sleep(Duration::from_millis(400));
                s1.send(x);
                x /= 3.;
            }
        }
    });

    // both receiver_1 and receiver_2 will recieve the same messages

    let receiver_1 = thread::spawn({
        let rx_1 = rx.clone();
        move || {
            loop {
                if let Ok(msg) = rx_1.try_recv() {
                    println!("Value: {msg}");
                }
            }
        }
    });

    let receiver_2 = thread::spawn({
        let rx_2 = rx.clone();
        move || {
            while let Ok(msg) = rx_2.recv() {
                println!("RX2 msg: {msg}");
            }
        }
    });
}
