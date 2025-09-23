use std::{thread, time::Duration};

use trotcast::prelude::*;

fn main() {
    let (tx, rx) = channel::<f32>(5);

    // sender 1 and 2 can send messages by cloning tx.
    let sender_1 = thread::spawn({
        let s1 = tx.clone();
        move || {
            let mut x = 0.;
            for _ in 0..500 {
                loop {
                    if s1.send(x).is_err() {
                        continue;
                    } else {
                        break;
                    }
                }
                x += 1.;
            }
        }
    });

    let sender_2 = thread::spawn({
        let s2 = tx.clone();
        move || {
            let mut x = 990.;
            for _ in 0..100 {
                std::thread::sleep(Duration::from_millis(400));
                loop {
                    if s2.send(x).is_err() {
                        continue;
                    } else {
                        break;
                    }
                }
                x /= 3.;
            }
        }
    });

    // both receiver_1 and receiver_2 will recieve the same messages

    let receiver_1 = thread::spawn({
        let mut rx_1 = rx.clone();
        move || {
            loop {
                match rx_1.try_recv() {
                    Ok(msg) => {
                        println!("RX1 msg: {msg}");
                    }
                    Err(TryRecvError::Disconnected) => {
                        break;
                    }
                    Err(_) => {}
                }
            }
        }
    });

    let receiver_2 = thread::spawn({
        let mut rx_2 = rx.clone();
        move || loop {
            match rx_2.recv() {
                Ok(msg) => {
                    println!("RX2({}) msg: {msg}", rx_2.head);
                }
                Err(e) => {
                    println!("Error: {e:?}");
                }
            }
        }
    });
    // let receiver_2 = thread::spawn({
    //     let mut rx_2 = rx.clone();
    //     move || {
    //         while let Ok(msg) = rx_2.recv() {
    //             println!("RX2({}) msg: {msg}", rx_2.head);
    //         }
    //     }
    // });

    // thread::spawn({
    //     let debug = tx.debugger();
    //     move || {
    //         loop {
    //             std::thread::sleep(Duration::from_secs(1));
    //             debug.print_state();
    //         }
    //     }
    // });

    let handles = [sender_1, sender_2, receiver_1, receiver_2];
    //let handles = [sender_1, sender_2, receiver_1, receiver_2];
    loop {
        if handles.iter().all(|handle| handle.is_finished()) {
            break;
        }
    }
}
