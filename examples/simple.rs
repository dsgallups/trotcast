use std::{thread, time::Duration};

use trotcast::prelude::*;
fn main() {
    let (tx, rx) = channel::<f32>(20);

    // sender 1 and 2 can send messages by cloning tx.
    thread::spawn({
        let s1 = tx.clone();
        move || {
            for i in 0..=3 {
                std::thread::sleep(Duration::from_millis(200));
                s1.send(i as f32).unwrap();
            }
        }
    });

    thread::spawn({
        let s2 = tx.clone();
        move || {
            let mut x = 990.;
            for _ in 0..5 {
                std::thread::sleep(Duration::from_millis(400));
                s2.send(x).unwrap();
                x /= 3.;
            }
        }
    });

    // both receiver_1 and receiver_2 will recieve the same messages

    thread::spawn({
        let mut rx_1 = rx.clone();
        move || {
            let mut count = 0;
            loop {
                match rx_1.try_recv() {
                    Ok(msg) => {
                        println!("RX1({count}) msg: {msg}");
                        count += 1;
                    }
                    Err(TryRecvError::Disconnected) => {
                        break;
                    }
                    Err(_) => {}
                }
            }
        }
    });

    thread::spawn({
        let mut rx_2 = rx.clone();
        move || {
            let mut count = 0;
            while let Ok(msg) = rx_2.recv() {
                println!("RX2({count}) msg: {msg}");
                count += 1;
            }
        }
    });

    thread::sleep(Duration::from_secs(5));
}
