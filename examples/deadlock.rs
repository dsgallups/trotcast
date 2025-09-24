#![allow(unused)]
use std::{collections::VecDeque, thread, time::Duration};

use tracing::{Level, info, warn};
use trotcast::prelude::*;

fn main() {
    let (tx, mut rx) = channel::<f32>(2);

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // sender 1 and 2 can send messages by cloning tx.
    let sender_1 = thread::spawn({
        let s1 = tx.clone();
        move || {
            let mut x = 0.;
            loop {
                if s1.send(x).is_err() {
                    continue;
                } else {
                    x += 1.;
                }
            }
        }
    });

    let sender_2 = thread::spawn({
        let s2 = tx.clone();
        move || {
            let mut x = 990.;
            loop {
                loop {
                    if s2.send(x).is_err() {
                        continue;
                    } else {
                        break;
                    }
                }
                x /= 2.;
            }
        }
    });

    std::thread::sleep(Duration::from_secs(1));

    let mut count = 0;
    loop {
        match rx.try_recv() {
            Ok(msg) => {
                info!("RX1({count}) msg: {msg}");
                count += 1;
            }
            Err(TryRecvError::Disconnected) => {
                break;
            }
            Err(_) => {}
        }
    }
}
