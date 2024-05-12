use dioxus::prelude::*;
mod pub_store;
use pub_store::CounterStore;


fn main() {
    launch(app);
}

fn app() -> Element {
    let mut store = CounterStore::new();
    rsx! {
        button { onclick: move |_| store.inc(), "+1" }
        button { onclick: move |_| store.dec(), "-1" }
        "{store.count}"
    }
}

