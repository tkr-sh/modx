use {dioxus::prelude::*, modx::{store, props}};

fn main() {
    launch(app);
}

#[store]
struct Shop {
    input: String,
    items_string: Vec<String>,
}

impl Shop {
    fn add_item(&mut self) {
        self.items_string.push(self.input());
        self.input.set(String::new())
    }

    fn get_items(&self) -> Vec<Item> {
        self
            .items_string()
            .iter()
            .map(|item| {
                Item::new(ItemProps { name: item.clone() })
            })
            .collect::<Vec<_>>()
    }

    fn on_input(&mut self, s: String) {
        self.input.set(s);
    }
}


#[derive(Debug)]
#[props(name)]
#[store]
struct Item {
    name: String,
    number_to_buy: usize,
}

impl Item {
    fn inc(&mut self) {
        self.number_to_buy += 1;
    }

    fn dec(&mut self) {
        if self.number_to_buy() > 0 {
            self.number_to_buy -= 1;
        }
    }
}



fn app() -> Element {
    let mut shop = Shop::new();


    rsx!(
        ul {
            list_style: "none",
            padding_left: "0px",
            for (idx, &mut item) in shop.get_items().iter_mut().enumerate() {
                li {
                    key: "{idx}",
                    padding: "10px",
                    margin_top: "10px",
                    background: "#eee",
                    "{item.name}"
                    div {
                        "{item.number_to_buy}"
                        button { onclick: move |_| item.clone().inc(), "+1" }
                        button { onclick: move |_| item.clone().dec(), "-1" }
                    }
                }
            }
        }
        input {
            oninput: move |e| shop.on_input(e.data().value()),
            onkeydown: move |e| if e.data().key() == Key::Enter { shop.add_item() },
            value: "{shop.input}"
        }
        button {
            onclick: move |_| shop.add_item(),
            "Add a new item!",
        }
    )
}
