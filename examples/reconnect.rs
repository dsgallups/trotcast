fn main() {
    let tx = trotcast::channel::<i32>(4);

    tx.send(5).unwrap_err();

    let mut rx = tx.spawn_rx();
    tx.send(6).unwrap();
    let val = rx.try_recv().unwrap();
    assert_eq!(val, 6);
    drop(rx);
    tx.send(42).unwrap_err();

    let mut rx1 = tx.spawn_rx();
    let mut rx2 = tx.spawn_rx();
    tx.send(90).unwrap();
    let val1 = rx1.try_recv().unwrap();
    let val2 = rx2.try_recv().unwrap();
    assert_eq!(val1, 90);
    assert_eq!(val2, 90);
}
