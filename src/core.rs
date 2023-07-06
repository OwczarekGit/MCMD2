use std::path::{PathBuf, Path};
use std::{sync::OnceLock};
use std::fs::{File, write};
use async_trait::async_trait;
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

use crate::mc_mod::MinecraftMod;


pub static USER_AGENT: OnceLock<String> = OnceLock::new();

pub fn init() {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    USER_AGENT.get_or_init(|| format!("OwczarekGit/{}/{}", name, version));
}

pub fn client() -> reqwest::Client {
    reqwest::ClientBuilder::new()
        .user_agent(USER_AGENT.get().expect("User agent to be set at this point."))
        .build().expect("Client to be created.")
}

pub async fn download_file(url: &str, filename: &str) -> DownloadStatus {
    if file_exists(filename) {
        return DownloadStatus::FileExists;
    }

    let Ok(response) = reqwest::get(url).await else {
        return DownloadStatus::Error;
    };

    let Ok(file_bytes) = response.bytes().await else {
        return DownloadStatus::Error;
    };
            
    write("./mods/".to_owned() + filename, &file_bytes).expect("The file to be saved.");

    DownloadStatus::Success
}

pub fn file_exists<P: AsRef<Path>>(filename: P) -> bool {
    File::open(filename).is_ok()
}

pub fn fit_string(text: &str, width: usize) -> String {
    let formatted = format!("{}", text);
    if formatted.len() <= width {
        format!("{:<width$}", formatted, width = width)
    } else {
        formatted.get(..width).unwrap().to_owned()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum KeyAction {
    Quit,
    FocusUp,
    FocusDown,
    MoveLeft,
    MoveRight,
    FocusFirst,
    FocusLast,
    Delete,
    StartSearchMode,
    Download,
    Open,
    Clear,
    None,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum DownloadStatus {
    Success,
    Error,
    FileExists,
}

#[derive(Clone, Copy, Debug)]
pub enum ApplicationMode {
    Search,
    Normal,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ModLoader {
    Forge,
    Fabric,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModStatus {
    Normal,
    UpToDate,
    CanUpdate,
    Removed,
}

impl Default for ModStatus {
    fn default() -> Self {
        Self::Normal
    }
}

impl From<ModLoader> for String {
    fn from(value: ModLoader) -> Self {
        match value {
            ModLoader::Forge => crate::core::loaders::FORGE.to_owned(),
            ModLoader::Fabric => crate::core::loaders::FABRIC.to_owned(),
        }
    }
}

pub mod loaders {
    pub static FORGE:  &str = "forge";
    pub static FABRIC: &str = "fabric";
}

pub trait Url {
    fn url(&self) -> String;
}

pub trait Open {
    fn open(&self);
}

pub trait Status {
    fn status(&self) -> ModStatus;
}

#[async_trait]
pub trait Download {
    async fn download(&mut self) -> DownloadStatus;
}

#[derive(Clone, Parser, Debug)]
pub struct Preferences {
    pub path: PathBuf,
    pub version: Option<String>,
    pub mod_loader: Option<ModLoader>,
    pub mod_repository: Option<ModRepository>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ModRepository {
    Modrinth,
    Curseforge,
}

#[async_trait]
pub trait Repository {
    async fn search_mods(&self, name: &str, version: &str, mod_loader: ModLoader) -> Vec<MinecraftMod>;
    async fn download_mod(&self, mod_identifier: &str) -> DownloadStatus;
    fn open(&self, mod_identifier: &str);
    fn url(&self, mod_identifier: &str) -> String;
}
