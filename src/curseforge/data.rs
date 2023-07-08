use std::{collections::HashMap};
use clap::ValueEnum;

use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::core::ModLoader;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeResponse {
    pub data: Vec<CurseforgeMod>,

    #[serde(flatten)]
    other: HashMap<String, Value>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeMod {
    pub id: u32,
    pub name: String,
    pub links: ModLink,
    pub latest_files: Vec<LatestFile>,


    #[serde(flatten)]
    other: HashMap<String, Value>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModLink {
    pub website_url: Option<String>,

    #[serde(flatten)]
    other: HashMap<String, Value>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LatestFile {
    pub id: u32,
    pub file_name: Option<String>,
    pub game_versions: Vec<String>,
    pub download_url: Option<String>,

    #[serde(flatten)]
    other: HashMap<String, Value>
}

#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq, PartialEq, ValueEnum)]
pub enum ModLoaderType {
    Any = 0,
    Forge = 1,
    Cauldron = 2,
    LiteLoader = 3,
    Fabric = 4,
    Quilt = 5,
}

impl From<ModLoader> for ModLoaderType {
    fn from(value: ModLoader) -> Self {
        match value {
            ModLoader::Forge => Self::Forge,
            ModLoader::Fabric => Self::Fabric,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CurseForgeGetModResponse {
    pub data: CurseforgeMod,

    #[serde(flatten)]
    other: HashMap<String, Value>
}