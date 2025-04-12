use {dioxus::prelude::*, modx::store};

#[store(Default)]
pub struct CounterStore {
    pub count: i64,
}

impl CounterStore {
    pub fn inc(&mut self) {
        self.count += 1;
    }

    pub fn dec(&mut self) {
        self.count -= 1;
    }
}

#[allow(dead_code, reason = "Just checking that it compiles")]
const fn main() {}
