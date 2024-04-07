# modx

modx is an experimental way to handle states with structs in Dioxus inspired by [mobx](https://mobx.js.org/README.html).


```rs
#[modx(Default)]
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
    let mut store = CounterStore::default();
    rsx! {
        button { onclick: move |_| store.inc(), "+1" }
        button { onclick: move |_| store.dec(), "-1" }
        "{store.count}"
    }
}
```
