#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Default,
    Rgb(u8, u8, u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
}

impl Cell {
    pub const fn new(ch: char) -> Self {
        Self {
            ch,
            fg: Color::Default,
        }
    }

    pub const fn with_fg(mut self, fg: Color) -> Self {
        self.fg = fg;
        self
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::new(' ')
    }
}

/**
 * Frame buffer for renderer consisting of 
 * row-major vector of character cells
 * 
 * row 0: 0 1 2 3 4 ...
 * row 1: ...
 * row 2: ...
 */
#[derive(Debug, PartialEq, Eq)]
pub struct Buffer {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl Buffer {
    /// Approximate terminal cell width divided by cell height.
    pub const DEFAULT_CELL_ASPECT: f64 = 0.5;

    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![Cell::default(); width * height],
        }
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

    pub fn cell_aspect(&self) -> f64 {
        Self::DEFAULT_CELL_ASPECT
    }

    pub fn display_aspect(&self) -> f64 {
        self.aspect() * self.cell_aspect()
    }

    pub fn get(&self, x: usize, y: usize) -> Cell {
        self.cells[self.index(x, y)]
    }

    pub fn set(&mut self, x: isize, y: isize, cell: Cell) {
        if x < 0 || y < 0 {
            return;
        }

        let (x, y) = (x as usize, y as usize);

        if x >= self.width || y >= self.height {
            return;
        }

        let index = self.index(x, y);
        self.cells[index] = cell;
    }

    pub fn clear(&mut self) {
        self.cells.fill(Cell::default());
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
}
