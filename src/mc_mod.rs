use std::{path::PathBuf};
use std::path::Path;


use serde::{Serialize, Deserialize};

use crate::{core::{ModStatus, ModLoader, ModRepository}};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftMod {
    pub status: ModStatus,
    pub corresponding_file: Option<PathBuf>,
    pub mod_identifier: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModDirectory {
    pub game_version: String,
    pub mod_loader: ModLoader,
    pub mod_repository: ModRepository,

    pub mods: Vec<MinecraftMod>,
}

impl ModDirectory {
    pub fn save(&self, path: &Path) {
        let mut me = self.clone();
        me.mods.retain(|m| m.status != ModStatus::Bad);

        let text = serde_json::to_string_pretty(&me)
            .expect("To turn into json");

        let mut path = path.to_path_buf();
        path.push("mcmd.json");

        let _ = std::fs::write(path, text);
    }
}