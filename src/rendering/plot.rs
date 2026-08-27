use crate::{
    geometry::Point,
};
use super::{Buffer, Cell, Color};
use super::{
    draw_axes_2d, 
    draw_axes_ticks, 
    draw_border,
    draw_line, 
    draw_point,
};


pub struct PlotRenderer {
    pub point_cell: Cell,
    pub line_cell: Cell,
    pub x_axis_cell: Cell,
    pub y_axis_cell: Cell,
    pub x_tick_cell: Cell,
    pub y_tick_cell: Cell,
    pub pad_width: usize,
    pub pad_height: usize,
    pub show_axes: bool,
    pub show_ticks: bool,
    pub num_ticks: usize,
    pub ticks_color: Color,
    pub show_border: bool,
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
            .filter_map(|&point| self.project(point, viewport, buffer))
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

        // Draw x/y axes
        if self.show_axes {
            let origin = Point::new(0.0, 0.0);
            if let Some((origin_x, origin_y)) = self.project(
                origin,
                viewport,
                buffer,
            ) {
                draw_axes_2d(
                    buffer, 
                    origin_x, 
                    origin_y, 
                    self.x_axis_cell, 
                    self.y_axis_cell,
                );
            }
        }

        // Draw x/y tick labels
        if self.show_ticks {
            let origin = Point::new(0.0, 0.0);
            if let Some((origin_x, origin_y)) = self.project(
                origin,
                viewport,
                buffer,
            ) {
                draw_axes_ticks(
                    buffer, 
                    viewport,
                    origin_x, 
                    origin_y, 
                    self.y_axis_cell,
                    self.x_axis_cell,
                    self.num_ticks,
                    self.ticks_color,
                );
            }
        }

        // Draw border
        if self.show_border {
            draw_border(buffer);
        }
    }

    /**
     * Map 2D point coordinates to buffer coordinates
     */
    fn project(
        &self,
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
        // -- top-left (0 + p_w, 0 + p_h) 
        //      -> bottom-right (W - p_w - 1, H - p_h - 1)
        // -- width  -= 2 * p_w
        // -- height -= 2 * p_h
        let plot_width  = buffer.width()
            .checked_sub(self.pad_width.checked_mul(2)?)?;
        let plot_height = buffer.height()
            .checked_sub(self.pad_height.checked_mul(2)?)?;

        let x_span = plot_width.checked_sub(1)? as f64;
        let y_span = plot_height.checked_sub(1)? as f64;

        let x_screen = self.pad_width as f64
            + x_norm * x_span;
        let y_screen = self.pad_height as f64
            + (1.0 - y_norm) * y_span;   // flip y for top->bottom

        Some((
            x_screen.round() as isize, 
            y_screen.round() as isize,
        ))
    }
}

impl Default for PlotRenderer {
    fn default() -> Self {
        PlotRenderer {
            point_cell: Cell::new('@'),
            line_cell: Cell::new('*'),
            x_axis_cell: Cell::new('─').with_fg(Color::Rgb(120, 120, 120)),
            y_axis_cell: Cell::new('│').with_fg(Color::Rgb(120, 120, 120)),
            x_tick_cell: Cell::new('│').with_fg(Color::Rgb(120, 120, 120)),
            y_tick_cell: Cell::new('─').with_fg(Color::Rgb(120, 120, 120)),
            pad_width: 0,
            pad_height: 0,
            show_axes: true,
            show_ticks: true,
            num_ticks: 10,
            ticks_color: Color::Rgb(220, 220, 80),
            show_border: true,
        }
    }
}