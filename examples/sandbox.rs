#![allow(unused)]
use std::{collections::VecDeque, thread, time::Duration};

use tracing::{Level, info, warn};
use trotcast::prelude::*;

fn main() {
    let (tx, rx) = channel::<f32>(4);

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // sender 1 and 2 can send messages by cloning tx.
    // let sender_1 = thread::spawn({
    //     let s1 = tx.clone();
    //     move || {
    //         let mut x = 0.;
    //         for _ in 0..2000 {
    //             loop {
    //                 if s1.send(x).is_err() {
    //                     continue;
    //                 } else {
    //                     break;
    //                 }
    //             }
    //             x += 1.;
    //         }
    //     }
    // });
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
                //std::thread::sleep(Duration::from_millis(400));
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
    /*

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
    */

    let (tx_vals, receiver_vals) = crossbeam_channel::unbounded();

    std::thread::sleep(Duration::from_secs(1));
    thread::spawn({
        let mut rx_1 = rx;
        let tx = tx_vals.clone();
        move || {
            let mut count = 0;
            loop {
                match rx_1.try_recv() {
                    Ok(msg) => {
                        info!("RX1({count}) msg: {msg}");
                        _ = tx.send((1, msg, count, rx_1.head));
                        count += 1;
                        //std::thread::sleep(Duration::from_millis(100));
                    }
                    Err(TryRecvError::Disconnected) => {
                        break;
                    }
                    Err(_) => {}
                }
            }
        }
    });

    // Thoughts:
    //
    // I think it's possible that the receiver's head is sitting behind the tail which is no good.

    thread::spawn({
        let dbger = tx.debugger();
        move || {
            let mut rx_1_head = None;
            let mut rx_2_head = None;
            loop {
                std::thread::sleep(Duration::from_secs(1));

                info!("{}", dbger.print_state());
                while let Ok((id, _, _, head)) = receiver_vals.try_recv() {
                    if id == 1 {
                        rx_1_head = Some(head);
                    } else {
                        rx_2_head = Some(head);
                    }
                }
                info!("rx_1_head: {rx_1_head:?}\nrx_2_head: {rx_2_head:?}");
            }
        }
    });

    let mut i = 2;
    loop {
        // std::thread::sleep(Duration::from_secs(2));

        // for i in 0..((i % 4) + 2) {
        //     std::thread::sleep(Duration::from_millis(2));
        //     thread::spawn({
        //         let mut my_rx = tx.spawn_rx();
        //         let tx = tx_vals.clone();
        //         move || {
        //             let mut count = 0;
        //             loop {
        //                 match my_rx.recv() {
        //                     Ok(msg) => {
        //                         info!("RX{i}({count}) msg: {msg}");
        //                         //_ = tx.send((2, msg, count, my_rx.head));
        //                         count += 1;
        //                         if count > 30 {
        //                             warn!("Dropped RX{i}");
        //                             break;
        //                         }
        //                     }
        //                     Err(e) => {
        //                         info!("Error: {e:?}");
        //                     }
        //                 }
        //             }
        //         }
        //     });
        // }

        // i += 1;
    }

    // let mut recv1 = Vec::new();
    // let mut recv2 = Vec::new();
    // let mut statements = VecDeque::new();
    // let mut running_max = 0;
    // while let Ok((thread_no, val, count, head)) = receiver_vals.recv() {
    //     if thread_no == 1 {
    //         recv1.push((val, count, head));
    //     } else {
    //         recv2.push((val, count, head));
    //     }
    //     for ((val1, c1, c1_head), (val2, c2, c2_head)) in recv1.iter().zip(recv2.iter()) {
    //         assert_eq!(c1, c2);
    //         if *c1 > running_max {
    //             running_max += 1;
    //             statements.push_back((debug.print_state(), *c1_head, *c2_head, count));
    //             if statements.len() > 8 {
    //                 statements.pop_front();
    //             }
    //         }

    //         if val1 != val2 {
    //             let final_state = debug.print_state();

    //             let mut past_strs = String::new();
    //             for (statement, c1_head, c2_head, count) in statements {
    //                 past_strs.push_str(&format!("State at {count}:\n{statement}\n\nthread1 head: {c1_head}\nthread2 head: {c2_head}\n\n======="));
    //             }
    //             //info!("unequal (RX@{c1}: {val1}, RX@{c2}): {val2}\n{recv1:?}\n{recv2:?}");

    //             panic!(
    //                 "unequal (RX@{c1}: {val1}, RX@{c2}): {val2}\n{recv1:?}\n{recv2:?}\n{final_state}\n\n{past_strs}"
    //             );
    //         }
    //     }
    // }
}
