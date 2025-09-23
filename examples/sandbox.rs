#![allow(unused)]
use std::{collections::VecDeque, thread, time::Duration};

use trotcast::prelude::*;

fn main() {
    let (tx, rx) = channel::<f32>(5);

    // sender 1 and 2 can send messages by cloning tx.
    let sender_1 = thread::spawn({
        let s1 = tx.clone();
        move || {
            let mut x = 0.;
            for _ in 0..2000 {
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

    let (tx_vals, receiver_vals) = crossbeam_channel::unbounded();

    // both receiver_1 and receiver_2 will recieve the same messages

    thread::spawn({
        let mut rx_1 = rx.clone();
        let tx = tx_vals.clone();
        move || {
            let mut count = 0;
            loop {
                match rx_1.try_recv() {
                    Ok(msg) => {
                        println!("RX1({count}) msg: {msg}");
                        _ = tx.send((1, msg, count, rx_1.head));
                        count += 1;
                        std::thread::sleep(Duration::from_millis(100));
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
        let tx = tx_vals.clone();
        move || {
            let mut count = 0;
            loop {
                match rx_2.recv() {
                    Ok(msg) => {
                        println!("RX2({count}) msg: {msg}");
                        _ = tx.send((2, msg, count, rx_2.head));
                        count += 1;
                    }
                    Err(e) => {
                        println!("Error: {e:?}");
                    }
                }
            }
        }
    });

    let debug = tx.debugger();
    let mut recv1 = Vec::new();
    let mut recv2 = Vec::new();
    let mut statements = VecDeque::new();
    let mut running_max = 0;
    while let Ok((thread_no, val, count, head)) = receiver_vals.recv() {
        if thread_no == 1 {
            recv1.push((val, count, head));
        } else {
            recv2.push((val, count, head));
        }
        for ((val1, c1, c1_head), (val2, c2, c2_head)) in recv1.iter().zip(recv2.iter()) {
            assert_eq!(c1, c2);
            if *c1 > running_max {
                running_max += 1;
                statements.push_back((debug.print_state(), *c1_head, *c2_head, count));
                if statements.len() > 8 {
                    statements.pop_front();
                }
            }

            if val1 != val2 {
                let final_state = debug.print_state();

                let mut past_strs = String::new();
                for (statement, c1_head, c2_head, count) in statements {
                    past_strs.push_str(&format!("State at {count}:\n{statement}\n\nthread1 head: {c1_head}\nthread2 head: {c2_head}\n\n======="));
                }

                panic!(
                    "unequal (RX@{c1}: {val1}, RX@{c2}): {val2}\n{recv1:?}\n{recv2:?}\n{final_state}\n\n{past_strs}"
                );
            }
        }
    }
}
