#[derive(serde::Serialize, serde::Deserialize)]
struct Query {
    content: Option<String>,
    edit: Option<bool>,
    user: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Musical {
    pub id: u64,
    name: String,
    pub url: String,
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

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
struct Users {
    version: u8,
    items: Vec<String>,
}
