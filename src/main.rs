use log::info;
use web_sys::{HtmlInputElement, InputEvent};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(serde::Serialize, serde::Deserialize)]
struct Query {
    content: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
struct Musical {
    id: u64,
    name: String,
    url: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
struct ListItem {
    id: u64,
    musical_id: u64,
    viewed: bool,
    rating: u8,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
struct MusicaList {
    version: u8,
    author: String,
    items: Vec<ListItem>,
}

#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
    #[at("/musicalist/")]
    Home,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! {
            <Home />
        },
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

static MUSICALS: once_cell::sync::Lazy<Vec<Musical>> = once_cell::sync::Lazy::new(|| {
    let musicals_csv = include_str!("musicals.csv");
    let mut musicals: Vec<Musical> = vec![];
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(musicals_csv.as_bytes());
    for record in reader.deserialize::<Musical>() {
        match record {
            Ok(musical) => musicals.push(musical),
            Err(err) => info!("{:?}", err),
        };
    }
    musicals
});

#[function_component(Home)]
fn home() -> Html {
    wasm_logger::init(wasm_logger::Config::default());
    let default_list = MusicaList {
        version: 1,
        author: "".to_string(),
        items: vec![],
    };
    let bookmark_url = use_state(|| "".to_string());

    let navigator = use_navigator().unwrap();

    let current_location = use_location().unwrap();

    let content = current_location
        .query::<Query>()
        .map_or("".to_string(), |query| query.content);
    let list_value: MusicaList = base64::decode(content)
        .map(|bytes| rmp_serde::from_read_ref(&bytes).unwrap_or(default_list.clone()))
        .unwrap_or(default_list);

    let list = use_state(|| list_value.clone());

    let delete = |id| {
        let list = list.clone();
        let url = bookmark_url.clone();
        let navigator = navigator.clone();
        move |_| {
            let list_out = MusicaList {
                version: list.version,
                author: list.author.clone(),
                items: list
                    .items
                    .iter()
                    .filter(|item| item.id != id)
                    .cloned()
                    .collect(),
            };
            let (new_url, content) = get_url(&list_out);
            url.set(new_url);
            let _ = navigator.push_with_query(&Route::Home, &Query { content });
            list.set(list_out);
        }
    };

    fn update_item_in_list(
        list: &MusicaList,
        id: u64,
        f: impl Fn(&ListItem) -> ListItem,
    ) -> MusicaList {
        MusicaList {
            version: list.version,
            author: list.author.clone(),
            items: list
                .items
                .iter()
                .map(
                    |item: &ListItem| {
                        if item.id == id {
                            f(item)
                        } else {
                            item.clone()
                        }
                    },
                )
                .collect(),
        }
    }

    let change_viewed = |id| {
        let list = list.clone();
        let url = bookmark_url.clone();
        let navigator = navigator.clone();
        move |_| {
            let list_out = update_item_in_list(&list, id, |item| ListItem {
                viewed: !item.viewed,
                ..item.clone()
            });
            let (new_url, content) = get_url(&list_out);
            url.set(new_url);
            let _ = navigator.push_with_query(&Route::Home, &Query { content });
            list.set(list_out);
        }
    };

    let update_rating = |id: u64, delta: i8| {
        let list = list.clone();
        let url = bookmark_url.clone();
        let navigator = navigator.clone();
        move |_| {
            let list_out = update_item_in_list(&list, id, |item| ListItem {
                rating: {
                    let new_rating = item.rating as i8 + delta;
                    if new_rating < 0 {
                        10
                    } else if new_rating > 10 {
                        0
                    } else {
                        new_rating as u8
                    }
                },
                ..item.clone()
            });
            let (new_url, content) = get_url(&list_out);
            url.set(new_url);
            let _ = navigator.push_with_query(&Route::Home, &Query { content });
            list.set(list_out);
        }
    };

    let change_musical = |id: u64| {
        let list = list.clone();
        let url = bookmark_url.clone();
        let navigator = navigator.clone();
        move |e: Event| {
            let list_out = update_item_in_list(&list, id, |item| ListItem {
                musical_id: e
                    .target_unchecked_into::<HtmlInputElement>()
                    .value()
                    .parse::<u64>()
                    .unwrap(),
                ..item.clone()
            });
            let (new_url, content) = get_url(&list_out);
            url.set(new_url);
            let _ = navigator.push_with_query(&Route::Home, &Query { content });
            list.set(list_out);
        }
    };

    fn get_url(list: &MusicaList) -> (String, String) {
        let val = MusicaList {
            version: list.version,
            author: list.author.clone(),
            items: list.items.clone(),
        };
        let str = rmp_serde::to_vec(&val).unwrap();
        // convert str to base64
        let str = base64::encode(str);
        (format!("?content={}", str), str)
    }

    let update_author = {
        let url = bookmark_url.clone();
        let navigator = navigator.clone();
        let list = list.clone();
        Callback::from(move |e: InputEvent| {
            let list_out = MusicaList {
                version: (*list).clone().version,
                author: e
                    .target_unchecked_into::<HtmlInputElement>()
                    .value()
                    .clone(),
                items: (*list).clone().items,
            };
            let (new_url, content) = get_url(&list_out);
            url.set(new_url);
            let _ = navigator.push_with_query(&Route::Home, &Query { content });
            list.set(list_out);
        })
    };

    let add_musical = {
        let url = bookmark_url.clone();
        let navigator = navigator.clone();
        let list = list.clone();
        move |_| {
            let mut items = list.items.clone();
            items.push(ListItem {
                id: list.items.len() as u64 + 1,
                musical_id: 1,
                viewed: false,
                rating: 0,
            });
            let list_out = MusicaList {
                version: list.version,
                author: list.author.clone(),
                items,
            };
            let (new_url, content) = get_url(&list_out);
            url.set(new_url);
            let _ = navigator.push_with_query(&Route::Home, &Query { content });
            list.set(list_out);
        }
    };

    fn get_musical_url(musical_id: u64) -> String {
        format!(
            "https://en.wikipedia.org/wiki/{}",
            MUSICALS
                .iter()
                .find(|m| m.id == musical_id)
                .map(|m| m.url.clone())
                .unwrap_or("".to_string())
        )
    }

    html! {
        <>
        { "Musicalist for " }
        <input type="text" value={ (*list).clone().author } oninput={update_author}/>
        <br/>
        <br/>
        <table class={"center"}>
            <tr>
                <th>{ "Musical" }</th>
                <th>{ "Wiki" }</th>
                <th>{ "Viewed" }</th>
                <th>{ "Rating"}</th>
                <th>{ "actions" }</th>
            </tr>
            { for (*list).clone().items.iter().map(|item| {
                html! {
                    <tr>
                        <td>
                        <select onchange={change_musical(item.id)}>
                            { for MUSICALS.iter().map(|m| {
                                if m.id == item.musical_id {
                                    html! {
                                        <option value={ format!("{}", m.id) } selected=true>{ &m.name }</option>
                                    }
                                } else {
                                    html! {
                                    }
                                }
                            })}
                            { for MUSICALS.iter().map(|m| {
                                if m.id != item.musical_id {
                                html! {
                                    <option value={ format!("{}", m.id) }>{ &m.name }</option>
                                }
                                } else {
                                    html! {
                                    }
                                }
                            })}
                        </select>
                        </td>
                        <td>
                        <a href={get_musical_url(item.musical_id)}>{"?"}</a>
                        </td>
                        <td><input type="checkbox" value={ format!("{}", item.viewed) } onchange={change_viewed(item.id)}/></td>
                        <td>{ item.rating }</td>
                        <td>
                        <button onclick={update_rating(item.id, 1)}>{ "+" } </button>
                        <button onclick={update_rating(item.id, -1)}>{ "-" } </button>
                        <button onclick={delete(item.id)}>{ "🗑 " } </button>
                        </td>
                    </tr>
                }
            })}
        </table>
        <button onclick={add_musical}>{ "[+]" } </button>
        <a href={"/musicalist"}>{ "[Clear all]" }</a>
        <a href={"https://github.com/yazgoo/musicalist"}>{ "[about]" }</a>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
