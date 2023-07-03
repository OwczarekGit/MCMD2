
use crate::core::{Download, Status};
use crate::core::ModStatus;
use std::{io::{stdout}, fmt::Display};

use crossterm::{terminal::{enable_raw_mode, disable_raw_mode, Clear}, queue};
use modrinth::{ModrinthMod};
use crate::core::{Url, Open};


mod core;
mod search_field;
mod display;
mod modrinth;

#[tokio::main]
async fn main() -> Result<(),()> {
    core::init();

    let _ = enable_raw_mode();
    let prefs = core::Preferences::new("1.20.1".to_owned(), core::ModLoader::Fabric);

    let mut display = display::Display::new(prefs).unwrap();
    display.process_events().await;
    let _ = disable_raw_mode();
    let _ = queue!(stdout(), Clear(crossterm::terminal::ClearType::All));
    Ok(())
}

pub struct PanelEntry<T> where T: Display + Open + Url + Download + Status {
    data: T
}

impl PanelEntry<ModrinthMod> {
    pub fn new(data: ModrinthMod) -> Self {
        Self {
            data
        }
    }
    
    pub fn draw(&self, x: u16, y: u16, max_width: u16) {
        let length = self.data.title.len();
        let display_string_length = (max_width-2).min(length as u16);
        let chars = self.data.title.chars().collect::<Vec<char>>();
        let display_string = &chars[0..display_string_length as usize].iter().collect::<String>();

        let final_string = String::from_utf8(vec![b' '; (max_width-2) as usize]).unwrap();
        let final_string = display_string[..display_string.len()].to_owned() + &final_string[display_string.len()..];

        use crossterm::cursor::{MoveTo};
        use crossterm::style::{Print};

        let _ = queue!(stdout(), MoveTo(x,y), Print(final_string));

    }
}

pub struct Panel {
    pub width: u16,
    pub height: u16,
    pub panel_entries: Vec<PanelEntry<ModrinthMod>>,
    pub selection: usize,
}

impl Panel {
    pub fn new(x: u16, y: u16) -> Self {
        Self { width: x, height: y, panel_entries: vec![], selection: 0 }
    }

    pub async fn download_all(&mut self) {
        for entry in self.panel_entries.iter_mut() {
            if let Ok(status) = entry.data.download().await {
                entry.data.status = ModStatus::UpToDate;
            }
        }
    }

    pub fn open_selected(&self) {
        if let Some(selected) = self.panel_entries.get(self.selection) {
            selected.data.open();
        }
    }

    pub fn delete_selection(&mut self) {
        if let Some(selection) = self.panel_entries.get(self.selection) {
            match selection.data.status() {
                core::ModStatus::UpToDate => todo!(),
                core::ModStatus::CanUpdate => todo!(),
                core::ModStatus::Removed => todo!(),
                core::ModStatus::Normal => self.panel_entries.remove(self.selection),
            };
                
        }
        self.fix_selection();
    }

    pub fn get_focused(&self) -> Option<&PanelEntry<ModrinthMod>> {
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
            if self.selection.eq(&i) {
                let _ = queue!(stdout, SetBackgroundColor(crossterm::style::Color::DarkBlue));
                entry.draw(xoff, i as u16 + 1, self.width);
            } else {
                let _ = queue!(stdout, SetBackgroundColor(crossterm::style::Color::Reset));
                entry.draw(xoff, i as u16 + 1, self.width);
            }
        }
        let _ = queue!(stdout, SetBackgroundColor(crossterm::style::Color::Reset));
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