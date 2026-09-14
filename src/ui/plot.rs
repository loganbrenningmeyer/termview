use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    geometry::{Mesh, Point, Vertex, sample_curve_at, sample_surface_at},
    math::{Projection, OrthographicProjection, Transform, Vec3},
    parsing::TokenNode,
    rendering::{
        Buffer, 
        Camera, 
        CameraOrbit,
        PlotViewport2d,
        PlotViewport3d,
        PlotRenderer2d,
        PlotRenderer3d,
    },
};
use super::{
    Animation,
    KeyResult, 
    PaneController, 
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


/**
 * Holds Pane plot state and drives Pane rendering,
 * input, animation updates, and resampling functionality
 * - 2D / 3D active plotting mode & settings
 * - Last plot displayed
 * - Current Pane plot content
 */
pub struct PlotController {
    pub active_plot_mode: PlotMode,
    pub view_2d: PlotView2d,
    pub view_3d: PlotView3d,
    pub last_plot: Option<LastPlot>,
    pub content: PlotContent,
}

impl PlotController {
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

impl PaneController for PlotController {
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
        if key.code == KeyCode::Char(' ') {
            let Some(animation) = self.content.animation_mut() else {
                return KeyResult::Ignored;
            };

            animation.playing = !animation.playing;
            return KeyResult::Changed;
        }

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

    /**
     * Config text block for rendering in Pane
     */
    fn config_text(&self) -> Vec<String> {
        match self.active_plot_mode {
            PlotMode::TwoD => vec![
                "Dimension      2D".to_string(),
                format!(
                    "View           X: ({:.2}, {:.2})",
                    self.view_2d.viewport.x_min, self.view_2d.viewport.x_max,
                ),
                format!(
                    "               Y: ({:.2}, {:.2})",
                    self.view_2d.viewport.y_min, self.view_2d.viewport.y_max,
                ),
                format!("Samples        {}", self.view_2d.samples),
                format!("Axes           {}", self.on_off(self.view_2d.renderer.show_axes)),
                format!("Ticks          {}", self.on_off(self.view_2d.renderer.show_ticks)),
            ],
            PlotMode::ThreeD => vec![
                "Dimension      3D".to_string(),
                format!(
                    "View           X: ({:.2}, {:.2})",
                    self.view_3d.viewport.x_min, self.view_3d.viewport.x_max,
                ),
                format!(
                    "               Y: ({:.2}, {:.2})",
                    self.view_3d.viewport.y_min, self.view_3d.viewport.y_max,
                ),
                format!(
                    "               Z: ({:.2}, {:.2})",
                    self.view_3d.viewport.z_min, self.view_3d.viewport.z_max,
                ),
                format!("Samples        ({}, {})", self.view_3d.samples.0, self.view_3d.samples.1),
                format!("Axes           {}", self.on_off(self.view_3d.renderer.show_axes)),
                format!("Ticks          {}", self.on_off(self.view_3d.renderer.axes_renderer.show_ticks)),
            ]
        }
    }

    /**
     * Tag text for top-right label
     */
    fn tag_text(&self) -> String {
        // Animated: Show t-value
        let Some(animation) = self.content.animation() else {
            return String::new();
        };

        return format!("t={:.2}", animation.time);
    }

    /**
     * Label text for top-left label next to number
     */    
    fn label_text(&self) -> String {
        match self.active_plot_mode {
            PlotMode::TwoD => "2D".to_string(),
            PlotMode::ThreeD => "3D".to_string(),
        }
    }
}

impl Default for PlotController {
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
     * inside the current plot, None if empty pane or static plot
     */
    pub fn animation_mut(&mut self) -> Option<&mut Animation> {
        match self {
            Self::TwoD(data) => data.animation.as_mut(),
            Self::ThreeD(data) => data.animation.as_mut(),
            Self::Empty => None,
        }
    }

    /**
     * Give a reference to the Animation or None if empty/static
     */
    pub fn animation(&self) -> Option<&Animation> {
        match self {
            Self::TwoD(data) => data.animation.as_ref(),
            Self::ThreeD(data) => data.animation.as_ref(),
            Self::Empty => None,
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

    pub fn reset_view(&mut self) {
        self.viewport = PlotViewport2d::default();
    }

    /**
     * Fit viewport to x/y-values range
     */
    pub fn fit_view(&mut self, points: &[Point]) {
        // Determine x/y-value range
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;

        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for point in points {
            if point.y.is_finite() {
                min_y = min_y.min(point.y);
                max_y = max_y.max(point.y);

                min_x = min_x.min(point.x);
                max_x = max_x.max(point.x);
            }
        }

        // No valid samples: preserve the existing viewport.
        if !min_y.is_finite() || !max_y.is_finite()
            || !min_x.is_finite() || !max_x.is_finite() {
            return;
        }

        let fill = 0.90;

        // Shift x view if no x-values on either side
        let lower_x = self.viewport.x_min.max(min_x);
        let upper_x = self.viewport.x_max.min(max_x);

        if lower_x.is_finite() && upper_x.is_finite() && lower_x < upper_x {
            self.viewport.x_min = lower_x;
            self.viewport.x_max = upper_x;
        }

        // Set y-values range
        let center = min_y * 0.5 + max_y * 0.5;

        let half_height = if min_y == max_y {
            // Give constant functions a nonzero visible range.
            (center.abs() * 0.05).max(0.5)
        } else {
            (max_y * 0.5 - min_y * 0.5) / fill
        };

        let lower_y = center - half_height;
        let upper_y = center + half_height;

        if lower_y.is_finite() && upper_y.is_finite() && lower_y < upper_y {
            self.viewport.y_min = lower_y;
            self.viewport.y_max = upper_y;
        }
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

    pub fn fit_animation_view(
        &self,
        view: &mut PlotView2d,
        time_samples: usize,
    ) {
        // Static plots run normal fit_view
        let Some(animation) = &self.animation else {
            view.fit_view(&self.points);
            return;
        };

        if !animation.period.is_finite() || animation.period <= 0.0 {
            return;
        }

        let time_samples = time_samples.max(2);
        let x_samples = view.samples.max(2);

        let x_min = view.viewport.x_min;
        let x_max = view.viewport.x_max;

        let mut points = Vec::with_capacity(x_samples);

        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;

        for i in 0..time_samples {
            let time = animation.period
                * i as f64 / (time_samples - 1) as f64;

            sample_curve_at(
                &self.expression,
                x_min,
                x_max,
                x_samples,
                time,
                &mut points,
            );

            for point in &points {
                if point.y.is_finite() {
                    min_y = min_y.min(point.y);
                    max_y = max_y.max(point.y);

                    min_x = min_x.min(point.x);
                    max_x = max_x.max(point.x);
                }
            }
        }

        if !min_y.is_finite() || !max_y.is_finite() {
            return;
        }

        // Set bounds with min/max y_vals and full x-range
        let bounds = [
            Point::new(min_x, min_y),
            Point::new(max_x, max_y),
        ];

        view.fit_view(&bounds);
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

    /**
     * Adjust camera's azimuth / elevation to orbit around target
     */
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

    /**
     * Zoom the camera by increasing / decreasing its distance
     */
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

    /**
     * Reset camera orbit and projection method to their defaults
     */
    pub fn reset_camera(&mut self) {
        self.camera.projection = match &self.camera.projection {
            Projection::Perspective(_) => Projection::Perspective(Default::default()),
            Projection::Orthographic(_) => Projection::Orthographic(Default::default()),
        };
        self.camera.orbit = CameraOrbit::default();

        self.camera.update();
    }

    /**
     * Adjusts the camera to the viewport's bounds so that it 
     * fits the bounds properly in the terminal by default
     */
    pub fn fit_camera(&mut self, target: &Buffer, vertices: &[Vertex]) {
        // Set camera target to center of viewport box
        let center = Vec3::new(
            (self.viewport.x_min + self.viewport.x_max) / 2.0,
            (self.viewport.y_min + self.viewport.y_max) / 2.0,
            (self.viewport.z_min + self.viewport.z_max) / 2.0,
        );

        self.camera.orbit.target = center;
        self.camera.update();   // establish rotation quaternions from azimuth/elevation

        // Define inverse rotation quaternion from world -> camera coordinates
        let inverse_rotation = self.camera.rotation.inverse();

        // Collect the shared axis origin and the three axis endpoints.
        let v = &self.viewport;

        let points = [
            Vec3::new(v.x_min, v.y_min, v.z_min), // Shared starting point
            Vec3::new(v.x_max, v.y_min, v.z_min), // X endpoint
            Vec3::new(v.x_min, v.y_max, v.z_min), // Y endpoint
            Vec3::new(v.x_min, v.y_min, v.z_max), // Z endpoint
        ];

        // Include plotted points that leave the viewport box
        let surface_points = vertices.iter()
            .map(|v| v.position)
            .filter(|p| p.x.is_finite() && p.y.is_finite() && p.z.is_finite());

        let offsets: Vec<Vec3> = points.into_iter()
            .chain(surface_points)
            .map(|point| inverse_rotation.rotate_vec3(point - center))
            .collect();

        let aspect = target.display_aspect();
        let fill = 0.85;
        let depth_margin = 0.01;

        let mut distance: f64 = 0.0;
        let min_z = offsets.iter()
            .map(|q| q.z)
            .fold(f64::INFINITY, f64::min);

        match &mut self.camera.projection {
            Projection::Perspective(p) => {
                let tan_y = (p.fov_y.to_radians() / 2.0).tan();
                let tan_x = tan_y * aspect;

                for q in &offsets {
                    distance = distance
                        .max(q.z + q.x.abs() / (fill * tan_x))
                        .max(q.z + q.y.abs() / (fill * tan_y))
                        .max(q.z + p.near + depth_margin);
                }

                p.far = p.far.max(distance - min_z + depth_margin);
            }

            Projection::Orthographic(o) => {
                let mut min_x = f64::INFINITY;
                let mut max_x = f64::NEG_INFINITY;
                let mut min_y = f64::INFINITY;
                let mut max_y = f64::NEG_INFINITY;

                for q in &offsets {
                    min_x = min_x.min(q.x);
                    max_x = max_x.max(q.x);
                    min_y = min_y.min(q.y);
                    max_y = max_y.max(q.y);
                }

                let half_width = (max_x - min_x) * 0.5;
                let half_height = (max_y - min_y) * 0.5;

                o.size = (half_width / (fill * aspect))
                    .max(half_height / fill)
                    .max(0.005);

                // Center the projected axes by moving the target in the
                // camera's right/up plane, preserving all point depths.
                let target_offset = Vec3::new(
                    (min_x + max_x) * 0.5,
                    (min_y + max_y) * 0.5,
                    0.0,
                );
                self.camera.orbit.target = center
                    + self.camera.rotation.rotate_vec3(target_offset);

                // Depth doesn't change orthographic screen size, so fit depth
                // to a sphere around the target that holds for any orbit angle
                // - Covers the whole viewport box and surface points outside it
                let mut radius: f64 = 0.0;
                for x in [v.x_min, v.x_max] {
                    for y in [v.y_min, v.y_max] {
                        for z in [v.z_min, v.z_max] {
                            let q = inverse_rotation.rotate_vec3(
                                Vec3::new(x, y, z) - center,
                            );
                            radius = radius.max((q - target_offset).magnitude());
                        }
                    }
                }

                for q in &offsets {
                    radius = radius.max((*q - target_offset).magnitude());
                }

                distance = radius + o.near + depth_margin;
                o.far = o.far.max(distance + radius + depth_margin);
            }
        }

        self.camera.orbit.distance = distance;
        self.camera.update();
    }

    pub fn reset_view(&mut self) {
        self.viewport = PlotViewport3d::default();
    }

    /**
     * Fit viewport to x/y-values range
     */
    pub fn fit_view(&mut self, vertices: &[Vertex]) {
        // Determine x/y/z-value range
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;

        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        let mut min_z = f64::INFINITY;
        let mut max_z = f64::NEG_INFINITY;

        for v in vertices {
            if v.position.z.is_finite() {
                min_x = min_x.min(v.position.x);
                max_x = max_x.max(v.position.x);

                min_y = min_y.min(v.position.y);
                max_y = max_y.max(v.position.y);

                min_z = min_z.min(v.position.z);
                max_z = max_z.max(v.position.z);
            }
        }

        // No valid samples: preserve the existing viewport.
        if !min_y.is_finite() || !max_y.is_finite()
            || !min_x.is_finite() || !max_x.is_finite() 
            || !min_z.is_finite() || !max_z.is_finite() {
            return;
        }

        let fill = 0.60;

        // Shift x view if no x-values on either side
        let lower_x = self.viewport.x_min.max(min_x);
        let upper_x = self.viewport.x_max.min(max_x);

        if lower_x.is_finite() && upper_x.is_finite() && lower_x < upper_x {
            self.viewport.x_min = lower_x;
            self.viewport.x_max = upper_x;
        }

        // Shift y view if no x-values on either side
        let lower_y = self.viewport.y_min.max(min_y);
        let upper_y = self.viewport.y_max.min(max_y);

        if lower_y.is_finite() && upper_y.is_finite() && lower_y < upper_y {
            self.viewport.y_min = lower_y;
            self.viewport.y_max = upper_y;
        }

        // Set z-values range
        let center = min_z * 0.5 + max_z * 0.5;

        let half_height = if min_z == max_z {
            // Give constant functions a nonzero visible range
            (center.abs() * 0.05).max(0.5)
        } else {
            (max_z * 0.5 - min_z * 0.5) / fill
        };

        let lower_z = center - half_height;
        let upper_z = center + half_height;

        if lower_z.is_finite() && upper_z.is_finite() && lower_z < upper_z {
            self.viewport.z_min = lower_z;
            self.viewport.z_max = upper_z;
        }
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
