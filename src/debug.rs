use std::sync::Arc;

use crate::state::State;

pub struct Debug<T> {
    pub shared: Arc<State<T>>,
}

impl<T> Debug<T> {
    pub fn print_state(&self) {
        let mut str = String::new();
        for (i, ring) in self.shared.ring.iter().enumerate() {
            str.push_str(&format!("Seat({i}): {ring:?}\n"));
            //todo
        }
        println!("{str}");
    }
}
