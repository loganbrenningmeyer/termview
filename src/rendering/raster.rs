use super::{Buffer, Cell};

pub fn draw_point(
    buffer: &mut Buffer,
    x: isize,
    y: isize,
    cell: Cell,
) {
    buffer.set(x, y, cell);
}

/**
 * Bresenham's line algorithm for efficiently drawing lines between
 * points in the terminal
 */
pub fn draw_line(
    buffer: &mut Buffer,
    x0: isize,
    y0: isize,
    x1: isize,
    y1: isize,
    cell: Cell,
) {
    let mut x0_ = x0;
    let mut y0_ = y0;

    let dx = (x1 - x0_).abs();
    let dy = (y1 - y0_).abs();

    let sx = if x0_ < x1 { 1 } else { -1 };
    let sy = if y0_ < y1 { 1 } else { -1 };

    let mut error = dx - dy;

    loop {
        buffer.set(x0_, y0_, cell);

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