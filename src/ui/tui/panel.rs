use std::io::stdout;
use std::path::PathBuf;
use crossterm::cursor::MoveTo;
use crossterm::queue;
use crate::core::{DownloadStatus, ModLoader, ModStatus, Repository};
use crossterm::style::{SetForegroundColor, SetBackgroundColor, Print};
use crate::ui::tui::glyphs;
use crate::ui::tui::panel_entry::PanelEntry;

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
                DownloadStatus::Error => entry.data.status = ModStatus::Bad,
                DownloadStatus::Success(filename) => {
                    entry.data.status = ModStatus::Ok;
                    entry.data.corresponding_file = Some(filename.into())
                },
                DownloadStatus::FileExists => entry.data.status = ModStatus::Ok,
            };
        }
    }

    pub async fn open_selected(&self, repository: &Box<dyn Repository>) {
        if let Some(selected) = self.panel_entries.get(self.selection) {
            repository.open(&selected.data.mod_identifier).await;
        }
    }

    pub fn delete_selection(&mut self) {
        if let Some(selection) = self.panel_entries.get(self.selection) {
            match selection.data.status {
                ModStatus::Ok => todo!(),
                ModStatus::CanUpdate => todo!(),
                ModStatus::Bad => todo!(),
                ModStatus::Normal => self.panel_entries.remove(self.selection),
                ModStatus::Missing => todo!(),
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
        let _ = queue!(stdout(), MoveTo(x,y), Print(c));
    }

    pub fn move_to_panel(&mut self, other_panel: &mut Panel) {
        if self.panel_entries.get(self.selection).is_some() {
            let entry = self.panel_entries.remove(self.selection);
            other_panel.panel_entries.push(entry);
        }
        self.fix_selection();
    }

    pub fn draw_entries(&self, x_off: u16) {
        let mut stdout = stdout();
        for (i, entry) in self.panel_entries.iter().enumerate() {

            let text_color = match entry.data.status {
                ModStatus::Normal => crossterm::style::Color::Reset,
                ModStatus::Ok => crossterm::style::Color::Green,
                ModStatus::CanUpdate => crossterm::style::Color::Cyan,
                ModStatus::Bad => crossterm::style::Color::Red,
                ModStatus::Missing => crossterm::style::Color::Yellow,
            };

            let _ = queue!(stdout, SetForegroundColor(text_color));

            if self.selection.eq(&i) {
                let _ = queue!(stdout, SetBackgroundColor(crossterm::style::Color::DarkBlue));
                entry.draw(x_off, i as u16 + 1, self.width);
            } else {
                let _ = queue!(stdout, SetBackgroundColor(crossterm::style::Color::Reset));
                entry.draw(x_off, i as u16 + 1, self.width);
            }
        }
        let _ = queue!(stdout, SetBackgroundColor(crossterm::style::Color::Reset), SetForegroundColor(crossterm::style::Color::Reset));
    }

    pub fn draw_frame(&self, offset_x: u16, offset_y: u16, bold: bool) {
        if bold {
            for y in offset_y..offset_y+self.height {
                for x in offset_x..offset_x+self.width {
                    match (x, y) {
                        (ox, 0) if ox == offset_x => self.draw_character(x, y, glyphs::TL_BOLD),
                        (ox, 0) if ox == offset_x+self.width-1 => self.draw_character(x, y, glyphs::TR_BOLD),
                        (ox, oy) if ox == offset_x && oy == offset_y+self.height-1 => self.draw_character(x, y, glyphs::BL_BOLD),
                        (ox, oy) if ox == offset_x+self.width-1 && oy == offset_y+self.height-1 => self.draw_character(x, y, glyphs::BR_BOLD),
                        (ox, _) if ox == offset_x => self.draw_character(x, y, glyphs::V_LINE_BOLD),
                        (ox, _) if ox == offset_x+self.width-1 => self.draw_character(x, y, glyphs::V_LINE_BOLD),
                        (_, oy) if oy == offset_y => self.draw_character(x, y, glyphs::H_LINE_BOLD),
                        (_, oy) if oy == offset_y+self.height-1 => self.draw_character(x, y, glyphs::H_LINE_BOLD),
                        (_, _) => self.draw_character(x, y, " "),
                    };
                }
            }
        } else {
            for y in offset_y..offset_y+self.height {
                for x in offset_x..offset_x+self.width {
                    match (x, y) {
                        (ox, 0) if ox == offset_x => self.draw_character(x, y, glyphs::TL),
                        (ox, 0) if ox == offset_x+self.width-1 => self.draw_character(x, y, glyphs::TR),
                        (ox, oy) if ox == offset_x && oy == offset_y+self.height-1 => self.draw_character(x, y, glyphs::BL),
                        (ox, oy) if ox == offset_x+self.width-1 && oy == offset_y+self.height-1 => self.draw_character(x, y, glyphs::BR),
                        (ox, _) if ox == offset_x => self.draw_character(x, y, glyphs::V_LINE),
                        (ox, _) if ox == offset_x+self.width-1 => self.draw_character(x, y, glyphs::V_LINE),
                        (_, oy) if oy == offset_y => self.draw_character(x, y, glyphs::H_LINE),
                        (_, oy) if oy == offset_y+self.height-1 => self.draw_character(x, y, glyphs::H_LINE),
                        (_, _) => self.draw_character(x, y, " "),
                    };
                }
            }
        }
    }
}
