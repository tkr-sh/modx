<h1 align="center"> modx </h1>

<p align="center">
    <a href="https://github.com/tkr-sh/modx">
        <img
            src="https://img.shields.io/github/v/release/tkr-sh/modx?colorA=363a4f&colorB=a6da95&style=for-the-badge&logo=github&logoColor=cad3f5"
            alt="github release"
        />
    </a>
    <a href="https://crates.io/crates/modx">
        <img
            src="https://img.shields.io/crates/d/modx.svg?colorA=363a4f&colorB=b7bdf8&style=for-the-badge&logo=rust&logoColor=cad3f5"
            alt="crates.io downloads"
        />
    </a>
    <a href="https://docs.rs/modx">
        <img
            src="https://img.shields.io/badge/docs-latest-blue.svg?colorA=363a4f&colorB=f5a97f&style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNTYgMjU2Ij4KPHBhdGggZD0iTTIxNiwzMlYxOTJhOCw4LDAsMCwxLTgsOEg3MmExNiwxNiwwLDAsMC0xNiwxNkgxOTJhOCw4LDAsMCwxLDAsMTZINDhhOCw4LDAsMCwxLTgtOFY1NkEzMiwzMiwwLDAsMSw3MiwyNEgyMDhBOCw4LDAsMCwxLDIxNiwzMloiIHN0eWxlPSJmaWxsOiAjQ0FEM0Y1OyIvPgo8L3N2Zz4="
            alt="docs.rs docs"
        />
    </a>
    <a href="https://github.com/tkr-sh/modx">
        <img
            src="https://img.shields.io/github/stars/tkr-sh/modx?colorA=363a4f&colorB=eed49f&style=for-the-badge&logo=star"
            alt="stars"
        />
    </a>
</p>

modx is an experimental way to handle states with structs in Dioxus inspired by [mobx](https://mobx.js.org/README.html).


```rs
#[modx::store]
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
```


## Development progress

In the near future, most of the hooks should be rewrote to work with modx.

Here is the current status:

- [X] Signals
- [X] Props
- [X] Resources
- [ ] Memo
- [ ] Server future
