use crate::core::Preferences;
use clap::Parser;
use mc_mod::{ModDirectory};
use ui::tui::display;

mod core;
mod ui;
mod curseforge;
mod mc_mod;
mod search_field;
mod modrinth;

#[tokio::main]
async fn main() -> Result<(), String>{
    core::init();
    let mut prefs = Preferences::parse();
    let prefs2 = prefs.clone();
    prefs.path.push("mcmd.json");

    let text = match std::fs::read_to_string(prefs.path) {
        Ok(text) => text,
        Err(_) => {
            if prefs.mod_loader.is_none() || prefs.version.is_none() || prefs.mod_repository.is_none() {
                return Err("This seams to be a new mod directory. Please provide mod loader, game version and repository. ".to_owned());
            } else {
                let md = ModDirectory {
                game_version: prefs.version.unwrap(),
                mod_loader: prefs.mod_loader.unwrap(),
                mod_repository: prefs.mod_repository.unwrap(),
                mods: vec![]
             };
             serde_json::to_string(&md).unwrap()
         }
      }
    };

    let mut mod_directory: ModDirectory = serde_json::from_str(&text).unwrap();
    mod_directory.verify(&prefs2.path);
    let mut display = display::Display::new(mod_directory, prefs2.path).unwrap();

    display.process_events().await;

    Ok(())
}