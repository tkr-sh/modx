use {
    dioxus::prelude::*,
    modx::{props, store},
};

fn main() {
    launch(app);
}

// CoutnerS
#[props(counters)]
#[store]
struct CountersStore {
    counters: Vec<CounterStore>,
}

// Counter
#[derive(Debug, PartialEq, Eq)]
#[store]
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

#[derive(PartialEq, Props, Clone)]
struct StoreProps {
    store: CounterStore,
}

fn my_button(mut props: StoreProps) -> Element {
    rsx! {
        button {
            onclick: move |_| props.store.inc(),
            "+1"
        }
        button {
            onclick: move |_| props.store.dec(),
            "-1"
        }
    }
}

fn app() -> Element {
    let a = CounterStore::new();
    let b = CounterStore::new();
    let store = CountersStore::new(CountersStoreProps {
        counters: vec![a, b],
    });

    rsx! {
        my_button { store: store.counters()[0] }
        br{}
        my_button { store: store.counters()[1] }
        br{}
        "{a.count}"
        br{}
        "{b.count}"
    }
}
