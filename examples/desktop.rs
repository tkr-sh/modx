use {dioxus::prelude::*, modx::modx};

fn main() {
    launch(app);
}

#[modx(Default)]
struct TodoStore {
    tasks: Vec<String>,
    value: String,
}

impl TodoStore {
    fn add_todo(&mut self) {
        self.tasks.push(self.value());
        self.value.set(String::new());
    }

    fn pop_todo(&mut self) {
        self.tasks.pop();
    }

    fn update_value(&mut self, s: String) {
        self.value.set(s);
    }
}

fn app() -> Element {
    let mut store = TodoStore::default();

    rsx!(
        input {
            oninput: move |e| store.update_value(e.data().value()),
            onkeydown: move |e| if e.data().key() == Key::Enter { store.add_todo() },
            value: "{store.value}"
        }
        button {
            onclick: move |_| store.add_todo(),
            "Add Task"
        }
        button {
            onclick: move |_| store.pop_todo(),
            "Pop Task"
        }
        ul {
            for (idx, task) in store.tasks().iter().enumerate() {
                li {
                    key: "{idx}",
                    "{task}"
                }
            }
        }
    )
}
