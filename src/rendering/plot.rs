use super::{BrailleBuffer, Buffer, Cell, Color};
use super::{draw_axes_2d, draw_axes_ticks, draw_border};
use crate::geometry::Point;

#[derive(Debug, Clone, Copy)]
pub struct PlotStyle {
    pub point: Cell,
    pub line: Cell,
    pub x_axis: Cell,
    pub y_axis: Cell,
    pub x_tick: Cell,
    pub y_tick: Cell,
    pub tick_color: Color,
    pub border_color: Color,
}

impl PlotStyle {
    pub const fn curve_color(mut self, color: Color) -> Self {
        self.point = self.point.with_fg(color);
        self.line = self.line.with_fg(color);
        self
    }

    pub const fn axis_color(mut self, color: Color) -> Self {
        self.x_axis = self.x_axis.with_fg(color);
        self.y_axis = self.y_axis.with_fg(color);
        self
    }

    pub const fn tick_color(mut self, color: Color) -> Self {
        self.x_tick = self.x_tick.with_fg(color);
        self.y_tick = self.y_tick.with_fg(color);
        self.tick_color = color;
        self
    }

    pub const fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }
}

impl Default for PlotStyle {
    fn default() -> Self {
        let axis_color = Color::Rgb(120, 120, 120);

        Self {
            point: Cell::new('@'),
            line: Cell::new('*'),
            x_axis: Cell::new('─').with_fg(axis_color),
            y_axis: Cell::new('│').with_fg(axis_color),
            x_tick: Cell::new('┼').with_fg(axis_color),
            y_tick: Cell::new('┼').with_fg(axis_color),
            tick_color: Color::Rgb(220, 220, 80),
            border_color: Color::Default,
        }
    }
}

pub struct PlotRenderer {
    pub style: PlotStyle,
    pub aspect: PlotAspect,
    pub pad_width: usize,
    pub pad_height: usize,
    pub show_axes: bool,
    pub show_ticks: bool,
    pub num_ticks: usize,
    pub show_border: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum PlotAspect {
    Auto,
    Equal { cell_aspect: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlotArea {
    pub left: isize,
    pub right: isize,
    pub top: isize,
    pub bottom: isize,
}

impl PlotArea {
    pub const fn width(self) -> isize {
        self.right - self.left
    }

    pub const fn height(self) -> isize {
        self.bottom - self.top
    }

    pub fn fit_equal_aspect(
        self,
        viewport: &PlotViewport,
        cell_aspect: f64,
    ) -> Self {
        let x_range = viewport.x_max - viewport.x_min;
        let y_range = viewport.y_max - viewport.y_min;

        let data_aspect = x_range / y_range;

        let area_aspect = self.width() as f64 * cell_aspect / self.height() as f64;

        if area_aspect > data_aspect {
            // Available area is too wide: reduce and center its width.
            let new_width =
                (self.height() as f64 * data_aspect / cell_aspect).round() as isize;

            let offset = (self.width() - new_width) / 2;

            PlotArea {
                left: self.left + offset,
                right: self.left + offset + new_width,
                ..self
            }
        } else {
            // Available area is too tall: reduce and center its height.
            let new_height =
                (self.width() as f64 * cell_aspect / data_aspect).round() as isize;

            let offset = (self.height() - new_height) / 2;

            PlotArea {
                top: self.top + offset,
                bottom: self.top + offset + new_height,
                ..self
            }
        }
    }
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
    pub fn render(&self, points: &[Point], viewport: &PlotViewport, buffer: &mut Buffer) {
        let Some(mut area) = self.plot_area(buffer) else {
            return;
        };

        if let PlotAspect::Equal { cell_aspect } = self.aspect {
            area = area.fit_equal_aspect(viewport, cell_aspect);
        }

        let mut braille = BrailleBuffer::new(buffer.width(), buffer.height());

        // Project points onto the 2x4 subpixel grid.
        let projected: Vec<_> = points
            .iter()
            .filter_map(|&point| self.project_braille(point, viewport, area))
            .collect();

        // Draw lines between points
        for pair in projected.windows(2) {
            let (x0, y0) = pair[0];
            let (x1, y1) = pair[1];

            braille.draw_line(x0, y0, x1, y1, self.style.line.fg);
        }

        // Set the sampled points over the Braille line.
        for (x, y) in projected {
            braille.set(x, y, self.style.point.fg);
        }

        braille.composite(buffer);

        // Draw x/y axes
        if self.show_axes {
            let origin = Point::new(0.0, 0.0);
            if let Some((origin_x, origin_y)) = self.project(origin, viewport, area) {
                draw_axes_2d(
                    buffer,
                    area,
                    origin_x,
                    origin_y,
                    self.style.x_axis,
                    self.style.y_axis,
                );
            }
        }

        // Draw x/y tick labels
        if self.show_ticks {
            let origin = Point::new(0.0, 0.0);
            if let Some((origin_x, origin_y)) = self.project(origin, viewport, area) {
                draw_axes_ticks(
                    buffer,
                    viewport,
                    area,
                    origin_x,
                    origin_y,
                    self.style.x_tick,
                    self.style.y_tick,
                    self.num_ticks,
                    self.style.tick_color,
                );
            }
        }

        // Draw border
        if self.show_border {
            draw_border(buffer, self.style.border_color);
        }
    }

    fn plot_area(&self, buffer: &Buffer) -> Option<PlotArea> {
        let width = isize::try_from(buffer.width()).ok()?;
        let height = isize::try_from(buffer.height()).ok()?;
        let pad_width = isize::try_from(self.pad_width).ok()?;
        let pad_height = isize::try_from(self.pad_height).ok()?;

        let area = PlotArea {
            left: pad_width,
            right: width.checked_sub(pad_width + 1)?,
            top: pad_height,
            bottom: height.checked_sub(pad_height + 1)?,
        };

        (area.width() >= 0 && area.height() >= 0).then_some(area)
    }

    /**
     * Map 2D point coordinates to buffer coordinates
     */
    fn project(
        &self,
        point: Point,
        viewport: &PlotViewport,
        area: PlotArea,
    ) -> Option<(isize, isize)> {
        let x_range = viewport.x_max - viewport.x_min;
        let y_range = viewport.y_max - viewport.y_min;

        if x_range.abs() <= f64::EPSILON || y_range.abs() <= f64::EPSILON {
            return None;
        }

        // Normalize point [0, 1] within viewport
        let x_norm = (point.x - viewport.x_min) / x_range;
        let y_norm = (point.y - viewport.y_min) / y_range;

        // Map normalized point into the shared padded plot area.
        let x_screen = area.left as f64 + x_norm * area.width() as f64;
        let y_screen = area.top as f64 + (1.0 - y_norm) * area.height() as f64;

        Some((x_screen.round() as isize, y_screen.round() as isize))
    }

    fn project_braille(
        &self,
        point: Point,
        viewport: &PlotViewport,
        area: PlotArea,
    ) -> Option<(isize, isize)> {
        let x_range = viewport.x_max - viewport.x_min;
        let y_range = viewport.y_max - viewport.y_min;

        if x_range.abs() <= f64::EPSILON || y_range.abs() <= f64::EPSILON {
            return None;
        }

        let x_norm = (point.x - viewport.x_min) / x_range;
        let y_norm = (point.y - viewport.y_min) / y_range;

        let left = area.left * 2;
        let top = area.top * 4;
        let width = (area.width() + 1) * 2 - 1;
        let height = (area.height() + 1) * 4 - 1;

        let x_screen = left as f64 + x_norm * width as f64;
        let y_screen = top as f64 + (1.0 - y_norm) * height as f64;

        Some((x_screen.round() as isize, y_screen.round() as isize))
    }
}

impl Default for PlotRenderer {
    fn default() -> Self {
        Self {
            style: PlotStyle::default(),
            aspect: PlotAspect::Auto,
            pad_width: 0,
            pad_height: 0,
            show_axes: true,
            show_ticks: true,
            num_ticks: 10,
            show_border: false,
        }
    }
}
