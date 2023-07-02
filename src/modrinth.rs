use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_json::Value;

pub static API_URL: &str = "https://api.modrinth.com/v2/";

#[derive(Serialize, Deserialize, Debug)]
pub struct ModrinthMod {
    pub categories: Vec<String>,
    pub display_categories: Vec<String>,
    pub title: String,
    pub latest_version: String,
    pub versions: Vec<String>,
    pub project_type: String,
    pub project_id: String,

    #[serde(flatten)]
    other: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModrinthResponse {
    pub hits: Vec<ModrinthMod>,

    #[serde(flatten)]
    other: HashMap<String, Value>,
}

pub async fn search_mods(name: &str, version: &str) -> Result<ModrinthResponse, ()> {
    let client = crate::core::client();
    let url = reqwest::Url::parse_with_params(
        &(API_URL.to_string() + "search"),
        &[("query", name), ("limit", "20"), ("categories", "mod"), ("facets", format!("[[\"versions:{version}\"]]").as_str())]
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