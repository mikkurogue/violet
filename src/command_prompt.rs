use crossterm::event::KeyCode;

pub struct CommandPrompt {
    pub command: String,
    pub cursor_pos: usize, // only need on 1 axis
    active: bool,
}

impl CommandPrompt {
    pub fn new() -> Self {
        Self {
            command: String::new(),
            cursor_pos: 0,
            active: false,
        }
    }

    pub fn activate(&mut self) {
        self.command.clear();
        self.cursor_pos = 0;
        self.active = true;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn handle_key(&mut self, key: KeyCode) -> Option<String> {
        match key {
            KeyCode::Esc => {
                self.deactivate();
                None
            }
            KeyCode::Enter => {
                let command = self.command.clone();
                self.command.clear();
                self.cursor_pos = 0;
                self.deactivate();
                Some(command)
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.command.remove(self.cursor_pos);
                }
                None
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.command.len() {
                    self.command.remove(self.cursor_pos);
                }
                None
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
                None
            }
            KeyCode::Right => {
                if self.cursor_pos < self.command.len() {
                    self.cursor_pos += 1;
                }
                None
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
                None
            }
            KeyCode::End => {
                self.cursor_pos = self.command.len();
                None
            }
            KeyCode::Char(c) => {
                self.command.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
                None
            }
            _ => None,
        }
    }

    pub fn get_command(&self) -> &str {
        &self.command
    }

    pub fn get_cursor_pos(&self) -> usize {
        self.cursor_pos
    }
}
