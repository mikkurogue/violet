pub struct Buffer {
    pub buffer_name: String,
    pub text: String,
    pub line_offsets: Vec<usize>,
}

impl Buffer {
    pub fn new(buffer_name: String, text: String) -> Buffer {
        let mut line_offsets = vec![0];

        line_offsets.extend(text.match_indices('\n').map(|(i, _)| i + 1));

        Buffer {
            buffer_name,
            text,
            line_offsets,
        }
    }

    pub fn update_text(&mut self, text: String) {
        self.text = text;
        self.line_offsets.clear();
        self.line_offsets.push(0);
        self.line_offsets
            .extend(self.text.match_indices('\n').map(|(i, _)| i + 1));
    }

    pub fn line_count(&self) -> usize {
        self.line_offsets.len()
    }

    pub fn get_line(&self, line: usize) -> Option<&str> {
        if line >= self.line_offsets.len() {
            return None;
        }

        let start = self.line_offsets[line];
        let end = if line + 1 < self.line_offsets.len() {
            self.line_offsets[line + 1] - 1
        } else {
            self.text.len()
        };

        Some(&self.text[start..end])
    }

    pub fn char_to_byte_position(&self, line: usize, col: usize) -> Option<usize> {
        if line >= self.line_offsets.len() {
            return None;
        }

        let lstart = self.line_offsets[line];
        let ltext = self.get_line(line)?;

        let line_byte_pos = ltext
            .char_indices()
            .nth(col)
            .map(|(i, _)| i)
            .unwrap_or(ltext.len());

        Some(lstart + line_byte_pos)
    }
}
