use crossterm::event::{KeyCode, KeyEvent};

use super::{
    Animation,
    KeyResult, 
    Rect, 
    Widget, 
};
use crate::{
    geometry::{Mesh, Point, sample_curve_at, sample_surface_at},
    math::{OrthographicProjection, Projection, Transform},
    parsing::TokenNode,
    rendering::{
        draw_border, draw_text, 
        Buffer, 
        Camera, 
        CameraOrbit,
        Color,
        PlotViewport2d,
        PlotViewport3d,
        PlotRenderer2d,
        PlotRenderer3d,
    },
};


/**
 * Active Pane modes / state
 * - PlotMode: Plot in 2D or 3D
 * - InteractionMode: Enable / Disable plot movement
 * - FocusState: If Pane is currently focused 
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotMode {
    TwoD,
    ThreeD,
}

#[derive(Debug, Clone)]
pub struct LastPlot {
    pub expression: String,
    pub mode: PlotMode,
    pub animated: bool,
}

#[derive(Debug, PartialEq)]
pub enum InteractionMode {
    Static,
    Interactive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusState {
    InactivePane,
    ActivePane,
    InactiveCommand,
    ActiveCommand,
}

impl FocusState {
    pub fn border_color(self) -> Color {
        match self {
            Self::InactivePane => Color::Rgb(160, 160, 160),
            Self::ActivePane => Color::Rgb(160, 160, 0),
            Self::InactiveCommand => Color::Rgb(120, 120, 120),
            Self::ActiveCommand => Color::Rgb(240, 240, 240),
        }
    }
}


/**
 * Self-contained section of the terminal screen, owning its
 * own buffer, taking up a specified area, and containing a
 * Widget that provides its rendering and key handling
 */
pub struct Pane<W: Widget> {
    pub title: String,
    pub area: Rect,
    pub buffer: Buffer,
    pub mode: InteractionMode,
    pub focus: FocusState,
    pub widget: W,
}

impl<W: Widget> Pane<W> {
    pub fn new(area: Rect, mode: InteractionMode, focus: FocusState, widget: W) -> Self {
        Self {
            title: String::new(),
            area,
            buffer: Buffer::new(area.width, area.height),
            mode,
            focus,
            widget,
        }
    }

    /**
     * Handle key event depending on Pane's Widget 
     * - e.g., 2D vs 3D commands
     */
    pub fn handle_key(&mut self, key: KeyEvent) -> KeyResult {
        match self.mode {
            InteractionMode::Static => KeyResult::Ignored,
            InteractionMode::Interactive => {
                self.widget.handle_key(key)
            }
        }
    }

    /**
     * Render into Buffer using Widget.render() function,
     * then place the buffer into the target frame at the
     * Pane's placement Rect
     */
    pub fn render_into(&mut self, frame: &mut Buffer) {
        self.buffer.clear();

        if self.area.width == 0 || self.area.height == 0 {
            return;
        }

        self.widget.render(&mut self.buffer);

        // Draw border around pane
        draw_border(&mut self.buffer, self.focus.border_color(), true);

        // Write title at top-left of pane
        // - Drawn at (1, 0) pane-local coordinates,
        //   buffer.blit() handles the screen offset afterward
        draw_text(
            &mut self.buffer,
            2,
            0,
            &self.title,
            Color::Rgb(255, 255, 255),
        );

        frame.blit(&mut self.buffer, self.area.x, self.area.y);
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn set_area(&mut self, area: Rect) {
        if self.buffer.width() != area.width || self.buffer.height() != area.height 
        {
            self.buffer = Buffer::new(area.width, area.height);
        }
        self.area = area;
    }

    pub fn set_focus(&mut self, focus: FocusState) {
        self.focus = focus;
    }

    pub fn set_widget(&mut self, widget: W) {
        self.widget = widget;
    }
}


/**
 * Replacable data for the currently displayed plot,
 * containing AST expression tree, points or surface,
 * and animation settings
 */
pub enum PlotContent {
    Empty,
    TwoD(CurvePlot),
    ThreeD(SurfacePlot),
}

impl PlotContent {
    /**
     * Give a mutable reference to the Animation stored
     * inside the Widget, None if empty pane or static plot
     */
    pub fn animation_mut(&mut self) -> Option<&mut Animation> {
        match self {
            Self::TwoD(data) => data.animation.as_mut(),
            Self::ThreeD(data) => data.animation.as_mut(),
            Self::Empty => None,
        }
    }
}


/**
 * Persistent state belonging to one plot pane
 * - 2D / 3D active plotting mode & settings
 * - Last plot displayed
 * - Current Pane plot content
 */
pub struct PlotState {
    pub active_plot_mode: PlotMode,
    pub view_2d: PlotView2d,
    pub view_3d: PlotView3d,
    pub last_plot: Option<LastPlot>,
    pub content: PlotContent,
}

impl PlotState {
    /**
     * Compute new 2D f(x) values or 3D f(x, y) values
     *  for the current set of samples
     */
    pub fn resample(&mut self) {
        match &mut self.content {
            PlotContent::TwoD(data) => data.resample(&self.view_2d),
            PlotContent::ThreeD(data) => data.resample(),
            PlotContent::Empty => {}
        }
    }
}

impl Widget for PlotState {
    /**
     * Render 2D curve or 3D surface onto the target
     * buffer given the Pane's view settings
     */
    fn render(&self, target: &mut Buffer) {
        match &self.content {
            PlotContent::TwoD(curve) => {
                curve.render(&self.view_2d, target);
            }
            PlotContent::ThreeD(surface) => {
                surface.render(&self.view_3d, target);
            }
            PlotContent::Empty => {}
        }
    }

    /**
     * Handle key inputs depending on 2D/3D plot mode
     * - 2D changes need resampling, as zooming or panning
     *   will change the viewport and require new Y-values
     * - 3D changes only change the camera, just needs a 
     *   redraw after this function
     */
    fn handle_key(&mut self, key: KeyEvent) -> KeyResult {
        match &mut self.content {
            PlotContent::TwoD(curve) => {
                let result = self.view_2d.handle_key(key);

                if result == KeyResult::Changed {
                    curve.resample(&self.view_2d);
                }

                result
            }
            PlotContent::ThreeD(_) => {
                self.view_3d.handle_key(key)
            }
            PlotContent::Empty => KeyResult::Ignored,
        }
    }

    /**
     * If animation is enabled, update animation state
     * given the elapsed time
     * 
     * - Returns true to request a redraw, false if not animating
     */
    fn update(&mut self, delta_s: f64) -> bool {
        // No redraw if static or paused
        let Some(animation) = self.content.animation_mut() else {
            return false;
        };

        if !animation.playing || animation.speed == 0.0 {
            return false;
        }

        // Advance (t) by animation speed & elapsed time
        let limit = animation.period;

        animation.phase = (
            animation.phase + delta_s * animation.speed
        ).rem_euclid(2.0 * limit);

        animation.time = if animation.phase <= limit {
            animation.phase
        } else {
            2.0 * limit - animation.phase
        };

        self.resample();

        true
    }
}

impl Default for PlotState {
    fn default() -> Self {
        Self {
            active_plot_mode: PlotMode::TwoD,
            view_2d: PlotView2d::default(),
            view_3d: PlotView3d::default(),
            last_plot: None,
            content: PlotContent::Empty,
        }
    }
}


/**
 * Maintains information on how a 2D curve is viewed
 * and sampled, and how it is moved in the viewport
 */
pub struct PlotView2d {
    pub viewport: PlotViewport2d,
    pub renderer: PlotRenderer2d,
    pub samples: usize,
}

impl PlotView2d {
    pub fn handle_key(&mut self, key: KeyEvent) -> KeyResult {
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

impl Default for PlotView2d {
    fn default() -> Self {
        Self {
            viewport: PlotViewport2d::default(),
            renderer: PlotRenderer2d::default(),
            samples: 100,
        }
    }
}


pub struct CurvePlot {
    pub expression: TokenNode,
    pub points: Vec<Point>,
    pub animation: Option<Animation>,
}

impl CurvePlot {
    pub fn render(
        &self, 
        view: &PlotView2d, 
        target: &mut Buffer,
    ) {
        view.renderer.render(
            &self.points,
            &view.viewport,
            target,
        );
    } 

    pub fn resample(&mut self, view: &PlotView2d) {
        let time = self.animation
            .as_ref()
            .map_or(0.0, |animation| animation.time);

        sample_curve_at(
            &self.expression,
            view.viewport.x_min,
            view.viewport.x_max,
            view.samples,
            time,
            &mut self.points,
        )
    }
}


/**
 * Maintains information on how a 3D surface is viewed
 * and sampled, and how it is projected, moved, and rotated
 * in the viewport with respect to the camera
 */
pub struct PlotView3d {
    pub viewport: PlotViewport3d,
    pub renderer: PlotRenderer3d,
    pub camera: Camera,
    pub transform: Transform,
    pub samples: (usize, usize),
}

impl PlotView3d {
    pub fn handle_key(&mut self, key: KeyEvent) -> KeyResult {
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

impl Default for PlotView3d {
    fn default() -> Self {
        Self {
            viewport: PlotViewport3d::default(),
            renderer: PlotRenderer3d::default(),
            camera: Camera::default(),
            transform: Transform::default(),
            samples: (20, 20),
        }
    }
}


pub struct SurfacePlot {
    pub expression: TokenNode,
    pub mesh: Mesh,
    pub animation: Option<Animation>,
}

impl SurfacePlot {
    pub fn render(
        &self, 
        view: &PlotView3d, 
        target: &mut Buffer,
    ) {
        view.renderer.render(
            &self.mesh,
            view.transform,
            &view.viewport,
            &view.camera,
            target,
        );
    }

    pub fn resample(&mut self) {
        let time = self.animation
            .as_ref()
            .map_or(0.0, |animation| animation.time);

        sample_surface_at(
            &self.expression, 
            time, 
            &mut self.mesh,
        );
    }

    /**
     * Rebuild the mesh (x, y) grid then evaluate z = f(x, y)
     * for each sample pair
     */
    pub fn rebuild_mesh(&mut self, view: &PlotView3d) {
        let viewport = &view.viewport;
        let (x_samples, y_samples) = view.samples;

        self.mesh = Mesh::surface(
            viewport.x_min,
            viewport.x_max,
            viewport.y_min,
            viewport.y_max,
            x_samples,
            y_samples,
            |_, _| 0.0,     // Set all Z to 0 as a placeholder
        );

        self.resample();
    }
}