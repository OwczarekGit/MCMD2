use std::{collections::{HashMap}, fmt::Display};

use serde::{Serialize, Deserialize};
use serde_json::Value;
use async_trait::async_trait;

use crate::core::{ModLoader, Url, Open, client, Download, Status, ModStatus, DownloadStatus, download_file};

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

impl ModrinthReleases {
    pub fn fits_version_and_loader(&self, version: &str, loader: ModLoader) -> bool {
        self.game_versions.iter().find(|ver| *ver == version).is_some() && self.loaders.iter().find(|l| *l == &String::from(loader)).is_some()
    }
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

#[async_trait]
impl Download for ModrinthMod {
    type Output = Result<DownloadStatus, ()>;

    async fn download(&mut self) -> Self::Output {

        if self.status() == crate::core::ModStatus::UpToDate {
            return Ok(DownloadStatus::Success);
        }

        let releases = self.get_releases().await;
        let release = releases.iter()
            .find(|release| release.fits_version_and_loader("1.20.1", ModLoader::Fabric));
    
        let release = release.ok_or(())?;
        let file = release.files.first().ok_or(())?;
        let filename = file.filename.clone();
        let url = file.url.clone();

    
        Ok(download_file(&url, &filename).await)
    }
}

impl Status for ModrinthMod {
    fn status(&self) -> crate::core::ModStatus {
        self.status
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