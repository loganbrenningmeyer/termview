use super::{Buffer, Cell, Color};

/**
 * Braille cells are 2x4 grid of dots
 * -- Completed mask for the character is
 *    created by adding the corresponding bit
 *    to U+2800 (the blank braille character)
 */
const DOT_BITS: [[u8; 2]; 4] = [
    [0x01, 0x08],
    [0x02, 0x10],
    [0x04, 0x20],
    [0x40, 0x80],
];

#[derive(Clone, Copy)]
struct BrailleCell {
    dots: u8,
    color: Color,
    depth: f64,
    layer: u8,
}

impl Default for BrailleCell {
    fn default() -> Self {
        Self {
            dots: 0,
            color: Color::Default,
            depth: f64::INFINITY,
            layer: 0,
        }
    }
}

pub struct BrailleBuffer {
    cell_width: usize,
    cell_height: usize,
    cells: Vec<BrailleCell>,
}

impl BrailleBuffer {
    pub fn new(cell_width: usize, cell_height: usize) -> Self {
        Self {
            cell_width,
            cell_height,
            cells: vec![BrailleCell::default(); cell_width * cell_height],
        }
    }

    pub fn width(&self) -> usize {
        self.cell_width * 2
    }

    pub fn height(&self) -> usize {
        self.cell_height * 4
    }

    pub fn set(&mut self, x: isize, y: isize, color: Color) {
        self.set_depth(x, y, 0.0, 0, color);
    }

    pub fn set_depth(
        &mut self,
        x: isize,
        y: isize,
        depth: f64,
        layer: u8,
        color: Color,
    ) {
        if x < 0 || y < 0 {
            return;
        }

        let (x, y) = (x as usize, y as usize);

        if x >= self.width() || y >= self.height() {
            return;
        }

        let cell_x = x / 2;
        let cell_y = y / 4;
        let dot_x = x % 2;
        let dot_y = y % 4;
        let index = cell_y * self.cell_width + cell_x;
        let dot = DOT_BITS[dot_y][dot_x];
        let cell = &mut self.cells[index];

        if cell.dots == 0 {
            *cell = BrailleCell {
                dots: dot,
                color,
                depth,
                layer,
            };
        } else if cell.layer == layer {
            cell.dots |= dot;

            if depth <= cell.depth {
                cell.depth = depth;
                cell.color = color;
            }
        } else if depth <= cell.depth {
            *cell = BrailleCell {
                dots: dot,
                color,
                depth,
                layer,
            };
        }
    }

    pub fn draw_line(
        &mut self,
        x0: isize,
        y0: isize,
        x1: isize,
        y1: isize,
        color: Color,
    ) {
        self.draw_line_depth(x0, y0, 0.0, x1, y1, 0.0, 0, color);
    }

    pub fn draw_line_depth(
        &mut self,
        x0: isize,
        y0: isize,
        depth0: f64,
        x1: isize,
        y1: isize,
        depth1: f64,
        layer: u8,
        color: Color,
    ) {
        let (mut x, mut y) = (x0, y0);
        let dx = (x1 - x).abs();
        let dy = (y1 - y).abs();
        let steps = dx.max(dy);
        let step_x = if x < x1 { 1 } else { -1 };
        let step_y = if y < y1 { 1 } else { -1 };
        let mut error = dx - dy;
        let mut step = 0;

        loop {
            let t = if steps == 0 {
                0.0
            } else {
                step as f64 / steps as f64
            };
            let depth = depth0 + (depth1 - depth0) * t;

            self.set_depth(x, y, depth, layer, color);

            if x == x1 && y == y1 {
                break;
            }

            let error2 = 2 * error;

            if error2 > -dy {
                error -= dy;
                x += step_x;
            }

            if error2 < dx {
                error += dx;
                y += step_y;
            }

            step += 1;
        }
    }

    pub fn composite(&self, buffer: &mut Buffer) {
        for y in 0..self.cell_height {
            for x in 0..self.cell_width {
                let cell = self.cells[y * self.cell_width + x];

                if cell.dots == 0 {
                    continue;
                }

                // The cell already contains the nearest layer selected by
                // set_depth(). Do not merge it with an older output glyph,
                // since a terminal cell has only one foreground color.
                let ch = char::from_u32(0x2800 + cell.dots as u32)
                    .expect("a Braille dot mask is always a valid character");

                buffer.set(
                    x as isize,
                    y as isize,
                    Cell::new(ch).with_fg(cell.color),
                );
            }
        }
    }
}
