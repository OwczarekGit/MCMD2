use std::{path::PathBuf};


use serde::{Serialize, Deserialize};

use crate::{core::{ModStatus, ModLoader, ModRepository}};

#[derive(Serialize, Deserialize, Debug)]
pub struct MinecraftMod {
    pub status: ModStatus,
    pub coresponding_file: Option<PathBuf>,
    pub mod_identifier: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModDirectory {
    pub game_version: String,
    pub mod_loader: ModLoader,
    pub mod_repository: ModRepository,

    pub mods: Vec<MinecraftMod>,
}