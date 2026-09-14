use super::{Buffer, Cell, Color, PlotArea2d};
use crate::repl::{
    HORIZONTAL,
    VERTICAL,
    TOP_LEFT, TOP_LEFT_RD,
    TOP_RIGHT, TOP_RIGHT_RD,
    BOTTOM_LEFT, BOTTOM_LEFT_RD,
    BOTTOM_RIGHT, BOTTOM_RIGHT_RD,
};

/**
 * Draws rectangular border around buffer edges
 */
pub fn draw_border(buffer: &mut Buffer, color: Color, round: bool) {
    // Convert usize to isize safely
    let (Ok(width), Ok(height)) = (
        isize::try_from(buffer.width()),
        isize::try_from(buffer.height()),
    ) else {
        return;
    };

    // Vertical
    for y in 0..height {
        buffer.set(0, y, Cell::new(VERTICAL).with_fg(color));
        buffer.set(width - 1, y, Cell::new(VERTICAL).with_fg(color));
    }

    // Horizontal
    for x in 0..width {
        buffer.set(x, 0, Cell::new(HORIZONTAL).with_fg(color));
        buffer.set(x, height - 1, Cell::new(HORIZONTAL).with_fg(color));
    }

    // Corners
    let top_left = if round { TOP_LEFT_RD } else { TOP_LEFT };
    let top_right = if round { TOP_RIGHT_RD } else { TOP_RIGHT };
    let bottom_left = if round { BOTTOM_LEFT_RD } else { BOTTOM_LEFT };
    let bottom_right = if round { BOTTOM_RIGHT_RD } else { BOTTOM_RIGHT };

    buffer.set(0, 0, Cell::new(top_left).with_fg(color));
    buffer.set(width - 1, 0, Cell::new(top_right).with_fg(color));
    buffer.set(0, height - 1, Cell::new(bottom_left).with_fg(color));
    buffer.set(width - 1, height - 1, Cell::new(bottom_right).with_fg(color));
}

/**
 * 
 */
pub fn draw_text_in_area(
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
 * Draws multiple lines of text in a block
 */
pub fn draw_text_block(
    buffer: &mut Buffer,
    x: isize,
    y: isize,
    text: Vec<String>,
    color: Color,
    border: bool,
    title: &str,
    tag: &str,
) {
    // Determine block width / height
    let Some(width) = text
        .iter()
        .map(|line| line.chars().count())
        .max()
    else {
        return;
    };
    let width_i = width as isize;
    let height_i = text.len() as isize;

    // Prepare border offset
    let (x_new, y_new) = if border {
        (x + 1, y + 1)
    } else {
        (x, y)
    };

    // Define the text area bounds
    let text_area = PlotArea2d {
        left: x_new,
        right: x_new + width_i - 1,
        top: y_new,
        bottom: y_new + height_i - 1,
    };

    // Write each line in
    for (i, line) in text.iter().enumerate() {
        let line_padded = format!("{:<width$}", line, width = width);

        draw_text(
            buffer,
            text_area.left,
            text_area.top + i as isize,
            &line_padded,
            color,
            false,
        );
    }

    // Draw outer border
    if border {
        draw_border_area(buffer, text_area, color);
    }

    // Draw title at top left
    if !title.is_empty() {
        draw_text(
            buffer,
            text_area.left + 1,
            text_area.top - 1,
            title,
            color,
            false,
        );
    }

    // Draw tag at top right
    if !tag.is_empty() {
        draw_text(
            buffer,
            text_area.right - tag.chars().count() as isize,
            text_area.top - 1,
            tag,
            color,
            false,
        )
    }
}


pub fn draw_border_area(
    buffer: &mut Buffer,
    area: PlotArea2d,
    color: Color,
) {
    let width = area.right - area.left;
    let height = area.bottom - area.top;

    for x_offset in 0..=width {
        for y_offset in 0..=height {
            let x = area.left + x_offset;
            let y = area.top + y_offset;

            // -------------------------
            // Corners
            // -------------------------
            // Top-left
            if x_offset == 0 && y_offset == 0 
            {
                buffer.set(area.left - 1, area.top - 1, Cell { ch: TOP_LEFT_RD, fg: color });
            } 
            // Bottom-left
            if x_offset == 0 && y_offset == height
            {
                buffer.set(area.left - 1, area.bottom + 1, Cell { ch: BOTTOM_LEFT_RD, fg: color });
            }
            // Top-right
            if x_offset == width && y_offset == 0 
            {
                buffer.set(area.right + 1, area.top - 1, Cell { ch: TOP_RIGHT_RD, fg: color });
            }
            // Bottom-right
            if x_offset == width && y_offset == height 
            {
                buffer.set(area.right + 1, area.bottom + 1, Cell { ch: BOTTOM_RIGHT_RD, fg: color });
            }

            // -------------------------
            // Edges
            // -------------------------
            // Left & right
            if x_offset == 0
            {
                buffer.set(area.left - 1, y, Cell { ch: VERTICAL, fg: color });
            }
            if x_offset == width 
            {
                buffer.set(area.right + 1, y, Cell { ch: VERTICAL, fg: color });
            }
            // Top & bottom
            if y_offset == 0
            {
                buffer.set(x, area.top - 1, Cell { ch: HORIZONTAL, fg: color });
            }
            if y_offset == height
            {
                buffer.set(x, area.bottom + 1, Cell { ch: HORIZONTAL, fg: color });
            }
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
    border: bool,
) {
    // Prepare border offset
    let (x_new, y_new) = if border {
        (x + 1, y + 1)
    } else {
        (x, y)
    };

    for (offset, ch) in text.chars().enumerate() {
        let Ok(offset) = isize::try_from(offset) else {
            break;
        };

        if border {
            // Left / bottom-left / top-left borders
            if offset == 0 {
                buffer.set(x_new - 1, y_new, Cell { ch: VERTICAL, fg: color });
                buffer.set(x_new - 1, y_new - 1, Cell { ch: TOP_LEFT_RD, fg: color });
                buffer.set(x_new - 1, y_new + 1, Cell { ch: BOTTOM_LEFT_RD, fg: color });
            // Right / bottom-right / top-right borders
            } else if offset == text.chars().count() as isize - 1 {
                buffer.set(x_new + offset + 1, y_new, Cell { ch: VERTICAL, fg: color });
                buffer.set(x_new + offset + 1, y_new - 1, Cell { ch: TOP_RIGHT_RD, fg: color });
                buffer.set(x_new + offset + 1, y_new + 1, Cell { ch: BOTTOM_RIGHT_RD, fg: color });
            }
            // Top / bottom horizontal
            buffer.set(x_new + offset, y_new - 1, Cell { ch: HORIZONTAL, fg: color });
            buffer.set(x_new + offset, y_new + 1, Cell { ch: HORIZONTAL, fg: color });
        } 

        buffer.set(
            x_new + offset,
            y_new,
            Cell { ch, fg: color },
        );
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
