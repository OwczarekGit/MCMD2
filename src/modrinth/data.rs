use std::collections::HashMap;
use std::fmt::Display;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::{ModLoader, ModStatus};
use crate::mc_mod::MinecraftMod;

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
    pub mod_status: ModStatus,

    #[serde(flatten)]
    other: HashMap<String, Value>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModrinthProject {
    pub title: String,
    pub id: String,

    #[serde(flatten)]
    other: HashMap<String, Value>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModrinthReleases {
    pub id: String,
    pub project_id: String,
    pub files: Vec<ModrinthFile>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub dependencies: Vec<ModrinthDependency>,

    #[serde(flatten)]
    others: HashMap<String, Value>,
}

impl ModrinthReleases {
    pub fn fits_requirements(&self, version: &str, loader: ModLoader) -> bool {
        self.game_versions.iter().any(|v| *v == version)
            && self.loaders.iter().any(|l| *l == String::from(loader))

    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModrinthFile {
    pub url: String,
    pub filename: String,

    #[serde(flatten)]
    others: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModrinthDependency {
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub dependency_type: String,

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

impl From<ModrinthProject> for MinecraftMod {
    fn from(value: ModrinthProject) -> Self {
        MinecraftMod {
            status: ModStatus::Normal,
            name: value.title,
            mod_identifier: value.id,
            corresponding_file: None
        }
    }
}
