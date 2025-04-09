pub struct Cursor {
    pub x: usize,
    pub y: usize,
}

impl Default for Cursor {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

