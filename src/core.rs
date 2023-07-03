use std::{sync::OnceLock};
use reqwest::Client;

pub static USER_AGENT: OnceLock<String> = OnceLock::new();

pub fn init() {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    USER_AGENT.get_or_init(|| format!("OwczarekGit/{}/{}", name, version));
}

pub fn client() -> Client {
    reqwest::ClientBuilder::new()
        .user_agent(USER_AGENT.get().expect("User agent to be set at this point."))
        .build().expect("Client to be created.")
}

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
    Open,
    Clear,
    None,
}

pub enum ApplicationMode {
    Search,
    Normal,
}

#[derive(Clone, Copy)]
pub enum ModLoader {
    Forge,
    Fabric,
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

#[derive(Clone)]
pub struct Preferences {
    pub version: String,
    pub mod_loader: ModLoader,
}

impl Preferences {
    pub fn new(version: String, mod_loader: ModLoader) -> Self {
        Self { version, mod_loader }
    }
}