use crate::{
    math::Transform,
    geometry::Mesh,
    rendering::{draw_border, AxesStyle3d, BrailleBuffer, Cell, Color, WireframeStyle},
};
use super::{AxesRenderer3d, Buffer, Camera, WireframeRenderer};


#[derive(Debug, Clone, Copy)]
pub struct PlotViewport3d {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub z_min: f64,
    pub z_max: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct PlotStyle3d {
    pub surface: WireframeStyle,
    pub axes: AxesStyle3d,
    pub border: Color,
}

#[derive(Debug, Clone)]
pub struct PlotRenderer3d {
    pub surface_renderer: WireframeRenderer,
    pub axes_renderer: AxesRenderer3d,
    pub style: PlotStyle3d,
    pub show_axes: bool,
    pub show_border: bool,
}

impl Default for PlotViewport3d {
    fn default() -> Self {
        Self {
            x_min: -10.0,
            x_max: 10.0,
            y_min: -10.0,
            y_max: 10.0,
            z_min: -10.0,
            z_max: 10.0,
        }
    }
}

impl Default for PlotStyle3d {
    fn default() -> Self {
        Self {
            surface: WireframeStyle {
                edge: Cell::new('•').with_fg(Color::EDGE),
                vertex: Cell::new('●').with_fg(Color::VERTEX),
            },
            axes: AxesStyle3d::default(),
            border: Color::GRAY,
        }
    }
}

impl Default for PlotRenderer3d {
    fn default() -> Self {
        Self {
            surface_renderer: WireframeRenderer,
            axes_renderer: AxesRenderer3d::default(),
            style: PlotStyle3d::default(),
            show_axes: true,
            show_border: true,
        }
    }
}

impl PlotRenderer3d {
    pub fn render(
        &self,
        mesh: &Mesh,
        transform: Transform,
        viewport: &PlotViewport3d,
        camera: &Camera,
        buffer: &mut Buffer,
    ) {
        let display_aspect = buffer.display_aspect();
        let mut braille = BrailleBuffer::new(
            buffer.width(), 
            buffer.height()
        );

        self.surface_renderer.render_braille_into(
            mesh,
            transform,
            camera,
            self.style.surface,
            display_aspect,
            0,
            &mut braille,
        );

        // Get surface-occupied spans in each row
        let surface_spans = braille.row_spans();

        if self.show_axes {
            self.axes_renderer.render_lines_braille(
                &self.surface_renderer,
                viewport,
                transform,
                camera,
                self.style.axes,
                display_aspect,
                &mut braille,
            );
        }

        if self.show_axes {
            // Build annotations buffer with all annotations
            let mut annotations = Buffer::new(
                buffer.width(),
                buffer.height(),
            );

            self.axes_renderer.render_annotations(
                &self.surface_renderer,
                viewport,
                transform,
                camera,
                self.style.axes,
                &mut annotations,
            );

            // Erase all annotations behind the surface spans
            for (y, span) in surface_spans.iter().enumerate() {
                for x in 0..buffer.width() {
                    // Determine if x-cell falls within the surface row span
                    // if so, don't render the annotation
                    let covered = match span {
                        Some((left, right)) => x >= *left && x <= *right,
                        None => false,
                    };

                    if covered {
                        continue;
                    }

                    let cell = annotations.get(x, y);

                    // Only set uncovered, non-empty annotation cells
                    if cell.ch != ' ' {
                        buffer.set(x as isize, y as isize, cell);
                    }
                }
            }
        }

        braille.composite(buffer);

        if self.show_border {
            draw_border(buffer, self.style.border, true);
        }
    }
}
