use {
    dioxus::prelude::*,
    modx::{props, resource, store},
    serde::Deserialize,
};

fn main() {
    launch(app);
}

#[resource(fetch_cat)]
#[props(number_of_cats)]
#[store(Default)]
struct CatStore {
    number_of_cats: usize,
    fetch_cat:      Result<ApiResponse, reqwest::Error>,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    id: String,
}

impl CatStore {
    fn clear(&mut self) {
        self.fetch_cat.clear();
        self.number_of_cats.set(0);
    }

    async fn fetch_cat(&mut self) -> Result<ApiResponse, reqwest::Error> {
        self.number_of_cats += 1;

        match reqwest::get("https://cataas.com/cat?json=true").await {
            Ok(rep) => rep.json::<ApiResponse>().await,
            Err(why) => Err(why),
        }
    }
}

fn app() -> Element {
    let mut store = CatStore::new(CatStoreProps { number_of_cats: 1 });

    rsx!(
        match &*store.fetch_cat.read()  {
            Some(Ok(api_response)) =>
                rsx! {
                    div {
                        img {
                            max_width: "500px",
                            max_height: "500px",
                            src: "https://cataas.com/cat/{api_response.id}"
                        }
                    }
                    "Wow! An amazing cat! (N°{store.number_of_cats})"
                },
            Some(Err(_)) => rsx! { "An error occured while getting a cat :(" },
            None => rsx!( "No cat for now." ),
        }
        button {
            onclick: move |_| store.fetch_cat.restart(),
            "Get a new cat!"
        }
        button {
            onclick: move |_| store.clear(),
            "Clear cats"
        }
    )
}
