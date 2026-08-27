use crate::{
    geometry::Point,
};
use super::{Buffer, Cell};
use super::{draw_line, draw_point};


pub struct PlotRenderer {
    pub point_cell: Cell,
    pub line_cell: Cell,
    pub axis_cell: Cell,
}

pub struct PlotViewport {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl PlotRenderer {
    /**
     * Project / render 2D points into buffer, fitting
     * into viewport dimensions
     */
    pub fn render(
        &self,
        points: &[Point],
        viewport: &PlotViewport,
        buffer: &mut Buffer,
    ) {
        // Project points onto buffer screen coords
        let projected: Vec<_> = points
            .iter()
            .filter_map(|&point| project(point, viewport, buffer))
            .collect();

        // Draw lines between points
        for pair in projected.windows(2) {
            let (x0, y0) = pair[0];
            let (x1, y1) = pair[1];

            draw_line(
                buffer,
                x0,
                y0,
                x1,
                y1,
                self.line_cell,
            );
        }

        // Set point cells in buffer
        // -- Write over line cells at vertices if filled
        for (x, y) in projected {
            draw_point(buffer, x, y, self.point_cell);
        }
    }
}

/**
 * Map 2D point coordinates to buffer coordinates
 */
fn project(
    point: Point,
    viewport: &PlotViewport,
    buffer: &Buffer,
) -> Option<(isize, isize)> {
    let x_range = viewport.x_max - viewport.x_min;
    let y_range = viewport.y_max - viewport.y_min;

    if x_range.abs() <= f64::EPSILON ||
    y_range.abs() <= f64::EPSILON
    {
        return None;
    }

    // Normalize point [0, 1] within viewport
    let x_norm = (point.x - viewport.x_min) / x_range;
    let y_norm = (point.y - viewport.y_min) / y_range;

    // Map normalized point to buffer screen coordinates
    // -- Buffer top-left (0, 0) -> bottom-right (W-1, H-1)
    let x_screen = x_norm * (buffer.width() - 1) as f64;
    let y_screen = (1.0 - y_norm) * (buffer.height() - 1) as f64;   // flip y for top->bottom

    Some((
        x_screen.round() as isize, 
        y_screen.round() as isize,
    ))
}