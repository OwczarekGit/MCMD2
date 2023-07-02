
use std::{io::{Write, stdout}, f32::consts::E, ops::Deref};

use crossterm::{terminal::{enable_raw_mode, disable_raw_mode, Clear}, queue, cursor::{DisableBlinking, Hide}, style::SetForegroundColor};

fn main() -> Result<(),()> {
   enable_raw_mode();
   let mut display = Display::new()?;
   display.process_events();
   disable_raw_mode(); 
   queue!(stdout(), Clear(crossterm::terminal::ClearType::All));

   Ok(())
}

enum KeyAction {
    Quit,
    FocusUp,
    FocusDown,
    MoveLeft,
    MoveRight,
    None,
}

pub struct Display {
    pub width: u16,
    pub height: u16,
    pub left: Panel,
    pub right: Panel,
    pub focused_col: u8,
}

impl Display {
    pub fn new() -> Result<Self, ()> {
        let size = crossterm::terminal::size().map_err(|_| ())?;
        Ok(Self {
            width: size.0,
            height: size.1,
            left: Panel { width: size.0/2, height: size.1, panel_entries: vec![
                PanelEntry::new("Hello"),
                PanelEntry::new("World!"),
                PanelEntry::new("Some Long text that is likely to not fit within given range."),
            ], selection: 0 },
            right: Panel { width: size.0/2, height: size.1, panel_entries: vec![], selection: 0 },
            focused_col: 0
        })
    }

    pub fn process_events(&mut self) {
        self.redraw();
        'event_loop: loop {
            match crossterm::event::read().unwrap() {
                crossterm::event::Event::Key(key) => {
                    match key.code {
                        // crossterm::event::KeyCode::Backspace => todo!(),
                        // crossterm::event::KeyCode::Enter => todo!(),
                        crossterm::event::KeyCode::Left => self.move_entry_right(),
                        crossterm::event::KeyCode::Right => self.move_entry_left(),
                        crossterm::event::KeyCode::Up => self.focus_prev(),
                        crossterm::event::KeyCode::Down => self.focus_next(),
                        // crossterm::event::KeyCode::Home => todo!(),
                        // crossterm::event::KeyCode::End => todo!(),
                        // crossterm::event::KeyCode::PageUp => todo!(),
                        // crossterm::event::KeyCode::PageDown => todo!(),
                        crossterm::event::KeyCode::Tab => self.swap_column(),
                        // crossterm::event::KeyCode::BackTab => todo!(),
                        // crossterm::event::KeyCode::Delete => todo!(),
                        // crossterm::event::KeyCode::Insert => todo!(),
                        // crossterm::event::KeyCode::F(_) => todo!(),
                        crossterm::event::KeyCode::Char(c) => match self.handle_key(c) {
                            KeyAction::FocusUp   => self.focus_prev(),
                            KeyAction::FocusDown => self.focus_next(),
                            KeyAction::MoveLeft  => self.move_entry_right(),
                            KeyAction::MoveRight => self.move_entry_left(),
                            KeyAction::Quit => break 'event_loop,
                            KeyAction::None => {}
                        },
                        crossterm::event::KeyCode::Esc => break 'event_loop,
                        // crossterm::event::KeyCode::CapsLock => todo!(),
                        // crossterm::event::KeyCode::ScrollLock => todo!(),
                        // crossterm::event::KeyCode::NumLock => todo!(),
                        // crossterm::event::KeyCode::PrintScreen => todo!(),
                        // crossterm::event::KeyCode::Pause => todo!(),
                        // crossterm::event::KeyCode::Menu => todo!(),
                        // crossterm::event::KeyCode::KeypadBegin => todo!(),
                        // crossterm::event::KeyCode::Media(_) => todo!(),
                        // crossterm::event::KeyCode::Modifier(_) => todo!(),
                        _ => todo!()
                    }
                },
                // crossterm::event::Event::Mouse(_) => todo!(),
                // crossterm::event::Event::Paste(_) => todo!(),
                crossterm::event::Event::Resize(x, y) => self.update_size(x, y),
                _ => {},
            }

            self.redraw();
        }
    }

    fn handle_key(&mut self, key: char) -> KeyAction {
        match key {
            'q' => KeyAction::Quit,
            'j' => KeyAction::FocusDown,
            'k' => KeyAction::FocusUp,
            'h' => KeyAction::MoveLeft,
            'l' => KeyAction::MoveRight,
            _ => KeyAction::None,
        }
    }

    fn move_entry_left(&mut self) {
        match self.focused_col {
            0 => self.left.move_to_panel(&mut self.right),
            _ => {},
        }
    }
    
    fn move_entry_right(&mut self) {
        match self.focused_col {
            1 => self.right.move_to_panel(&mut self.left),
            _ => {},
        }
    }

    fn focus_next(&mut self) {
        match self.focused_col {
            0 => self.left.increase_selection(),
            1 => self.right.increase_selection(),
            _ => {},
        }
    }
    
    fn focus_prev(&mut self) {
        match self.focused_col {
            0 => self.left.decrease_selection(),
            1 => self.right.decrease_selection(),
            _ => {},
        }
    }

    fn swap_column(&mut self) {
        if self.focused_col == 0 {
            self.focused_col = 1;
        } else {
            self.focused_col = 0;
        }
    }

    pub fn update_size(&mut self, x: u16, y: u16) {
        self.width = x;
        self.height = y;

        self.left.width = self.width / 2;
        self.left.height = self.height;

        self.right.width = self.width / 2;
        self.right.height = self.width;
    }

    pub fn redraw(&self) {
        self.clear_screen();
        self.draw_ui();
        stdout().flush();
    }

    fn draw_ui(&self) {
        self.left.draw_frame(0, 0, self.focused_col == 0);
        self.right.draw_frame(self.width/2, 0, self.focused_col == 1);

        self.left.draw_entries(1);
        self.right.draw_entries(self.width/2+1);
    }

    fn clear_screen(&self) {
        use crossterm::cursor::{MoveTo};
        use crossterm::style::{Print};
        let mut stdout = stdout();

        for y in 0..self.height {
            for x in 0..self.width {
                queue!(stdout, DisableBlinking, Hide, MoveTo(x,y), Print(' '));
            }
        }
    }
}

pub struct PanelEntry {
    display_value: Box<dyn std::fmt::Display>,
}

impl PanelEntry {
    pub fn new(display_value: impl std::fmt::Display + 'static) -> Self {
        Self {
            display_value: Box::new(display_value)
        }
    }
    
    pub fn draw(&self, x: u16, y: u16, max_width: u16) {
        let length = self.display_value.to_string().len();
        let display_string_length = (max_width-2).min(length as u16);
        let chars = self.display_value.to_string().chars().collect::<Vec<char>>();
        let display_string = &chars[0..display_string_length as usize].iter().collect::<String>();

        let mut final_String = String::from_utf8(vec![b' '; (max_width-2) as usize]).unwrap();
        let fin_string = display_string[..display_string.len()].to_owned() + &final_String[display_string.len()..];

        use crossterm::cursor::{MoveTo};
        use crossterm::style::{Print};

        let _ = queue!(stdout(), MoveTo(x,y), Print(fin_string));

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

    pub fn increase_selection(&mut self) {
        self.selection = (self.selection + 1).clamp(0, (self.panel_entries.len() as isize - 1) as usize);
    }

    pub fn decrease_selection(&mut self) {
        self.selection = self.selection.saturating_sub(1)
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
        if let Some(_) = self.panel_entries.get(self.selection) {
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
                queue!(stdout, SetBackgroundColor(crossterm::style::Color::DarkBlue));
                entry.draw(xoff, i as u16 + 1, self.width);
            } else {
                queue!(stdout, SetBackgroundColor(crossterm::style::Color::Reset));
                entry.draw(xoff, i as u16 + 1, self.width);
            }
        }
        queue!(stdout, SetBackgroundColor(crossterm::style::Color::Reset));
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