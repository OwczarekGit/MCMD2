use std::{collections::HashMap, fmt::Display};

use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::core::{ModLoader, Url, Open};

pub static API_URL: &str = "https://api.modrinth.com/v2/";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModrinthMod {
    pub project_id: String,
    pub project_type: String,
    pub categories: Vec<String>,
    pub display_categories: Vec<String>,
    pub title: String,
    pub latest_version: String,
    pub versions: Vec<String>,
    pub slug: String,

    #[serde(flatten)]
    other: HashMap<String, Value>,
}

impl Display for ModrinthMod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title)
    }
}

impl Url for ModrinthMod {
    fn url(&self) -> String {
        format!("https://modrinth.com/mod/{}", self.slug)
    }
}

impl Open for ModrinthMod {
    fn open(&self) {
        let _ = open::that(self.url());
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModrinthResponse {
    pub hits: Vec<ModrinthMod>,

    #[serde(flatten)]
    other: HashMap<String, Value>,
}

pub async fn search_mods(name: &str, version: &str, mod_loader: ModLoader) -> Result<ModrinthResponse, ()> {
    let client = crate::core::client();
    let url = reqwest::Url::parse_with_params(
        &(API_URL.to_string() + "search"),
        &[
            ("query", name),
            ("limit", "20"),
            ("facets", format!("[[\"project_type:mod\"],[\"versions:{version}\"],[\"categories:{}\"]]", String::from(mod_loader)).as_str())
            ]
    )
    .map_err(|_| ())?;

    let request = client.get(url).build().unwrap();
    client.execute(request)
        .await
        .unwrap()
        .json()
        .await
        .map_err(|_|())
}