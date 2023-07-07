use crate::{core::{ApplicationMode, KeyAction, ModStatus, Repository}, Panel, PanelEntry, modrinth::{ModrinthRepository}, search_field::SearchField, mc_mod::{ModDirectory}};
use crossterm::{queue, cursor::{DisableBlinking, Hide, Show}, event::KeyEvent, terminal::{enable_raw_mode, disable_raw_mode, Clear}};
use std::{io::{Write, stdout}, path::PathBuf};
use crate::core::ModRepository;
use crate::curseforge::repository::CurseforgeRepository;

pub struct Display {
    pub width: u16,
    pub height: u16,

    pub left: Panel,
    pub right: Panel,
    pub focused_col: u8,

    pub mode: ApplicationMode,
    pub search_string: SearchField,
    pub mod_directory: ModDirectory,
    pub mod_location: PathBuf,

    pub repository: Box<dyn Repository>,

    pub should_close: bool,
}

impl Display {
    pub fn new(mod_directory: ModDirectory, mod_location: PathBuf) -> Result<Self, ()> {
        let size = crossterm::terminal::size().map_err(|_| ())?;

        let repository: Box<dyn Repository> = match mod_directory.mod_repository {
            ModRepository::Modrinth => Box::<ModrinthRepository>::default(),
            ModRepository::Curseforge => Box::<CurseforgeRepository>::default(),
        };

        let mut right = Panel::new(size.0/2, size.1);
        mod_directory.mods.iter().for_each(|m| right.panel_entries.push(PanelEntry { data: m.clone() }));

        let mod_directory = ModDirectory { 
            game_version: mod_directory.game_version.clone(), 
            mod_loader: mod_directory.mod_loader, 
            mod_repository: mod_directory.mod_repository,
            mods: vec![]
        };

        Ok(Self {
            mode: ApplicationMode::Normal,
            should_close: false,
            search_string: SearchField::new(),
            width: size.0,
            height: size.1,
            left: Panel::new(size.0/2, size.1),
            right,
            focused_col: 0,
            mod_directory,
            repository,
            mod_location,
        })
    }

    pub async fn process_events(&mut self) {
        let _ = enable_raw_mode();
        'event_loop: loop {
            self.redraw();

            match crossterm::event::read().unwrap() {
                crossterm::event::Event::Key(key) => {
                    match self.mode {
                        ApplicationMode::Normal => self.handle_key_normal_mode(key).await,
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

       let mod_exists_in_panel = |panel: &Panel, mod_identifier: &str| {
            panel.panel_entries
                .iter()
                .any(|entry| *entry.data.mod_identifier == *mod_identifier)
        };

        self.repository
            .search_mods(query, &self.mod_directory.game_version, self.mod_directory.mod_loader)
            .await
            .into_iter()
            .for_each(|m| {
                let exists_left = mod_exists_in_panel(&self.left, &m.mod_identifier);
                let exists_right = mod_exists_in_panel(&self.right, &m.mod_identifier);

                if !exists_left && !exists_right {
                    self.left.panel_entries.push(PanelEntry { data: m });
                }
            });
    }

    async fn handle_key_search_mode(&mut self, key: KeyEvent) {
        match key.code {
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
                self.search_mods(&self.search_string.get_text()).await;
                self.search_string.clear();

                self.enter_normal_mode();
                self.focused_col = 0;
            },
            crossterm::event::KeyCode::Char(c) => {
                self.search_string.push_char(c);
            },
            crossterm::event::KeyCode::Backspace => self.search_string.delete_last(),
            _ => {}
        }
    }

    async fn handle_key_normal_mode(&mut self, key: KeyEvent) {
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
                KeyAction::Open            => self.open().await,
                KeyAction::Delete          => self.delete(),
                KeyAction::Download        => self.download_all().await,
                KeyAction::Clear           => self.clear_left(),
                KeyAction::Quit            => self.exit(),
                KeyAction::None => {}
            },
            crossterm::event::KeyCode::Esc => self.exit(),
            // crossterm::event::KeyCode::CapsLock => todo!(),
            // crossterm::event::KeyCode::ScrollLock => todo!(),
            // crossterm::event::KeyCode::NumLock => todo!(),
            // crossterm::event::KeyCode::PrintScreen => todo!(),
            // crossterm::event::KeyCode::Pause => todo!(),
            // crossterm::event::KeyCode::Menu => todo!(),
            // crossterm::event::KeyCode::KeypadBegin => todo!(),
            // crossterm::event::KeyCode::Media(_) => todo!(),
            // crossterm::event::KeyCode::Modifier(_) => todo!(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.should_close = true;

        self.right.panel_entries
            .iter()
            .for_each(|m| self.mod_directory.mods.push(m.data.clone()) );

        self.mod_directory.save(&self.mod_location);

        let _ = disable_raw_mode();
    let _ = queue!(stdout(), Show, Clear(crossterm::terminal::ClearType::All));
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
            'o' => KeyAction::Open,
            'c' => KeyAction::Clear,
            'x' => KeyAction::Delete,
            'd' => KeyAction::Download,
            _ => KeyAction::None,
        }
    }

    async fn open(&self) {
        match self.focused_col {
            0 => self.left.open_selected(&self.repository).await,
            1 => self.right.open_selected(&self.repository).await,
            _ => {},
        }
    }

    async fn download_all(&mut self) {
        self.right.download_all(
            &self.repository,
            &self.mod_directory.game_version,
            &self.mod_directory.mod_loader,
            &self.mod_location
        ).await;
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
        if self.focused_col == 0  {
            self.left.move_to_panel(&mut self.right);
        }
    }
    
    fn move_entry_right(&mut self) {
        if self.focused_col == 1 && self.right.get_focused().is_some_and(|entry| entry.data.status == ModStatus::Normal) {
            self.right.move_to_panel(&mut self.left);
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
            1 => {
                let can_delete = self.right.get_focused().is_some_and(|entry| entry.data.status == ModStatus::Normal);
                if can_delete {
                    self.right.delete_selection();
                }
            }
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
        let _ = stdout().flush();
    }

    fn draw_ui(&self) {
        self.left.draw_frame(0, 0, self.focused_col == 0);
        self.right.draw_frame(self.width/2, 0, self.focused_col == 1);
        self.draw_search_field();

        self.left.draw_entries(1);
        self.right.draw_entries(self.width/2+1);
    }

    fn draw_search_field(&self) {
        use crossterm::cursor::{MoveTo};
        use crossterm::style::{Print};
        let mut stdout = stdout();

        let text = self.search_string.get_display((self.width/2-4) as usize);

        let _ = queue!(stdout, MoveTo(2,0), Print(text));
    } 

    fn clear_screen(&self) {
        use crossterm::cursor::{MoveTo};
        use crossterm::style::{Print};
        let mut stdout = stdout();

        for y in 0..self.height {
            for x in 0..self.width {
                let _ = queue!(stdout, DisableBlinking, Hide, MoveTo(x,y), Print(' '));
            }
        }
    }
}