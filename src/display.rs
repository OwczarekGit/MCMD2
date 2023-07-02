use crate::{core::{ApplicationMode, KeyAction}, Panel, PanelEntry, modrinth};
use crossterm::{queue, cursor::{DisableBlinking, Hide}, event::KeyEvent};
use std::{io::{Write, stdout}};

pub struct Display {
    pub width: u16,
    pub height: u16,
    pub left: Panel,
    pub right: Panel,
    pub focused_col: u8,
    pub mode: ApplicationMode,
    pub should_close: bool,
    pub search_string: String,
}

impl Display {
    pub fn new() -> Result<Self, ()> {
        let size = crossterm::terminal::size().map_err(|_| ())?;
        Ok(Self {
            mode: ApplicationMode::Normal,
            should_close: false,
            search_string: String::new(),
            width: size.0,
            height: size.1,
            left: Panel::new(size.0/2, size.1),
            right: Panel::new(size.0/2, size.1),
            focused_col: 0
        })
    }

    pub async fn process_events(&mut self) {
        'event_loop: loop {
            self.redraw();

            match crossterm::event::read().unwrap() {
                crossterm::event::Event::Key(key) => {
                    match self.mode {
                        ApplicationMode::Normal => self.handle_key_normal_mode(key),
                        ApplicationMode::Search => self.handle_key_search_mode(key).await,
                    }
                },
                // crossterm::event::Event::Mouse(_) => todo!(),
                // crossterm::event::Event::Paste(_) => todo!(),
                crossterm::event::Event::Resize(x, y) => self.update_size(x, y),
                _ => {},
            }

            if self.should_close {
                break 'event_loop;
            }
        }
    }

    async fn search_mods(&mut self, query: &str) {
        let mods = modrinth::search_mods(query, "1.20.1").await.unwrap();
        let mut mods: Vec<PanelEntry> = mods.hits.iter().map(|item| PanelEntry{display_value: Box::new(item.title.to_owned())}).collect();
        self.left.panel_entries.append(&mut mods);
    }

    async fn handle_key_search_mode(&mut self, key: KeyEvent) {
        match key.code {
            // crossterm::event::KeyCode::Backspace => todo!(),
            // crossterm::event::KeyCode::Left => todo!(),
            // crossterm::event::KeyCode::Right => todo!(),
            // crossterm::event::KeyCode::Up => todo!(),
            // crossterm::event::KeyCode::Down => todo!(),
            // crossterm::event::KeyCode::Home => todo!(),
            // crossterm::event::KeyCode::End => todo!(),
            // crossterm::event::KeyCode::PageUp => todo!(),
            // crossterm::event::KeyCode::PageDown => todo!(),
            // crossterm::event::KeyCode::Tab => todo!(),
            // crossterm::event::KeyCode::BackTab => todo!(),
            // crossterm::event::KeyCode::Delete => todo!(),
            // crossterm::event::KeyCode::Insert => todo!(),
            // crossterm::event::KeyCode::F(_) => todo!(),
            // crossterm::event::KeyCode::Null => todo!(),
            crossterm::event::KeyCode::Esc => self.enter_normal_mode(),
            // crossterm::event::KeyCode::CapsLock => todo!(),
            // crossterm::event::KeyCode::ScrollLock => todo!(),
            // crossterm::event::KeyCode::NumLock => todo!(),
            // crossterm::event::KeyCode::PrintScreen => todo!(),
            // crossterm::event::KeyCode::Pause => todo!(),
            // crossterm::event::KeyCode::Menu => todo!(),
            // crossterm::event::KeyCode::KeypadBegin => todo!(),
            // crossterm::event::KeyCode::Media(_) => todo!(),
            // crossterm::event::KeyCode::Modifier(_) => todo!(),
            crossterm::event::KeyCode::Enter => {
                self.search_mods(&self.search_string.clone()).await;
                self.search_string = String::new();

                self.enter_normal_mode();
                self.focused_col = 0;
            },
            crossterm::event::KeyCode::Char(c) => {
                self.search_string.push(c);
            },
            _ => {}
        }
    }

    fn handle_key_normal_mode(&mut self, key: KeyEvent) {
        match key.code {
            // crossterm::event::KeyCode::Backspace => todo!(),
            // crossterm::event::KeyCode::Enter => todo!(),
            crossterm::event::KeyCode::Left => self.move_entry_right(),
            crossterm::event::KeyCode::Right => self.move_entry_left(),
            crossterm::event::KeyCode::Up => self.focus_prev(),
            crossterm::event::KeyCode::Down => self.focus_next(),
            crossterm::event::KeyCode::Home => self.focus_first(),
            crossterm::event::KeyCode::End => self.focus_last(),
            // crossterm::event::KeyCode::PageUp => todo!(),
            // crossterm::event::KeyCode::PageDown => todo!(),
            crossterm::event::KeyCode::Tab => self.swap_column(),
            // crossterm::event::KeyCode::BackTab => todo!(),
            // crossterm::event::KeyCode::Delete => todo!(),
            // crossterm::event::KeyCode::Insert => todo!(),
            // crossterm::event::KeyCode::F(_) => todo!(),
            crossterm::event::KeyCode::Char(c) => match self.handle_key(c) {
                KeyAction::FocusUp         => self.focus_prev(),
                KeyAction::FocusDown       => self.focus_next(),
                KeyAction::MoveLeft        => self.move_entry_right(),
                KeyAction::MoveRight       => self.move_entry_left(),
                KeyAction::FocusFirst      => self.focus_first(),
                KeyAction::FocusLast       => self.focus_last(),
                KeyAction::StartSearchMode => self.enter_search_mode(),
                KeyAction::Delete          => self.delete(),
                KeyAction::Clear           => self.clear_left(),
                KeyAction::Quit            => self.should_close = true,
                KeyAction::None => {}
            },
            crossterm::event::KeyCode::Esc => self.should_close = true,
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
    }

    fn handle_key(&mut self, key: char) -> KeyAction {
        match key {
            'q' => KeyAction::Quit,
            'j' => KeyAction::FocusDown,
            'k' => KeyAction::FocusUp,
            'h' => KeyAction::MoveLeft,
            'l' => KeyAction::MoveRight,
            'g' => KeyAction::FocusFirst,
            'G' => KeyAction::FocusLast,
            'f' | '/' => KeyAction::StartSearchMode,
            'c' => KeyAction::Clear,
            'd' => KeyAction::Delete,
            _ => KeyAction::None,
        }
    }

    fn clear_left(&mut self) {
        self.left.panel_entries.clear();
    }

    fn enter_search_mode(&mut self) {
        self.mode = ApplicationMode::Search;
    }
    
    fn enter_normal_mode(&mut self) {
        self.mode = ApplicationMode::Normal;
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

    fn focus_first(&mut self) {
        match self.focused_col {
            0 => self.left.focus_first(),
            1 => self.right.focus_first(),
            _ => {},
        }
    }

    fn focus_last(&mut self) {
        match self.focused_col {
            0 => self.left.focus_last(),
            1 => self.right.focus_last(),
            _ => {},
        }
    }

    fn delete(&mut self) {
        match self.focused_col {
            0 => self.left.delete_selection(),
            1 => self.right.delete_selection(),
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