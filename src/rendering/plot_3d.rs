use crate::{
    geometry::Object,
    rendering::{draw_border, Axes3dStyle, Cell, Color, WireframeStyle},
};
use super::{Axes3dRenderer, Buffer, Camera, WireframeRenderer};


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
pub struct Plot3dStyle {
    pub surface: WireframeStyle,
    pub axes: Axes3dStyle,
    pub border: Color,
}

pub struct Plot3dRenderer {
    pub surface_renderer: WireframeRenderer,
    pub axes_renderer: Axes3dRenderer,
    pub style: Plot3dStyle,
    pub show_axes: bool,
    pub show_border: bool,
}

impl Default for PlotViewport3d {
    fn default() -> Self {
        Self {
            x_min: -3.0,
            x_max: 3.0,
            y_min: -3.0,
            y_max: 3.0,
            z_min: -2.0,
            z_max: 2.0,
        }
    }
}

impl Default for Plot3dStyle {
    fn default() -> Self {
        Self {
            surface: WireframeStyle {
                edge: Cell::new('·').with_fg(Color::Rgb(120, 200, 220)),
                vertex: Cell::new('·').with_fg(Color::Rgb(160, 220, 235)),
            },
            axes: Axes3dStyle::default(),
            border: Color::Rgb(160, 160, 160),
        }
    }
}

impl Default for Plot3dRenderer {
    fn default() -> Self {
        Self {
            surface_renderer: WireframeRenderer,
            axes_renderer: Axes3dRenderer::default(),
            style: Plot3dStyle::default(),
            show_axes: true,
            show_border: false,
        }
    }
}

impl Plot3dRenderer {
    pub fn render(
        &self,
        surface: &Object,
        viewport: &PlotViewport3d,
        camera: &Camera,
        buffer: &mut Buffer,
    ) {
        self.surface_renderer.render(
            std::slice::from_ref(surface),
            camera,
            self.style.surface,
            buffer,
        );

        if self.show_axes {
            self.axes_renderer.render(
                &self.surface_renderer,
                viewport,
                surface.transform,
                camera,
                self.style.axes,
                buffer,
            );
        }

        if self.show_border {
            draw_border(buffer, self.style.border);
        }
    }
}
