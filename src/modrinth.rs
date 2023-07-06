use std::{collections::{HashMap}, fmt::Display, vec};

use serde::{Serialize, Deserialize};
use serde_json::Value;
use async_trait::async_trait;

use crate::{core::{ModLoader, Url, client, Status, ModStatus, DownloadStatus, Repository}, mc_mod::MinecraftMod};

pub static API_URL: &str = "https://api.modrinth.com/v2/";

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ModrinthRepository {
    pub mods: HashMap<String, ModrinthMod>,
}

#[async_trait]
impl Repository for ModrinthRepository {

    async fn search_mods(&self, name: &str, version: &str, mod_loader: ModLoader) -> Vec<MinecraftMod> {
        let client = crate::core::client();
        let Ok(url) = reqwest::Url::parse_with_params(
            &(API_URL.to_string() + "search"),
            &[
                ("query", name),
                ("limit", "20"),
                ("facets", format!("[[\"project_type:mod\"],[\"versions:{version}\"],[\"categories:{}\"]]", String::from(mod_loader)).as_str())
            ]
        ) else {
            return vec![];
        };

        let request = client.get(url).build().unwrap();
        let mods: ModrinthResponse = client.execute(request)
            .await
            .unwrap()
            .json()
            .await
            .expect("The mods to be parsed correctly.");

        let mods = mods.hits;

        mods.iter().map(|m|
            MinecraftMod {
                coresponding_file: None,
                mod_identifier: m.project_id.clone(),
                name: m.title.clone(),
                status: ModStatus::Normal,
            }
        ).collect()
    }

    async fn download_mod(&self, _mod_identifier: &str) -> DownloadStatus {
        DownloadStatus::Success
    }

    fn open(&self, _mod_identifier: &str) {

    }

    fn url(&self, _mod_identifier: &str) -> String {
        String::new()
    }
}

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

    #[serde(default)]
    pub status: ModStatus,

    #[serde(flatten)]
    other: HashMap<String, Value>,
}

impl ModrinthMod {
    pub async fn get_releases(&self) -> Vec<ModrinthReleases> {
        let clinet = client();
        let url = &format!("https://api.modrinth.com/v2/project/{}/version", self.slug);
        let url = reqwest::Url::parse(url).expect("To parse correctly.");
        let request = clinet.get(url).build().unwrap();

        let response = clinet.execute(request)
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        response
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModrinthReleases {
    pub id: String,
    pub project_id: String,
    pub files: Vec<ModrinthFile>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,

    #[serde(flatten)]
    others: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModrinthFile {
    pub url: String,
    pub filename: String,

    #[serde(flatten)]
    others: HashMap<String, Value>,
}

impl Display for ModrinthMod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModrinthResponse {
    pub hits: Vec<ModrinthMod>,

    #[serde(flatten)]
    other: HashMap<String, Value>,
}
