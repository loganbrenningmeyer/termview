use super::{Buffer, Cell, Color, PlotArea2d, PlotViewport2d};

/**
 * Draws rectangular border around buffer edges
 */
pub fn draw_border(buffer: &mut Buffer, color: Color) {
    // Convert usize to isize safely
    let (Ok(width), Ok(height)) = (
        isize::try_from(buffer.width()),
        isize::try_from(buffer.height()),
    ) else {
        return;
    };

    // Vertical
    for y in 0..height {
        buffer.set(0, y, Cell::new('│').with_fg(color));
        buffer.set(width - 1, y, Cell::new('│').with_fg(color));
    }

    // Horizontal
    for x in 0..width {
        buffer.set(x, 0, Cell::new('─').with_fg(color));
        buffer.set(x, height - 1, Cell::new('─').with_fg(color));
    }

    // Corners
    buffer.set(0, 0, Cell::new('┌').with_fg(color));
    buffer.set(width - 1, 0, Cell::new('┐').with_fg(color));
    buffer.set(0, height - 1, Cell::new('└').with_fg(color));
    buffer.set(width - 1, height - 1, Cell::new('┘').with_fg(color));
}

/**
 * 
 */
fn draw_text_in_area(
    buffer: &mut Buffer,
    area: PlotArea2d,
    x: isize,
    y: isize,
    text: &str,
    color: Color,
) {
    if !(area.top..=area.bottom).contains(&y) {
        return;
    }

    for (offset, ch) in text.chars().enumerate() {
        let Ok(offset) = isize::try_from(offset) else {
            break;
        };
        let cell_x = x + offset;

        if (area.left..=area.right).contains(&cell_x) {
            buffer.set(cell_x, y, Cell::new(ch).with_fg(color));
        }
    }
}

/**
 * 
 */
pub fn draw_text(
    buffer: &mut Buffer,
    x: isize,
    y: isize,
    text: &str,
    color: Color,
) {
    for (offset, ch) in text.chars().enumerate() {
        let Ok(offset) = isize::try_from(offset) else {
            break;
        };

        buffer.set(
            x + offset,
            y,
            Cell { ch, fg: color },
        );
    }
}

/**
 * 
 */
pub fn draw_axes_ticks(
    buffer: &mut Buffer,
    viewport: &PlotViewport2d,
    area: PlotArea2d,
    origin_x: isize,
    origin_y: isize,
    tick_x: Cell,
    tick_y: Cell,
    num_ticks: usize,
    color: Color,
) {
    if num_ticks < 2 {
        return;
    }

    let x_range = viewport.x_max - viewport.x_min;
    let y_range = viewport.y_max - viewport.y_min;

    // X-ticks
    for i in 0..num_ticks {
        let t = i as f64 / (num_ticks - 1) as f64;

        let x_col = area.left + (t * area.width() as f64).round() as isize;

        let value = viewport.x_min + t * x_range;
        let label = format!("{value:.2}");

        // Center the label beneath its tick
        let label_width = isize::try_from(label.chars().count()).unwrap_or(0);

        let max_label_x = (area.right - label_width + 1).max(area.left);
        let label_x = (x_col - label_width / 2).clamp(area.left, max_label_x);
        let label_y = (origin_y + 1).clamp(area.top, area.bottom);

        draw_text_in_area(buffer, area, label_x, label_y, &label, color);

        buffer.set(x_col, origin_y, tick_x);
    }

    // Y-ticks
    for i in 0..num_ticks {
        let t = i as f64 / (num_ticks - 1) as f64;

        let y_row = area.top + (t * area.height() as f64).round() as isize;

        let value = viewport.y_max - t * y_range;
        let label = format!("{value:.2}");

        // Put to the left of the axis
        let label_width = isize::try_from(label.chars().count()).unwrap_or(0);
        let max_label_x = (area.right - label_width + 1).max(area.left);
        let label_x = (origin_x - label_width - 1).clamp(area.left, max_label_x);

        draw_text_in_area(buffer, area, label_x, y_row, &label, color);

        buffer.set(origin_x, y_row, tick_y);
    }

    // Draw origin label
    let origin_label_x = (origin_x - 1).clamp(area.left, area.right);
    let origin_label_y = (origin_y + 1).clamp(area.top, area.bottom);
    buffer.set(
        origin_label_x,
        origin_label_y,
        Cell::new('0').with_fg(color),
    );
}

/**
 * 
 */
pub fn draw_axes_2d(
    buffer: &mut Buffer,
    area: PlotArea2d,
    origin_x: isize,
    origin_y: isize,
    cell_x: Cell,
    cell_y: Cell,
) {
    if (area.top..=area.bottom).contains(&origin_y) {
        for x in area.left..=area.right {
            buffer.set(x, origin_y, cell_x);
        }
    }

    if (area.left..=area.right).contains(&origin_x) {
        for y in area.top..=area.bottom {
            buffer.set(origin_x, y, cell_y);
        }
    }
}

/**
 * 
 */
pub fn draw_point(buffer: &mut Buffer, x: isize, y: isize, cell: Cell) {
    buffer.set(x, y, cell);
}

/**
 * Bresenham's line algorithm for efficiently drawing lines between
 * points in the terminal
 */
pub fn draw_line(buffer: &mut Buffer, x0: isize, y0: isize, x1: isize, y1: isize, cell: Cell) {
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
