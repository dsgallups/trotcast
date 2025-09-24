use std::sync::{Arc, atomic::Ordering};

use crate::state::State;

/// Debug wrapper for accessing the internal state of the broadcast channel.
pub struct Debug<T> {
    pub shared: Arc<State<T>>,
}

impl<T> Debug<T> {
    pub fn print_state(&self) -> String {
        let mut str = String::new();
        str.push_str(&format!(
            "Tail: {}\nnum readers: {}\n",
            self.shared.tail.load(Ordering::Relaxed),
            self.shared.num_readers.load(Ordering::Relaxed)
        ));
        for (i, ring) in self.shared.ring.iter().enumerate() {
            str.push_str(&format!("Seat({i}): {ring:?}\n"));
            //todo
        }
        str
    }
}
