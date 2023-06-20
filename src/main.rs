mod model;
mod musicals;
use base64::{engine::general_purpose, Engine as _};
use web_sys::{HtmlInputElement, InputEvent};
use yew::prelude::*;
use yew_router::prelude::*;
include!("musicals.rs");

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

fn get_list_value(content: &str) -> MusicaList {
    let default_list = MusicaList {
        version: 1,
        author: "".to_string(),
        items: vec![],
    };
    general_purpose::STANDARD
        .decode(content)
        .map(|bytes| rmp_serde::from_read(&bytes[..]).unwrap_or(default_list.clone()))
        .unwrap_or(default_list)
}

fn get_url(list: &MusicaList) -> (String, String) {
    let val = MusicaList {
        version: list.version,
        author: list.author.clone(),
        items: list.items.clone(),
    };
    let str = rmp_serde::to_vec(&val).unwrap();
    // convert str to base64
    let str = general_purpose::STANDARD.encode(str);
    (format!("?content={}", str), str)
}

#[function_component(Home)]
fn home() -> Html {
    wasm_logger::init(wasm_logger::Config::default());
    let bookmark_url = use_state(|| "".to_string());

    let navigator = use_navigator().unwrap();

    let current_location = use_location().unwrap();

    let location = yew_hooks::use_location();

    let content = current_location
        .query::<Query>()
        .map_or("".to_string(), |query| query.content);

    let trigger = use_force_update();

    let edit = current_location
        .query::<Query>()
        .map_or(Some(false), |query| query.edit);

    let list_value: MusicaList = get_list_value(&content);

    let list = use_state(|| list_value.clone());

    macro_rules! update_list_fn {
        ($list:expr, $list_out:expr) => {{
            let bookmark_url = bookmark_url.clone();
            let navigator = navigator.clone();
            move |_| {
                let list_out = $list_out;
                let (new_url, content) = get_url(&list_out);
                bookmark_url.set(new_url);
                let _ = navigator.push_with_query(&Route::Home, &Query { content, edit });
                $list.set(list_out);
            }
        }};
    }

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

    let change_edit = {
        let navigator = navigator.clone();
        let list = list.clone();
        move |_| {
            let (_, content) = get_url(&list);
            let _ = navigator.push_with_query(
                &Route::Home,
                &Query {
                    content,
                    edit: Some(!(edit == Some(true))),
                },
            );
        }
    };

    let delete = |id| {
        let list = list.clone();
        update_list_fn!(
            list,
            MusicaList {
                version: list.version,
                author: list.author.clone(),
                items: list
                    .items
                    .iter()
                    .filter(|item| item.id != id)
                    .cloned()
                    .collect(),
            }
        )
    };

    let change_viewed = |id| {
        let list = list.clone();
        update_list_fn!(
            list,
            update_item_in_list(&list, id, |item| ListItem {
                viewed: !item.viewed,
                ..item.clone()
            })
        )
    };

    let move_item = |id: usize, delta: i8| {
        /* swap id and new_index in list */
        let list = list.clone();
        update_list_fn!(list, {
            let new_index = id as i8 + delta;
            let new_index = if new_index < 0 {
                0
            } else if new_index >= list.items.len() as i8 {
                list.items.len() as i8 - 1
            } else {
                new_index
            };
            let mut items = list.items.clone();
            items.swap(id as usize, new_index as usize);
            MusicaList {
                items,
                author: list.author.clone(),
                version: list.version,
            }
        })
    };

    let update_rating = |id: u64, delta: i8| {
        let list = list.clone();
        update_list_fn!(
            list,
            update_item_in_list(&list, id, |item| ListItem {
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
            })
        )
    };

    let add_musical = {
        let list = list.clone();
        update_list_fn!(list, {
            let mut items = list.items.clone();
            items.push(ListItem {
                id: list.items.len() as u64 + 1,
                musical_id: 1,
                viewed: false,
                rating: 0,
            });
            MusicaList {
                version: list.version,
                author: list.author.clone(),
                items,
            }
        })
    };

    let go = |i| {
        let navigator = navigator.clone();
        let list = list.clone();
        let trigger = trigger.clone();
        let current_location = current_location.clone();
        move |_| {
            navigator.go(i);
            trigger.force_update();
            let query = current_location.query::<Query>().unwrap();
            list.set(get_list_value(&query.content));
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
            let _ = navigator.push_with_query(&Route::Home, &Query { content, edit });
            list.set(list_out);
        }
    };

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
            let _ = navigator.push_with_query(&Route::Home, &Query { content, edit });
            list.set(list_out);
        })
    };

    fn get_musical_url(musical_id: u64) -> String {
        format!(
            "https://en.wikipedia.org/wiki/{}",
            musicals::MUSICALS
                .iter()
                .find(|m| m.id == musical_id)
                .map(|m| m.url.clone())
                .unwrap_or("".to_string())
        )
    }

    let mut i = 0;
    html! {
        <>
        if edit == Some(true) {
            { "Musicalist for " }
            <input type="text" value={ (*list).clone().author } oninput={update_author}/>
        } else {
            { (*list).clone().author }
            { "'s Musicalist" }
        }
        <br/>
        <br/>
        <table class={"center"}>
            <tr>
                <th>{ "Musical" }</th>
                <th>{ "Wiki" }</th>
                <th>{ "Viewed" }</th>
                <th>{ "Rating"}</th>
                if  edit == Some(true) {
                    <th>{ "actions" }</th>
                }
            </tr>
            { for (*list).clone().items.iter().map(|item| {
                                                              { i += 1; }
                html! {
                    <tr>
                        <td>
                        if i == (*list).items.len() && edit == Some(true) {
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
                        } else {
                            { MUSICALS.iter().find(|m| m.id == item.musical_id).map(|m| m.name.clone()).unwrap_or("".to_string()) }
                        }
                        </td>
                        <td>
                        <a href={get_musical_url(item.musical_id)}>{"?"}</a>
                        </td>
                        <td>
                        if edit == Some(true) {
                            <input type="checkbox" checked={ item.viewed } onchange={change_viewed(item.id)}/>
                        } else {
                            { if item.viewed { "👁" } else { "" } }
                        }
                        </td>
                        <td>{ item.rating }</td>
                        if edit == Some(true) {
                            <td>
                                <button title="increase rating" onclick={update_rating(item.id, 1)}>{ "➕" } </button>
                                { " " }
                                <button title="decrease rating" onclick={update_rating(item.id, -1)}>{ "➖" } </button>
                                { " " }
                                <button title="move up" onclick={move_item(i - 1, -1)}>{ "⬆" } </button>
                                { " " }
                                <button title="move down" onclick={move_item(i - 1, 1)}>{ "⬇" } </button>
                                { " " }
                                <button title="remove entry" onclick={delete(item.id)}>{ "🗑 " } </button>
                                </td>
                        }
                    </tr>
                }
            })}
        </table>
        <p>
        if edit == Some(true) {
            <button onclick={add_musical} title="add musical">{ "➕" } </button>
            { " " }
            <button onclick={go(-1)} title="undo">{ "🔙" } </button>
            { " " }
            <button onclick={go(1)} title="redo">{ "⏩" } </button>
            { " " }
        }
        <button onclick={change_edit} title={
            if edit == Some(true) {
                "switch to read-only mode"
            } else {
                "switch to edit mode"
            }
        }> {
            if edit == Some(true) {
                "👁 "
            } else {
                "🖊 "
            }
        } </button>
        </p>
        <p>
        <a href={"/musicalist"}>{ "Clear all" }</a>
        { " " }
        <a href={"https://github.com/yazgoo/musicalist"}>{ "about" }</a>
        { " " }
        <a href={ location.href.clone().replace("edit=true", "edit=false") }
        title={"Right click + copy link adress to get url"}>{ "sharing url" }</a>
        </p>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
