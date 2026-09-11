use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    geometry::{Object, Point},
    math::{Projection, OrthographicProjection},
    rendering::{
        Buffer,
        Camera,
        CameraOrbit,
        Cell,
        PlotViewport2d,
        PlotViewport3d,
        PlotRenderer2d,
        PlotRenderer3d,
    }
};


/**
 * 
 */
#[derive(Debug, PartialEq)]
pub enum KeyResult {
    Ignored,
    Changed,
    Exit,
}

pub trait Widget {
    fn render(&self, target: &mut Buffer);
    fn handle_key(&mut self, key: KeyEvent) -> KeyResult;
}

pub struct PlotWidget3d {
    pub surface: Object,
    pub camera: Camera,
    pub viewport: PlotViewport3d,
    pub renderer: PlotRenderer3d,
}

impl PlotWidget3d {
    pub fn new(
        surface: Object,
        camera: Camera,
        viewport: PlotViewport3d,
        renderer: PlotRenderer3d,
    ) -> Self {
        Self { surface, camera, viewport, renderer }
    }

    pub fn orbit_camera(
        &mut self,
        azimuth_delta: f64,
        elevation_delta: f64,
    ) {
        self.camera.orbit.azimuth += azimuth_delta;
        self.camera.orbit.elevation =
            (self.camera.orbit.elevation + elevation_delta).clamp(-85.0, 85.0);

        self.camera.update();
    }

    pub fn zoom_camera(
        &mut self,
        distance_delta: f64,
    ) {
        match &self.camera.projection {
            Projection::Perspective(p) => {
                self.camera.orbit.distance = 
                    (self.camera.orbit.distance + distance_delta).clamp(p.near, f64::MAX);

                self.camera.update();
            }
            // -------------------------
            // Orthographic does not change camera distance as
            // there is no division by distance; near and far are 
            // the same size
            // - Instead adjusts the size of the rendering plane
            // -------------------------
            Projection::Orthographic(o) => {
                self.camera.projection = Projection::Orthographic(
                    OrthographicProjection {
                        size: (o.size + distance_delta / 2.0).max(0.005),    // half because size on both sides
                        near: o.near,
                        far: o.far,
                    }
                )
            }
        }
    }

    pub fn reset_camera(&mut self) {
        self.camera.projection = match &self.camera.projection {
            Projection::Perspective(_) => Projection::Perspective(Default::default()),
            Projection::Orthographic(_) => Projection::Orthographic(Default::default()),
        };
        self.camera.orbit = CameraOrbit::default();

        self.camera.update();
    }
}

impl Widget for PlotWidget3d {
    fn render(&self, target: &mut Buffer) {
        self.renderer.render(
            &self.surface, 
            &self.viewport, 
            &self.camera, 
            target
        );
    }

    fn handle_key(&mut self, key: KeyEvent) -> KeyResult {
        match key.code {
            // -------------------------
            // Exit
            // -------------------------
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('x') => KeyResult::Exit,

            // -------------------------
            // Camera Orbit
            // -------------------------
            KeyCode::Left | KeyCode::Char('a') => {
                self.orbit_camera(-5.0, 0.0);
                KeyResult::Changed
            }
            KeyCode::Right | KeyCode::Char('d') => {
                self.orbit_camera(5.0, 0.0);
                KeyResult::Changed
            }
            KeyCode::Up | KeyCode::Char('w') => {
                self.orbit_camera(0.0, 5.0);
                KeyResult::Changed
            }
            KeyCode::Down | KeyCode::Char('s') => {
                self.orbit_camera(0.0, -5.0);
                KeyResult::Changed
            }

            // -------------------------
            // Camera Zoom
            // -------------------------
            // Zoom in
            KeyCode::Char('e') => {
                self.zoom_camera(-1.0);
                KeyResult::Changed
            }
            // Zoom out
            KeyCode::Char('q') => {
                self.zoom_camera(1.0);
                KeyResult::Changed
            }

            // -------------------------
            // Reset camera to default
            // -------------------------
            KeyCode::Char('r') => {
                self.reset_camera();
                KeyResult::Changed
            }

            _ => KeyResult::Ignored,
        }
    }
}

pub struct PlotWidget2d {
    pub points: Vec<Point>,
    pub viewport: PlotViewport2d,
    pub renderer: PlotRenderer2d,
}

impl PlotWidget2d {
    pub fn new(
        points: Vec<Point>,
        viewport: PlotViewport2d,
        renderer: PlotRenderer2d,
    ) -> Self {
        Self { points, viewport, renderer }
    }

    /**
     * Zoom 2D plot by resizing the viewport, but do 
     * not let the viewport go below MIN_HALF_SIZE
     */
    pub fn zoom(&mut self, distance_delta: f64) {
        const MIN_HALF_SIZE: f64 = 0.005;

        let x_center = (self.viewport.x_min + self.viewport.x_max) / 2.0;
        let y_center = (self.viewport.y_min + self.viewport.y_max) / 2.0;

        let x_half = (
            (self.viewport.x_max - self.viewport.x_min) / 2.0
            + distance_delta
        ).max(MIN_HALF_SIZE);

        let y_half = (
            (self.viewport.y_max - self.viewport.y_min) / 2.0
            + distance_delta
        ).max(MIN_HALF_SIZE);

        self.viewport.x_min = x_center - x_half;
        self.viewport.x_max = x_center + x_half;

        self.viewport.y_min = y_center - y_half;
        self.viewport.y_max = y_center + y_half;
    }

    pub fn pan(&mut self, x_pan: f64, y_pan: f64) {
        self.viewport.x_min += x_pan;
        self.viewport.x_max += x_pan;
        self.viewport.y_min += y_pan;
        self.viewport.y_max += y_pan;
    }
}

impl Widget for PlotWidget2d {
    fn render(&self, target: &mut Buffer) {
        self.renderer.render(
            &self.points,
            &self.viewport,
            target,
        );
    }

    fn handle_key(&mut self, key: KeyEvent) -> KeyResult {
        match key.code {
            // -------------------------
            // Exit
            // -------------------------
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('x') => KeyResult::Exit,

            // -------------------------
            // Pan
            // -------------------------
            KeyCode::Left | KeyCode::Char('a') => {
                self.pan(-1.0, 0.0);
                KeyResult::Changed
            }

            KeyCode::Right | KeyCode::Char('d') => {
                self.pan(1.0, 0.0);
                KeyResult::Changed
            }

            KeyCode::Up | KeyCode::Char('w') => {
                self.pan(0.0, 1.0);
                KeyResult::Changed
            }

            KeyCode::Down | KeyCode::Char('s') => {
                self.pan(0.0, -1.0);
                KeyResult::Changed
            }

            // -------------------------
            // Zoom
            // -------------------------
            KeyCode::Char('e') => {
                self.zoom(-1.0);
                KeyResult::Changed
            }

            KeyCode::Char('q') => {
                self.zoom(1.0);
                KeyResult::Changed
            }

            _ => KeyResult::Ignored,
        }
    }
}


/**
 * Widget for rendering typed commands in bottom Pane
 */
pub struct CommandWidget {
    pub text: String,
}

impl Widget for CommandWidget {
    fn render(&self, target: &mut Buffer) {
        // Leave room for the pane's border
        if target.width() < 3 || target.height() < 3 {
            return;
        }

        // Border on both sides = width - 2
        let width = target.width() - 2;
        let line = self.text.lines().next().unwrap_or("");  // single line

        // Show the end of long commands while typing
        let skip = line.chars().count().saturating_sub(width);

        // Skip `skip` chars, take up to width chars, and enumerate
        for (x, ch) in line.chars().skip(skip).take(width).enumerate() {
            target.set(
                (x + 1) as isize,
                1,
                Cell::new(ch),
            )
        }
    }

    fn handle_key(&mut self, _: KeyEvent) -> KeyResult {
        KeyResult::Ignored
    }
}