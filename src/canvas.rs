pub struct Canvas {
    width: usize,
    height: usize,
    cells: Vec<char>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        let n = width * height;

        Canvas {
            width: width,
            height: height,
            cells: vec![' '; n],
        }
    }

    /**
     * Terminal size in cells, or a conservative fallback
     */
    pub fn terminal_dims() -> (usize, usize) {
        terminal_size::terminal_size()
            .map(|(w, h)| (w.0 as usize, h.0 as usize))
            .unwrap_or((100, 40))
    }

    /**
     * Canvas filling the terminal, less one row so the final
     * line never pushes the frame into scrollback
     */
    pub fn from_terminal() -> Self {
        let (w, h) = Self::terminal_dims();
        Self::new(w, h.saturating_sub(1))
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn aspect(&self) -> f64 {
        self.width as f64 / self.height as f64
    }

    pub fn clear(&mut self) {
        self.cells.fill(' ');
    }

    pub fn set(
        &mut self,
        x: isize,
        y: isize,
        character: char,
    ) {
        // Off-canvas coordinates should be discarded
        if x < 0 || y < 0 || x >= self.width as isize || y >= self.height as isize {
            return;
        }

        let idx = self.index(x as usize, y as usize);
        self.cells[idx] = character;
    }

    /**
     * Bresenham's line algorithm for efficiently drawing lines between
     * points in the terminal
     */
    pub fn draw_line(
        &mut self,
        x0: isize,
        y0: isize,
        x1: isize,
        y1: isize,
        character: char,
    ) {
        let mut x0_ = x0;
        let mut y0_ = y0;

        let dx = (x1 - x0_).abs();
        let dy = (y1 - y0_).abs();

        let sx = if x0_ < x1 { 1 } else { -1 };
        let sy = if y0_ < y1 { 1 } else { -1 };

        let mut error = dx - dy;

        loop {
            self.set(x0_, y0_, character);

            if x0_ == x1 && y0_ == y1 {
                break;
            }

            let e2 = 2 * error;

            if e2 > -dy {
                error -= dy;
                x0_ += sx;
            }

            if e2 < dx {
                error += dx;
                y0_ += sy;
            }
        }
    }

    /**
     * Serialize the canvas into `buf` as newline-separated rows.
     * Takes a caller-owned buffer so the allocation is reused across frames.
     */
    pub fn write_to(&self, buf: &mut String) {
        buf.clear();

        for y in 0..self.height {
            buf.extend(self.cells[y * self.width..(y + 1) * self.width].iter());

            // No trailing newline on final row
            if y + 1 < self.height {
                buf.push('\n');
            }
        }
    }

    /**
     * Convert screen coordinates to cells index
     */
    fn index(
        &self,
        x: usize, 
        y: usize,
    ) -> usize {
        self.width * y + x
    }
}