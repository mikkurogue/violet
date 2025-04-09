use crossterm::style::Color;

#[derive(PartialEq, Clone)]
pub struct RenderCell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub italic: bool,
}

impl Default for RenderCell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: Color::Reset,
            bg: Color::Reset,
            bold: false,
            italic: false,
        }
    }
}
