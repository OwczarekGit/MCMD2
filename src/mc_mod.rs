use std::{path::PathBuf};


use serde::{Serialize, Deserialize};

use crate::{core::{ModStatus, ModLoader, ModRepository}};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftMod {
    pub status: ModStatus,
    pub coresponding_file: Option<PathBuf>,
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
    pub fn save(&self, path: &PathBuf) {
        let text = serde_json::to_string_pretty(self)
            .expect("To turn into json");

        let mut path = path.clone();
        path.push("mcmd.json");

        let _ = std::fs::write(path, text);
    }
}