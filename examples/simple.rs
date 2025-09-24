use std::{thread, time::Duration};

use tracing::{Level, info};
use trotcast::prelude::*;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    let (tx, rx) = channel::<f32>(20);

    thread::spawn({
        let s1 = tx.clone();
        move || {
            let mut x = 0.;
            for _ in 0..500 {
                std::thread::sleep(Duration::from_millis(200));
                loop {
                    if s1.send(x).is_ok() {
                        break;
                    };
                }
                x += 1.;
            }
        }
    });

    thread::spawn({
        let s2 = tx.clone();
        move || {
            let mut x = 990.;
            for _ in 0..100 {
                std::thread::sleep(Duration::from_millis(400));
                loop {
                    if s2.send(x).is_ok() {
                        break;
                    }
                }
                //s2.send(x).unwrap();
                x /= 3.;
            }
        }
    });

    let (tx_vals, receiver_vals) = crossbeam_channel::unbounded();

    thread::spawn({
        let mut rx_1 = rx.clone();
        let tx = tx_vals.clone();
        move || {
            let mut count = 0;
            loop {
                match rx_1.try_recv() {
                    Ok(msg) => {
                        info!("RX1 msg: {msg}");

                        _ = tx.send((1, msg, count, rx_1.head));
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
        let mut rx_2 = rx;
        let tx = tx_vals.clone();
        move || {
            let mut count = 0;
            while let Ok(msg) = rx_2.recv() {
                info!("RX2({count}) msg: {msg}");
                _ = tx.send((2, msg, count, rx_2.head));
                count += 1;
            }
        }
    });

    let dbger = tx.debugger();
    let mut rx_1_head = None;
    let mut rx_2_head = None;
    loop {
        std::thread::sleep(Duration::from_secs(1));

        while let Ok((id, _, _, head)) = receiver_vals.try_recv() {
            if id == 1 {
                rx_1_head = Some(head);
            } else {
                rx_2_head = Some(head);
            }
        }
        info!(
            "{}\nrx_1_head: {rx_1_head:?}\nrx_2_head: {rx_2_head:?}",
            dbger.print_state()
        );
    }
}
