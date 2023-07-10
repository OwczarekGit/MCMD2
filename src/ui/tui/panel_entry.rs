use crossterm::cursor::{MoveTo};
use crossterm::style::{Print};
use crossterm::queue;
use crate::mc_mod::MinecraftMod;
use crate::core::fit_string;
use std::io::stdout;

pub struct PanelEntry {
    pub data: MinecraftMod,
}

impl PanelEntry {
    pub fn new(data: MinecraftMod) -> Self {
        Self {
            data
        }
    }

    pub fn draw(&self, x: u16, y: u16, max_width: u16) {
        let _ = queue!(stdout(), MoveTo(x,y), Print(fit_string(&self.data.name, (max_width-2).into())));
    }
}
