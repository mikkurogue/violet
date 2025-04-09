use super::color::Color;

#[derive(Debug)]
pub struct Theme {
    pub name: String,
    pub style: Style,
    pub gutter_style: Style,
    pub statusline_style: StatuslineStyle,
}

impl Theme {
    pub fn get_style(&self, scope: &str) -> Option<Style> {
        match scope {
            "function" => Some(Style {
                fg: Some(Color::Rgb {
                    r: 66,
                    g: 135,
                    b: 245,
                }), // Blue
                bold: true,
                ..Style::default()
            }),
            "keyword" => Some(Style {
                fg: Some(Color::Rgb {
                    r: 204,
                    g: 120,
                    b: 50,
                }), // Orange
                bold: true,
                ..Style::default()
            }),
            "string" => Some(Style {
                fg: Some(Color::Rgb {
                    r: 152,
                    g: 195,
                    b: 121,
                }), // Green
                ..Style::default()
            }),
            "comment" => Some(Style {
                fg: Some(Color::Rgb {
                    r: 92,
                    g: 99,
                    b: 112,
                }), // Gray
                italic: true,
                ..Style::default()
            }),
            _ => None,
        }
    }

    pub fn get_selection_bg(&self) -> Color {
        todo!()
    }
}

impl Default for Theme {
    fn default() -> Theme {
        Theme {
            name: "default".to_string(),
            style: Style {
                fg: Some(Color::Rgb {
                    r: 220,
                    g: 220,
                    b: 220,
                }), // Light gray
                bg: Some(Color::Rgb {
                    r: 40,
                    g: 42,
                    b: 54,
                }), // Dark background
                bold: false,
                italic: false,
            },
            gutter_style: Style {
                fg: Some(Color::Rgb {
                    r: 150,
                    g: 150,
                    b: 150,
                }),
                bg: Some(Color::Rgb {
                    r: 30,
                    g: 32,
                    b: 44,
                }),
                ..Style::default()
            },
            statusline_style: StatuslineStyle::default(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct StatuslineStyle {
    pub outer_style: Style,
    pub outer_chars: [char; 4],
    pub inner_style: Style,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
}

impl Style {
    pub fn fallback_bg(&self, fallback_bg: &Style) -> Style {
        let bg = self
            .bg
            .or(fallback_bg.bg)
            .or(Some(Color::Rgb { r: 0, g: 0, b: 0 }));
        self.with_bg(bg)
    }

    pub fn with_bg(&self, bg: Option<Color>) -> Style {
        Style { bg, ..self.clone() }
    }

    pub fn inverted(&self) -> Style {
        Style {
            fg: self.bg,
            bg: self.fg,
            bold: self.bold,
            italic: self.italic,
        }
    }
}

#[derive(Debug)]
pub struct StyleInfo {
    pub start: usize,
    pub end: usize,
    pub style: Style,
}

impl StyleInfo {
    pub fn contains(&self, pos: usize) -> bool {
        pos >= self.start && pos < self.end
    }
}
