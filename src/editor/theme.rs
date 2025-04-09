use super::color::Color;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub style: Style,
    pub gutter: Style,
    pub statusline: Style,
    pub command_prompt: Style,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Style {
    pub text_color: Option<Color>,
    pub background_color: Option<Color>,
    pub bold: bool,
    pub italic: bool,
}

impl Theme {
    pub fn get_style(&self, scope: &str) -> Option<Style> {
        match scope {
            "function" => Some(Style {
                text_color: Some(Color::Rgb {
                    r: 66,
                    g: 135,
                    b: 245,
                }), // Blue
                bold: true,
                ..Style::default()
            }),
            "keyword" => Some(Style {
                text_color: Some(Color::Rgb {
                    r: 204,
                    g: 120,
                    b: 50,
                }), // Orange
                bold: true,
                ..Style::default()
            }),
            "string" => Some(Style {
                text_color: Some(Color::Rgb {
                    r: 152,
                    g: 195,
                    b: 121,
                }), // Green
                ..Style::default()
            }),
            "comment" => Some(Style {
                text_color: Some(Color::Rgb {
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

    pub fn get_selection_background_color(&self) -> Color {
        todo!()
    }
}

impl Default for Theme {
    fn default() -> Theme {
        Theme {
            name: "default".to_string(),
            style: Style {
                text_color: Some(Color::Rgb {
                    r: 207,
                    g: 159,
                    b: 255,
                }), // Lightviolet
                background_color: Some(Color::Rgb {
                    r: 52,
                    g: 21,
                    b: 57,
                }), // Dark violet
                bold: false,
                italic: false,
            },
            gutter: Style {
                text_color: Some(Color::Rgb {
                    r: 150,
                    g: 150,
                    b: 150,
                }),
                background_color: Some(Color::Rgb {
                    r: 30,
                    g: 32,
                    b: 44,
                }),
                ..Style::default()
            },
            statusline: Style {
                text_color: Some(Color::Rgb {
                    r: 207,
                    g: 159,
                    b: 255,
                }),
                background_color: Some(Color::Rgb {
                    r: 52,
                    g: 21,
                    b: 57,
                }),
                bold: false,
                italic: false,
                ..Default::default()
            },
            command_prompt: Style {
                text_color: Some(Color::Rgb {
                    r: 207,
                    g: 159,
                    b: 255,
                }),
                background_color: Some(Color::Rgb {
                    r: 52,
                    g: 21,
                    b: 57,
                }),
                ..Default::default()
            },
        }
    }
}

impl Style {
    pub fn fallback_background_color(&self, fallback_bg: &Style) -> Style {
        let background_color = self
            .background_color
            .or(fallback_bg.background_color)
            .or(Some(Color::Rgb { r: 0, g: 0, b: 0 }));
        self.with_background_color(background_color)
    }

    pub fn with_background_color(&self, bg: Option<Color>) -> Style {
        Style {
            background_color: bg,
            ..self.clone()
        }
    }

    pub fn inverted(&self) -> Style {
        Style {
            text_color: self.background_color,
            background_color: self.text_color,
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
