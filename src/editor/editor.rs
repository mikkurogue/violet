use crossterm::{
    cursor::{self, MoveTo},
    event::KeyCode,
    queue,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal,
};
use std::path::Path;
use std::{
    fs::read_to_string,
    io::{self, Write},
};

use crate::{
    buffer::{buffer::Buffer, render_buffer::RenderBuffer, render_cell::RenderCell},
    command_prompt::CommandPrompt,
    cursor::Cursor,
    highlighter::Highlighter,
};

use super::{mode::Mode, theme::Theme};

pub struct Editor {
    buffer: Buffer,
    cursor: Cursor,
    mode: Mode,
    command_prompt: CommandPrompt,
    render_buffer: RenderBuffer,
    prev_render_buffer: RenderBuffer,
    viewport_x: usize,
    viewport_y: usize,
    last_key: Option<KeyCode>,
    motion_count: Option<usize>,
    highlighter: Highlighter,
}

impl Editor {
    pub fn new(filename: Option<String>) -> Self {
        let (width, height) = terminal::size().unwrap_or((80, 24));

        // initialize with empty buffer
        let buffer = if let Some(path) = filename {
            Self::open_file(&path).unwrap_or_else(|_| {
                // if no file, create a new empty buffer with the filename
                Buffer::new(path, String::new())
            })
        } else {
            Buffer::new("Untitled".to_string(), String::new())
        };

        let def_theme = Theme::default();
        let h = Highlighter::new(def_theme).unwrap();

        Self {
            buffer,
            cursor: Cursor::default(),
            mode: Mode::Normal,
            command_prompt: CommandPrompt::new(),
            render_buffer: RenderBuffer::new(width as usize, height as usize),
            prev_render_buffer: RenderBuffer::new(width as usize, height as usize),
            viewport_x: 0,
            viewport_y: 0,
            last_key: None,
            motion_count: None,
            highlighter: h,
        }
    }

    fn open_file(path: &str) -> Result<Buffer, io::Error> {
        let contents = read_to_string(path)?;
        let filename = Path::new(path)
            .file_name()
            .map(|osstr| osstr.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());

        Ok(Buffer::new(filename, contents))
    }

    pub fn handle_keypress(&mut self, key: KeyCode) -> bool {
        match self.mode {
            Mode::Normal => self.handle_normal_mode(key),
            Mode::Insert => self.handle_insert_mode(key),
            Mode::Command => self.handle_command_mode(key),
        }
    }

    fn handle_command_mode(&mut self, key: KeyCode) -> bool {
        if let Some(command) = self.command_prompt.handle_key(key) {
            if command == "q" || command == "quit" {
                return true;
            } else {
                self.execute_command(&command);
                self.mode = Mode::Normal;
            }
        }

        if !self.command_prompt.is_active() {
            self.mode = Mode::Normal;
        }

        false
    }

    fn handle_normal_mode(&mut self, key: KeyCode) -> bool {
        if let Some(KeyCode::Char('d')) = self.last_key {
            if key == KeyCode::Char('d') {
                self.delete_current_line();
                self.last_key = None;
                self.motion_count = None;
                return false;
            }
            self.last_key = None;
            return self.handle_normal_mode(key);
        }

        match key {
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let digit = c.to_digit(10).unwrap() as usize;
                self.motion_count = Some(
                    self.motion_count
                        .unwrap_or(0)
                        .saturating_mul(10)
                        .saturating_add(digit),
                );
            }

            KeyCode::Char('j') => {
                let count = self.motion_count.take().unwrap_or(1);
                for _ in 0..count {
                    self.move_cursor_down();
                }
            }
            KeyCode::Char('k') => {
                let count = self.motion_count.take().unwrap_or(1);
                for _ in 0..count {
                    self.move_cursor_up();
                }
            }
            KeyCode::Char('h') => {
                let count = self.motion_count.take().unwrap_or(1);
                for _ in 0..count {
                    self.move_cursor_left();
                }
            }
            KeyCode::Char('l') => {
                let count = self.motion_count.take().unwrap_or(1);
                for _ in 0..count {
                    self.move_cursor_right();
                }
            }
            KeyCode::Char('w') => {
                let count = self.motion_count.take().unwrap_or(1);
                for _ in 0..count {
                    self.jump_cursor_word();
                }
            }

            KeyCode::Char('b') => {
                let count = self.motion_count.take().unwrap_or(1);
                for _ in 0..count {
                    self.jump_cursor_word_reverse();
                }
            }

            KeyCode::Char('d') => {
                self.last_key = Some(KeyCode::Char('d'));
                self.motion_count = None;
            }

            KeyCode::Char('x') => {
                self.delete_char_at_cursor();
                self.motion_count = None;
            }

            KeyCode::Char('i') => {
                self.mode = Mode::Insert;
                self.motion_count = None;
            }

            KeyCode::Char(':') => {
                self.mode = Mode::Command;
                self.command_prompt.activate();
                self.motion_count = None;
            }

            KeyCode::Char('0') => {
                if self.motion_count.is_some() {
                    self.motion_count = Some(self.motion_count.unwrap().saturating_mul(10));
                } else {
                    self.cursor.x = 0;
                }
            }

            KeyCode::Char('$') => {
                if let Some(line) = self.buffer.get_line(self.cursor.y) {
                    self.cursor.x = line.len().saturating_sub(1);
                }
                self.motion_count = None;
            }

            KeyCode::Char('g') => {
                self.cursor.y = 0;
                self.cursor.x = 0;
                self.motion_count = None;
            }

            KeyCode::Char('G') => {
                self.cursor.y = self.buffer.line_count().saturating_sub(1);
                self.cursor.x = 0;
                self.motion_count = None;
            }

            _ => {
                self.motion_count = None; // unknown key resets count
            }
        }

        false
    }

    fn handle_insert_mode(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Char(c) => self.insert_char(c),
            KeyCode::Backspace => self.delete_char_before_cursor(),
            KeyCode::Delete => self.delete_char_at_cursor(),
            KeyCode::Enter => self.insert_newline(),
            KeyCode::Left => self.move_cursor_left(),
            KeyCode::Right => self.move_cursor_right(),
            KeyCode::Up => self.move_cursor_up(),
            KeyCode::Down => self.move_cursor_down(),
            _ => {}
        }
        false
    }

    fn execute_command(&mut self, command: &str) -> bool {
        let command = command.trim();

        match command {
            "w" | "write" => {
                if !self.buffer.buffer_name.is_empty() && self.buffer.buffer_name != "Untitled" {
                    self.save_buffer(&self.buffer.buffer_name.clone());
                }
            }
            // save da buffer
            cmd if cmd.starts_with("w ") => {
                let filename = cmd[2..].trim();
                if !filename.is_empty() {
                    self.save_buffer(filename);
                }
            }
            cmd if cmd.starts_with("e ") => {
                let filename = cmd[2..].trim();
                if !filename.is_empty() {
                    if let Ok(buffer) = Self::open_file(filename) {
                        self.buffer = buffer;
                        self.cursor.x = 0;
                        self.cursor.y = 0;
                    }
                }
            }
            _ => {} // Unknown command
        }

        false
    }

    fn save_buffer(&mut self, filename: &str) {
        if std::fs::write(filename, &self.buffer.text).is_ok() {
            self.buffer.buffer_name = Path::new(filename)
                .file_name()
                .map(|osstr| osstr.to_string_lossy().to_string())
                .unwrap_or_else(|| filename.to_string());
        }
    }

    fn insert_char(&mut self, ch: char) {
        if ch == '\n' {
            self.insert_newline();
            return;
        }

        if let Some(byte_pos) = self
            .buffer
            .char_to_byte_position(self.cursor.y, self.cursor.x)
        {
            self.buffer.text.insert(byte_pos, ch);
            self.cursor.x += 1;

            // Update line offsets for all lines after this one
            for i in self.cursor.y + 1..self.buffer.line_count() {
                self.buffer.line_offsets[i] += ch.len_utf8();
            }
        }
    }

    fn delete_char_before_cursor(&mut self) {
        if self.cursor.x > 0 {
            if let Some(byte_pos) = self
                .buffer
                .char_to_byte_position(self.cursor.y, self.cursor.x - 1)
            {
                let ch = self.buffer.text.remove(byte_pos);
                self.cursor.x -= 1;

                // Update line offsets for all lines after this one
                for i in self.cursor.y + 1..self.buffer.line_count() {
                    self.buffer.line_offsets[i] -= ch.len_utf8();
                }
            }
        } else if self.cursor.y > 0 {
            // Handle joining with previous line
            let prev_line_len = self
                .buffer
                .get_line(self.cursor.y - 1)
                .map(|l| l.len())
                .unwrap_or(0);

            if let Some(byte_pos) = self.buffer.char_to_byte_position(self.cursor.y, 0) {
                self.buffer.text.remove(byte_pos - 1); // Remove the newline
                self.buffer.line_offsets.remove(self.cursor.y);

                // Update line offsets for all lines after this one
                for i in self.cursor.y..self.buffer.line_count() {
                    self.buffer.line_offsets[i] -= 1;
                }

                self.cursor.y -= 1;
                self.cursor.x = prev_line_len;
            }
        }
    }

    fn delete_char_at_cursor(&mut self) {
        if let Some(byte_pos) = self
            .buffer
            .char_to_byte_position(self.cursor.y, self.cursor.x)
        {
            let ch = self.buffer.text.remove(byte_pos);

            if ch == '\n' {
                // Remove the line offset for the next line
                let pos = self
                    .buffer
                    .line_offsets
                    .iter()
                    .position(|&p| p == byte_pos + 1);

                if let Some(pos) = pos {
                    self.buffer.line_offsets.remove(pos);

                    // Update line offsets for all lines after this one
                    for i in pos..self.buffer.line_count() {
                        self.buffer.line_offsets[i] -= 1;
                    }
                }
            } else {
                // Update line offsets for all lines after this one
                for i in self.cursor.y + 1..self.buffer.line_count() {
                    self.buffer.line_offsets[i] -= ch.len_utf8();
                }
            }
        }
    }

    fn delete_current_line(&mut self) {
        if self.buffer.line_count() == 0 {
            return;
        }

        // Get the byte range for the current line
        let line_start = self.buffer.line_offsets[self.cursor.y];
        let line_end = if self.cursor.y + 1 < self.buffer.line_offsets.len() {
            self.buffer.line_offsets[self.cursor.y + 1] - 1 // -1 to exclude the newline
        } else {
            self.buffer.text.len()
        };

        // Remove the line
        self.buffer.text.replace_range(line_start..line_end, "");
        self.buffer.line_offsets.remove(self.cursor.y);

        // Adjust cursor position
        if self.cursor.y >= self.buffer.line_count() {
            self.cursor.y = self.buffer.line_count().saturating_sub(1);
        }
        self.cursor.x = 0;
    }

    fn insert_newline(&mut self) {
        let byte_pos = self
            .buffer
            .char_to_byte_position(self.cursor.y, self.cursor.x)
            .unwrap_or_else(|| self.buffer.text.len());

        self.buffer.text.insert(byte_pos, '\n');

        // Update line offsets
        let new_line_pos = byte_pos + 1;
        self.buffer.line_offsets.push(new_line_pos);
        self.buffer.line_offsets.sort();

        self.cursor.y += 1;
        self.cursor.x = 0;
    }

    fn move_cursor_left(&mut self) {
        if self.cursor.x > 0 {
            self.cursor.x -= 1;
        } else if self.cursor.y > 0 {
            // Move to end of previous line
            self.cursor.y -= 1;
            self.cursor.x = self
                .buffer
                .get_line(self.cursor.y)
                .map(|l| l.chars().count())
                .unwrap_or(0);
        }
    }

    fn move_cursor_right(&mut self) {
        if let Some(line) = self.buffer.get_line(self.cursor.y) {
            let line_len = line.chars().count();
            if self.cursor.x < line_len {
                self.cursor.x += 1;
            } else if self.cursor.y + 1 < self.buffer.line_count() {
                // Move to start of next line
                self.cursor.y += 1;
                self.cursor.x = 0;
            }
        }
    }

    fn jump_cursor_word(&mut self) {
        if let Some(line) = self.buffer.get_line(self.cursor.y) {
            let line_len = line.len();

            if self.cursor.x < line_len {
                let mut new_cursor_x = self.cursor.x;
                let mut char_iter = line.chars().skip(self.cursor.x);

                // Skip current word
                while let Some(c) = char_iter.next() {
                    if c.is_whitespace() {
                        break;
                    }
                    new_cursor_x += 1;
                }

                // Skip whitespace
                while let Some(c) = char_iter.next() {
                    if !c.is_whitespace() {
                        break;
                    }
                    new_cursor_x += 1;
                }

                self.cursor.x = new_cursor_x.min(line_len);
            } else if self.cursor.y + 1 < self.buffer.line_count() {
                // Move to next line
                self.cursor.y += 1;
                self.cursor.x = 0;
                self.jump_cursor_word();
            }
        }
    }

    fn jump_cursor_word_reverse(&mut self) {
        if let Some(line) = self.buffer.get_line(self.cursor.y) {
            let line_len = line.len();

            if self.cursor.x < line_len {
                let mut new_cursor_x = self.cursor.x;
                let mut char_iter = line.chars().skip(self.cursor.x);

                // Skip current word
                while let Some(c) = char_iter.next() {
                    if c.is_whitespace() {
                        break;
                    }
                    new_cursor_x -= 1;
                }

                // Skip whitespace
                while let Some(c) = char_iter.next() {
                    if !c.is_whitespace() {
                        break;
                    }
                    new_cursor_x -= 1;
                }

                self.cursor.x = new_cursor_x.min(line_len);
            } else if self.cursor.y - 1 < self.buffer.line_count() {
                // Move to next line
                self.cursor.y -= 1;
                self.cursor.x = 0;
                self.jump_cursor_word();
            }
        }
    }

    fn move_cursor_up(&mut self) {
        if self.cursor.y > 0 {
            self.cursor.y -= 1;
            self.clamp_cursor_x();

            if self.cursor.y < self.viewport_y {
                self.viewport_y = self.cursor.y;
            }
        }
    }

    fn move_cursor_down(&mut self) {
        if self.cursor.y + 1 < self.buffer.line_count() {
            self.cursor.y += 1;
            self.clamp_cursor_x();

            let visible_lines = self.render_buffer.height.saturating_sub(1);
            if self.cursor.y >= self.viewport_y + visible_lines {
                self.viewport_y = self.cursor.y - visible_lines + 1;
            }
        }
    }

    fn clamp_cursor_x(&mut self) {
        if let Some(line) = self.buffer.get_line(self.cursor.y) {
            let line_len = line.chars().count();
            if self.cursor.x > line_len {
                self.cursor.x = line_len;
            }
        }
    }

    pub fn prepare_render_buffer(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (term_width, term_height) = terminal::size()?;

        // Resize buffers if terminal size changed
        self.render_buffer
            .resize(term_width as usize, term_height as usize);
        self.prev_render_buffer
            .resize(term_width as usize, term_height as usize);

        // Clear the current render buffer
        for y in 0..self.render_buffer.height {
            for x in 0..self.render_buffer.width {
                self.render_buffer.set_cell(x, y, RenderCell::default());
            }
        }

        // Prepare content for rendering
        let total_lines = self.buffer.line_count();
        let line_number_width = total_lines.to_string().len();

        // Calculate visible area (viewport)
        let visible_lines = self.render_buffer.height.saturating_sub(2); // Leave room for status bar and command prompt
        let visible_cols = self
            .render_buffer
            .width
            .saturating_sub(line_number_width + 2); // Account for line numbers

        // Adjust viewport to keep cursor visible
        if self.cursor.y < self.viewport_y {
            self.viewport_y = self.cursor.y;
        } else if self.cursor.y >= self.viewport_y + visible_lines {
            self.viewport_y = self.cursor.y - visible_lines + 1;
        }

        if self.cursor.x < self.viewport_x {
            self.viewport_x = self.cursor.x;
        } else if self.cursor.x >= self.viewport_x + visible_cols {
            self.viewport_x = self.cursor.x - visible_cols + 1;
        }

        // Render text content (only visible portion)
        for render_y in 0..visible_lines {
            let buffer_y = self.viewport_y + render_y;

            // Skip if we've gone past the buffer contents
            if buffer_y >= self.buffer.line_count() {
                break;
            }

            let line = self.buffer.get_line(buffer_y).unwrap_or("");
            let is_active = buffer_y == self.cursor.y; // Render line numbers/status column

            for x in 0..line_number_width {
                let mut cell = RenderCell::default();
                let line_number = (buffer_y + 1).to_string();
                let padding = line_number_width - line_number.len();

                if x >= padding {
                    cell.ch = line_number.chars().nth(x - padding).unwrap_or(' ');
                } else {
                    cell.ch = ' ';
                }

                cell.fg = if is_active {
                    Color::Yellow
                } else {
                    Color::DarkGrey
                };
                self.render_buffer.set_cell(x, render_y, cell);
            }

            let mut separator_cell = RenderCell::default();
            separator_cell.ch = '│';
            separator_cell.fg = Color::DarkGrey;
            self.render_buffer
                .set_cell(line_number_width, render_y, separator_cell);

            self.highlighter.highlight(line)?;

            // Render visible portion of the line
            for (buffer_x, ch) in line
                .chars()
                .skip(self.viewport_x)
                .take(visible_cols)
                .enumerate()
            {
                let render_x = line_number_width + 1 + buffer_x;
                if render_x >= self.render_buffer.width {
                    break;
                }

                let mut cell = RenderCell::default();
                cell.ch = ch;

                if is_active && (self.viewport_x + buffer_x) == self.cursor.x {
                    match self.mode {
                        Mode::Normal => {
                            cell.bg = Color::White;
                            cell.fg = Color::Black;
                        }
                        Mode::Insert => {
                            cell.bg = Color::Red;
                            cell.fg = Color::Black;
                        }
                        Mode::Command => {
                            cell.bg = Color::Yellow;
                            cell.fg = Color::Black;
                        }
                    }
                }

                self.render_buffer.set_cell(render_x, render_y, cell);
            }

            // Render cursor at end of line if needed
            if is_active && self.cursor.x == line.len() {
                let cursor_rel_x = self.cursor.x.saturating_sub(self.viewport_x);
                let render_x = line_number_width + 1 + cursor_rel_x;
                if render_x < self.render_buffer.width {
                    let mut cell = RenderCell::default();
                    cell.ch = ' ';
                    match self.mode {
                        Mode::Normal => {
                            cell.bg = Color::White;
                            cell.fg = Color::Black;
                        }
                        Mode::Insert => {
                            cell.bg = Color::Green;
                            cell.fg = Color::Black;
                        }
                        Mode::Command => {
                            cell.bg = Color::White;
                            cell.fg = Color::Black;
                        }
                    }
                    self.render_buffer.set_cell(render_x, render_y, cell);
                }
            }
        }

        // Render status bar
        let mode_str = match self.mode {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Command => "COMMAND",
        };

        let status = format!(
            "{} | {} | Line: {}/{} Col: {}/{} ",
            mode_str,
            self.buffer.buffer_name,
            self.cursor.y + 1,
            self.buffer.line_count(),
            self.cursor.x + 1,
            self.buffer
                .get_line(self.cursor.y)
                .map(|l| l.len())
                .unwrap_or(0)
        );

        let status_y = term_height as usize - 2;
        for (x, ch) in status.chars().enumerate() {
            if x >= self.render_buffer.width {
                break;
            }

            let mut cell = RenderCell::default();
            cell.ch = ch;

            let cell_color = match self.mode {
                Mode::Normal => Color::Green,
                Mode::Command => Color::Yellow,
                Mode::Insert => Color::Red,
            };

            cell.bg = cell_color;
            cell.fg = Color::White;

            self.render_buffer.set_cell(x, status_y, cell);
        }

        // Render command prompt if active
        if self.command_prompt.is_active() {
            let prompt_text = format!("~ {}", self.command_prompt.get_command());
            let prompt_width = prompt_text.len();
            let prompt_x = (self.render_buffer.width.saturating_sub(prompt_width)) / 2;
            let prompt_y = 0;

            // Draw background for prompt
            for x in 0..self.render_buffer.width {
                let mut cell = RenderCell::default();
                cell.ch = ' ';
                cell.bg = Color::DarkBlue;
                self.render_buffer.set_cell(x, prompt_y, cell);
            }

            // Draw prompt text
            for (i, ch) in prompt_text.chars().enumerate() {
                let x = prompt_x + i;
                if x < self.render_buffer.width {
                    let mut cell = RenderCell::default();
                    cell.ch = ch;
                    cell.fg = Color::Black;
                    cell.bg = Color::DarkBlue;
                    self.render_buffer.set_cell(x, prompt_y, cell);
                }
            }

            // Draw cursor in prompt
            let cursor_x = prompt_x + 2 + self.command_prompt.get_cursor_pos();
            if cursor_x < self.render_buffer.width {
                let mut cell = RenderCell::default();
                cell.ch = ' ';
                cell.bg = Color::White;
                cell.fg = Color::Black;
                self.render_buffer.set_cell(cursor_x, prompt_y, cell);
            }
        }

        Ok(())
    }

    pub fn render<W: Write>(&mut self, out: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        self.prepare_render_buffer()?;

        // Only update cells that have changed
        for y in 0..self.render_buffer.height {
            for x in 0..self.render_buffer.width {
                let current = self.render_buffer.get_cell(x, y);
                let previous = self.prev_render_buffer.get_cell(x, y);

                if current != previous {
                    if let Some(cell) = current {
                        queue!(
                            out,
                            MoveTo(x as u16, y as u16),
                            SetForegroundColor(cell.fg),
                            SetBackgroundColor(cell.bg),
                            Print(cell.ch)
                        )?;
                    }
                }
            }
        }

        // Position cursor correctly
        let (cursor_x, cursor_y) = if self.command_prompt.is_active() {
            let prompt_text = format!(":{}", self.command_prompt.get_command());
            let prompt_width = prompt_text.len();
            let prompt_x = (self.render_buffer.width.saturating_sub(prompt_width)) / 2;
            (
                (prompt_x + 1 + self.command_prompt.get_cursor_pos()) as u16,
                0, // Top line
            )
        } else {
            let line_number_width = self.buffer.line_count().to_string().len();
            (
                (line_number_width + 1 + self.cursor.x.saturating_sub(self.viewport_x)) as u16,
                (self.cursor.y.saturating_sub(self.viewport_y)) as u16,
            )
        };

        match self.mode {
            Mode::Normal => {
                queue!(
                    out,
                    MoveTo(cursor_x, cursor_y),
                    cursor::SetCursorStyle::BlinkingBlock,
                    cursor::Show
                )?;
            }
            Mode::Insert => {
                queue!(
                    out,
                    MoveTo(cursor_x, cursor_y),
                    cursor::SetCursorStyle::BlinkingBar,
                    cursor::Show
                )?;
            }
            Mode::Command => {
                queue!(
                    out,
                    MoveTo(cursor_x, cursor_y),
                    cursor::SetCursorStyle::BlinkingUnderScore,
                    cursor::Show
                )?;
            }
        }

        // Swap buffers
        std::mem::swap(&mut self.render_buffer, &mut self.prev_render_buffer);

        out.flush()?;
        Ok(())
    }
}
