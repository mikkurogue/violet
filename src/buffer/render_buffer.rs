use super::render_cell::RenderCell;

pub struct RenderBuffer {
    pub buffer: Vec<Vec<RenderCell>>,
    pub width: usize,
    pub height: usize,
}

impl RenderBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut buffer = Vec::with_capacity(height);
        for _ in 0..height {
            let row = vec![RenderCell::default(); width];
            buffer.push(row);
        }
        Self {
            buffer,
            width,
            height,
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        if self.width == width && self.height == height {
            return;
        }

        self.width = width;
        self.height = height;

        self.buffer
            .resize_with(height, || vec![RenderCell::default(); width]);
        for row in &mut self.buffer {
            row.resize_with(width, RenderCell::default);
        }
    }

    pub fn get_cell(&self, x: usize, y: usize) -> Option<&RenderCell> {
        self.buffer.get(y).and_then(|row| row.get(x))
    }

    pub fn set_cell(&mut self, x: usize, y: usize, cell: RenderCell) {
        if y < self.buffer.len() && x < self.width {
            if let Some(row) = self.buffer.get_mut(y) {
                if x < row.len() {
                    row[x] = cell;
                }
            }
        }
    }
}
