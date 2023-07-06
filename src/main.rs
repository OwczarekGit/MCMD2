
use crate::core::{fit_string};
use crate::core::ModStatus;

use crate::core::Repository;
use crate::core::Preferences;
use crate::core::ModLoader;
use std::io::stdout;
use std::path::PathBuf;


use clap::Parser;
use crossterm::style::SetForegroundColor;
use crossterm::{terminal::{enable_raw_mode, disable_raw_mode, Clear}, queue};
use mc_mod::{ModDirectory, MinecraftMod};

mod core;
mod mc_mod;
mod search_field;
mod display;
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

    let mod_directory: ModDirectory = serde_json::from_str(&text).unwrap();
    let _ = enable_raw_mode();
    let display = display::Display::new(mod_directory, prefs2.path);
    display.unwrap().process_events().await;
    let _ = disable_raw_mode();
    let _ = queue!(stdout(), Clear(crossterm::terminal::ClearType::All));

    Ok(())
}

pub struct PanelEntry {
    data: MinecraftMod,
}

impl PanelEntry {
    pub fn new(data: MinecraftMod) -> Self {
        Self {
            data
        }
    }
    
    pub fn draw(&self, x: u16, y: u16, max_width: u16) {
        use crossterm::cursor::{MoveTo};
        use crossterm::style::{Print};
        let _ = queue!(stdout(), MoveTo(x,y), Print(fit_string(&self.data.name, (max_width-2).into())));
    }
}

pub struct Panel {
    pub width: u16,
    pub height: u16,
    pub panel_entries: Vec<PanelEntry>,
    pub selection: usize,
}

impl Panel {
    pub fn new(x: u16, y: u16) -> Self {
        Self { width: x, height: y, panel_entries: vec![], selection: 0 }
    }

    pub async fn download_all(&mut self, repository: &Box<dyn Repository>, mod_version: &str, loader: &ModLoader, location: &PathBuf) {
        for entry in self.panel_entries.iter_mut() {
            match repository.download_mod(&entry.data.mod_identifier, mod_version, loader, location).await {
                core::DownloadStatus::Error => entry.data.status = ModStatus::CanUpdate,
                core::DownloadStatus::Success(filename) => {
                    entry.data.status = ModStatus::UpToDate;
                    entry.data.coresponding_file = Some(filename.into())
                },
                core::DownloadStatus::FileExists => entry.data.status = ModStatus::CanUpdate,
            };
        }
    }

    pub fn open_selected(&self, repository: &Box<dyn Repository>) {
        if let Some(selected) = self.panel_entries.get(self.selection) {
            repository.open(&selected.data.mod_identifier);
        }
    }

    pub fn delete_selection(&mut self) {
        if let Some(selection) = self.panel_entries.get(self.selection) {
            match selection.data.status {
                core::ModStatus::UpToDate => todo!(),
                core::ModStatus::CanUpdate => todo!(),
                core::ModStatus::Removed => todo!(),
                core::ModStatus::Normal => self.panel_entries.remove(self.selection),
            };
                
        }
        self.fix_selection();
    }

    pub fn get_focused(&self) -> Option<&PanelEntry> {
        self.panel_entries.get(self.selection)
    }

    pub fn increase_selection(&mut self) {
        self.selection = (self.selection + 1).clamp(0, (self.panel_entries.len() as isize - 1) as usize);
    }

    pub fn decrease_selection(&mut self) {
        self.selection = self.selection.saturating_sub(1)
    }

    pub fn focus_first(&mut self) {
        self.selection = 0;
    }

    pub fn focus_last(&mut self) {
        self.selection = (self.panel_entries.len() as isize - 1).max(0) as usize;
    }

    pub fn fix_selection(&mut self) {
        self.selection = self.selection.clamp(0, (self.panel_entries.len() as isize - 1) as usize);
    }

    pub fn resize(&mut self, x: u16, y: u16) {
        self.width = x;
        self.height = y;
    }

    pub fn draw_character(&self, x: u16, y: u16, c: impl std::fmt::Display) {
        use crossterm::cursor::{MoveTo};
        use crossterm::style::{Print};

        let _ = queue!(stdout(), MoveTo(x,y), Print(c));
    }

    pub fn move_to_panel(&mut self, other_panel: &mut Panel) {
        if self.panel_entries.get(self.selection).is_some() {
            let entry = self.panel_entries.remove(self.selection);
            other_panel.panel_entries.push(entry);
        }
        self.fix_selection();
    }

    pub fn draw_entries(&self, xoff: u16) {
        use crossterm::style::{SetBackgroundColor};
        let mut stdout = stdout();
        for (i, entry) in self.panel_entries.iter().enumerate() {

            let text_color = match entry.data.status {
                ModStatus::Normal => crossterm::style::Color::Reset,
                ModStatus::UpToDate => crossterm::style::Color::Green,
                ModStatus::CanUpdate => crossterm::style::Color::Cyan,
                ModStatus::Removed => crossterm::style::Color::Red,
            };

            let _ = queue!(stdout, SetForegroundColor(text_color));

            if self.selection.eq(&i) {
                let _ = queue!(stdout, SetBackgroundColor(crossterm::style::Color::DarkBlue));
                entry.draw(xoff, i as u16 + 1, self.width);
            } else {
                let _ = queue!(stdout, SetBackgroundColor(crossterm::style::Color::Reset));
                entry.draw(xoff, i as u16 + 1, self.width);
            }
        }
        let _ = queue!(stdout, SetBackgroundColor(crossterm::style::Color::Reset), SetForegroundColor(crossterm::style::Color::Reset));
    }

    pub fn draw_frame(&self, offset_x: u16, offset_y: u16, bold: bool) {
        if bold {
            for y in offset_y..offset_y+self.height {
                for x in offset_x..offset_x+self.width {
                    match (x, y) {
                        (ox, 0) if ox == offset_x => self.draw_character(x, y, ui_chars::TL_BOLD),
                        (ox, 0) if ox == offset_x+self.width-1 => self.draw_character(x, y, ui_chars::TR_BOLD),
                        (ox, oy) if ox == offset_x && oy == offset_y+self.height-1 => self.draw_character(x, y, ui_chars::BL_BOLD),
                        (ox, oy) if ox == offset_x+self.width-1 && oy == offset_y+self.height-1 => self.draw_character(x, y, ui_chars::BR_BOLD),
                        (ox, _) if ox == offset_x => self.draw_character(x, y, ui_chars::V_LINE_BOLD),
                        (ox, _) if ox == offset_x+self.width-1 => self.draw_character(x, y, ui_chars::V_LINE_BOLD),
                        (_, oy) if oy == offset_y => self.draw_character(x, y, ui_chars::H_LINE_BOLD),
                        (_, oy) if oy == offset_y+self.height-1 => self.draw_character(x, y, ui_chars::H_LINE_BOLD),
                        (_, _) => self.draw_character(x, y, " "),
                    };
                }
            }
        } else {
            for y in offset_y..offset_y+self.height {
                for x in offset_x..offset_x+self.width {
                    match (x, y) {
                        (ox, 0) if ox == offset_x => self.draw_character(x, y, ui_chars::TL),
                        (ox, 0) if ox == offset_x+self.width-1 => self.draw_character(x, y, ui_chars::TR),
                        (ox, oy) if ox == offset_x && oy == offset_y+self.height-1 => self.draw_character(x, y, ui_chars::BL),
                        (ox, oy) if ox == offset_x+self.width-1 && oy == offset_y+self.height-1 => self.draw_character(x, y, ui_chars::BR),
                        (ox, _) if ox == offset_x => self.draw_character(x, y, ui_chars::V_LINE),
                        (ox, _) if ox == offset_x+self.width-1 => self.draw_character(x, y, ui_chars::V_LINE),
                        (_, oy) if oy == offset_y => self.draw_character(x, y, ui_chars::H_LINE),
                        (_, oy) if oy == offset_y+self.height-1 => self.draw_character(x, y, ui_chars::H_LINE),
                        (_, _) => self.draw_character(x, y, " "),
                    };
                }
            }
        }
    }
}

pub mod ui_chars {
    pub static H_LINE: char = '─';
    pub static H_LINE_BOLD: char = '━';

    pub static V_LINE: char = '│';
    pub static V_LINE_BOLD: char = '┃';

    pub static TL: char = '┌';
    pub static TL_BOLD: char = '┏';

    pub static TR: char = '┐';
    pub static TR_BOLD: char = '┓';

    pub static BL: char = '└';
    pub static BL_BOLD: char = '┗';

    pub static BR: char = '┘';
    pub static BR_BOLD: char = '┛';
}