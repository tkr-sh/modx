use {dioxus::prelude::*, modx::store};

fn main() {
    launch(app);
}

#[store(Default)]
struct CounterStore {
    count: i64,
}

impl CounterStore {
    fn inc(&mut self) {
        self.count += 1;
    }

    fn dec(&mut self) {
        self.count -= 1;
    }
}

fn app() -> Element {
    let mut store = CounterStore::new();
    rsx! {
        button { onclick: move |_| store.inc(), "+1" }
        button { onclick: move |_| store.dec(), "-1" }
        "{store.count}"
    }
}
