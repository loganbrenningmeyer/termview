use super::{BrailleBuffer, Buffer, Cell, Color};
use super::{draw_axes_2d, draw_axes_ticks, draw_border};
use crate::geometry::Point;
use crate::math::rangef;


#[derive(Debug, Clone, Copy)]
pub struct PlotStyle2d {
    pub point: Cell,
    pub line: Cell,
    pub x_axis: Cell,
    pub y_axis: Cell,
    pub x_tick: Cell,
    pub y_tick: Cell,
    pub tick_color: Color,
    pub border_color: Color,
    pub guide_color: Color,
}

impl PlotStyle2d {
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

impl Default for PlotStyle2d {
    fn default() -> Self {
        Self {
            point: Cell::new('@').with_fg(Color::VERTEX),
            line: Cell::new('*').with_fg(Color::EDGE),
            x_axis: Cell::new('─').with_fg(Color::GRAY),
            y_axis: Cell::new('│').with_fg(Color::GRAY),
            x_tick: Cell::new('┬').with_fg(Color::GRAY),
            y_tick: Cell::new('┤').with_fg(Color::GRAY),
            tick_color: Color::WHITE,
            border_color: Color::Default,
            guide_color: Color::LIGHT_GRAY,
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub enum PlotAspect2d {
    Auto,
    Equal { cell_aspect: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlotArea2d {
    pub left: isize,
    pub right: isize,
    pub top: isize,
    pub bottom: isize,
}

impl PlotArea2d {
    pub const fn width(self) -> isize {
        self.right - self.left
    }

    pub const fn height(self) -> isize {
        self.bottom - self.top
    }

    pub fn fit_equal_aspect(
        self,
        viewport: &PlotViewport2d,
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

            PlotArea2d {
                left: self.left + offset,
                right: self.left + offset + new_width,
                ..self
            }
        } else {
            // Available area is too tall: reduce and center its height.
            let new_height =
                (self.width() as f64 * cell_aspect / data_aspect).round() as isize;

            let offset = (self.height() - new_height) / 2;

            PlotArea2d {
                top: self.top + offset,
                bottom: self.top + offset + new_height,
                ..self
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PlotViewport2d {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl Default for PlotViewport2d {
    fn default() -> Self {
        Self {
            x_min: -10.0,
            x_max: 10.0,
            y_min: -10.0,
            y_max: 10.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlotLayout2d {
    pub data: PlotArea2d,   // Only curves go here.
    pub x_axis_y: isize,
    pub y_axis_x: isize,
}


#[derive(Debug, Clone)]
pub struct PlotRenderer2d {
    pub style: PlotStyle2d,
    pub aspect: PlotAspect2d,
    pub pad_width: usize,
    pub pad_height: usize,
    pub show_axes: bool,
    pub show_ticks: bool,
    pub num_ticks: usize,
    pub show_border: bool,
}

impl PlotRenderer2d {
    /**
     * Project / render 2D points into buffer, fitting
     * into viewport dimensions
     */
    pub fn render(
        &self, 
        points: &[Point], 
        viewport: &PlotViewport2d, 
        buffer: &mut Buffer
    ) {
        let Some(outer) = self.plot_area(buffer) else {
            return;
        };

        // -------------------------
        // Define plot area for axes / datapoints
        // -------------------------
        let max_y_label_width = self.max_y_label_width(viewport);

        let mut data_area = PlotArea2d {
            left: outer.left + max_y_label_width + 2,
            right: outer.right,
            top: outer.top,
            bottom: outer.bottom - 2,
        };

        // Guard against too small plot areas
        if data_area.width() <= 0 || data_area.height() <= 0 {
            return;
        }

        if let PlotAspect2d::Equal { cell_aspect } = self.aspect {
            data_area = data_area.fit_equal_aspect(viewport, cell_aspect);
        }

        let layout = PlotLayout2d {
            x_axis_y: data_area.bottom + 1,
            y_axis_x: data_area.left - 1,
            data: data_area,
        };

        // -------------------------
        // Render points in 2x4 braille subpixel grid
        // -------------------------
        let mut braille = BrailleBuffer::new(buffer.width(), buffer.height());

        // Project points onto braille buffer
        let projected: Vec<(isize, isize)> = points
            .iter()
            .filter_map(|&point| {
                self.project_braille(point, viewport, data_area)
            })
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

        // Draw x/y axes
        if self.show_axes {
            draw_axes_2d(
                buffer,
                &layout,
                self.style.x_axis,
                self.style.y_axis,
            );
        }

        // Draw x/y tick labels
        if self.show_ticks {
            draw_axes_ticks(
                buffer,
                viewport,
                &layout,
                outer,
                self.style.x_tick,
                self.style.y_tick,
                self.num_ticks,
                self.style.tick_color,
            );
        }

        // Draw zero guidelines
        if self.show_axes {
            self.draw_zero_lines(buffer, viewport, data_area);
        }

        // Composite curve sampled points onto buffer
        braille.composite(buffer);

        // Draw border
        if self.show_border {
            draw_border(buffer, self.style.border_color, true);
        }
    }

    fn plot_area(&self, buffer: &Buffer) -> Option<PlotArea2d> {
        let width = isize::try_from(buffer.width()).ok()?;
        let height = isize::try_from(buffer.height()).ok()?;
        let pad_width = isize::try_from(self.pad_width).ok()?;
        let pad_height = isize::try_from(self.pad_height).ok()?;

        let area = PlotArea2d {
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
        viewport: &PlotViewport2d,
        area: PlotArea2d,
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
        viewport: &PlotViewport2d,
        area: PlotArea2d,
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

    /**
     * Draw dotted lines through the origin onto the target buffer
     */
    fn draw_zero_lines(
        &self,
        buffer: &mut Buffer,
        viewport: &PlotViewport2d,
        area: PlotArea2d,
    ) {
        // Ignore if origin is offscreen
        let Some((zero_x, zero_y)) =
            self.project_braille(Point::new(0.0, 0.0), viewport, area)
        else {
            return;
        };

        // Create zero lines onto new braille buffer of same size as target
        let mut guides = BrailleBuffer::new(
            buffer.width(), 
            buffer.height()
        );

        // Convert plot area to braille
        let left = area.left * 2;
        let right = (area.right + 1) * 2 - 1;
        let top = area.top * 4;
        let bottom = (area.bottom + 1) * 4 - 1;

        // Horizontal zero-guide (y=0)
        if viewport.y_min <= 0.0 && viewport.y_max >= 0.0 {
            for x in (left..=right).step_by(4) {
                guides.set(x, zero_y, Color::LIGHT_GRAY);
            }
        }

        // Vertical zero-guide (x=0)
        if viewport.x_min <= 0.0 && viewport.x_max >= 0.0 {
            for y in (top..=bottom).step_by(4) {
                guides.set(zero_x, y, Color::LIGHT_GRAY);
            }
        }

        // Composite guides buffer onto target buffer
        guides.composite(buffer);
    }

    /**
     * Get widest y-axis label width given the viewport/renderer settings
     */
    fn max_y_label_width(&self, viewport: &PlotViewport2d) -> isize {
        if self.show_ticks && self.num_ticks >= 2 {
            let ticks = rangef(
                viewport.y_min, 
                viewport.y_max, 
                self.num_ticks, 
                true
            );

            ticks
                .iter()
                .map(|val| {
                    format!("{val:.2}").chars().count()
                })
                .max()
                .unwrap_or(0) as isize
        } else {
            0
        }
    }
}

impl Default for PlotRenderer2d {
    fn default() -> Self {
        Self {
            style: PlotStyle2d::default(),
            aspect: PlotAspect2d::Auto,
            pad_width: 2,
            pad_height: 2,
            show_axes: true,
            show_ticks: true,
            num_ticks: 6,
            show_border: false,
        }
    }
}
