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
    Clear,
    None,
}

pub enum ApplicationMode {
    Search,
    Normal,
}

pub mod loaders {
    pub static FORGE:  &str = "forge";
    pub static FABRIC: &str = "fabric";
}