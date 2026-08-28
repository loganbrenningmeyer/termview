use meval;

use super::Command;
use crate::{
    geometry::{Mesh, Object, Point}, 
    math::sin(sqrt(x*x + y*y)) / sqrt(x*x + y*y)Transform,
    rendering::{
        Buffer, 
        Camera, 
        Plot3dRenderer, 
        PlotRenderer, 
        PlotViewport, 
        PlotViewport3d, 
    }
};


pub enum SessionOutput {
    None,
    Plot2d(Vec<Point>),
    Plot3d(Object),
    Message(String),
}

enum LastPlot {
    Plot2d(String),
    Plot3d(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlotMode {
    TwoD,
    ThreeD,
}

pub struct Session {
    active_plot_mode: PlotMode,

    viewport_2d: PlotViewport,
    renderer_2d: PlotRenderer,
    samples_2d: usize,

    viewport_3d: PlotViewport3d,
    renderer_3d: Plot3dRenderer,
    camera_3d: Camera,
    transform_3d: Transform,
    samples_3d: (usize, usize),

    last_plot: Option<LastPlot>,
}

impl Session {
    pub fn execute(&mut self, command: Command) -> Result<SessionOutput, String> {
        match command {
            Command::Plot(expression) => {
                match self.active_plot_mode {
                    PlotMode::TwoD => {
                        let points = self.sample_expression(&expression)?;
                        self.last_plot = Some(LastPlot::Plot2d(expression));
                        Ok(SessionOutput::Plot2d(points))
                    }
                    PlotMode::ThreeD => {
                        let surface = self.sample_surface(&expression)?;
                        self.last_plot = Some(LastPlot::Plot3d(expression));
                        Ok(SessionOutput::Plot3d(surface))
                    }
                }
            }

            Command::Plot3d(expression) => {
                self.active_plot_mode = PlotMode::ThreeD;
                let surface = self.sample_surface(&expression)?;
                self.last_plot = Some(LastPlot::Plot3d(expression));

                Ok(SessionOutput::Plot3d(surface))
            }

            Command::Replot => {
                match &self.last_plot {
                    Some(LastPlot::Plot2d(expression)) => {
                        let points = self.sample_expression(&expression)?;
                        Ok(SessionOutput::Plot2d(points))
                    }

                    Some(LastPlot::Plot3d(expression)) => {
                        let surface = self.sample_surface(&expression)?;
                        Ok(SessionOutput::Plot3d(surface))
                    }

                    _ => Ok(SessionOutput::Message(
                        "No previous plot".into()
                    ))
                }
            }

            Command::SetDimension(dimension) => {
                self.active_plot_mode = match dimension {
                    2 => PlotMode::TwoD,
                    3 => PlotMode::ThreeD,
                    _ => return Err("plot dimension must be 2 or 3".into()),
                };

                Ok(SessionOutput::None)
            }

            Command::SetView {
                x_min,
                x_max,
                y_min,
                y_max,
                z_bounds,
            } => {
                if x_min >= x_max {
                    return Err("x_min must be less than x_max".into());
                }

                if y_min >= y_max {
                    return Err("y_min must be less than y_max".into());
                }

                match self.active_plot_mode {
                    PlotMode::TwoD => {
                        if z_bounds.is_some() {
                            return Err("2D view expects x and y bounds only".into());
                        }

                        self.viewport_2d = PlotViewport {
                            x_min,
                            x_max,
                            y_min,
                            y_max,
                        };
                    }
                    PlotMode::ThreeD => {
                        let Some((z_min, z_max)) = z_bounds else {
                            return Err("3D view also requires z_min and z_max".into());
                        };

                        if z_min >= z_max {
                            return Err("z_min must be less than z_max".into());
                        }

                        self.viewport_3d = PlotViewport3d {
                            x_min,
                            x_max,
                            y_min,
                            y_max,
                            z_min,
                            z_max,
                        };
                    }
                }

                Ok(SessionOutput::None)
            }

            Command::SetSamples(samples) => {
                if samples < 2 {
                    return Err("samples must be at least 2".into());
                }

                match self.active_plot_mode {
                    PlotMode::TwoD => self.samples_2d = samples,
                    PlotMode::ThreeD => self.samples_3d = (samples, samples),
                }
                Ok(SessionOutput::None)
            }

            Command::ShowAxes(show) => {
                match self.active_plot_mode {
                    PlotMode::TwoD => self.renderer_2d.show_axes = show,
                    PlotMode::ThreeD => self.renderer_3d.show_axes = show,
                }
                Ok(SessionOutput::None)
            }
            
            Command::ShowTicks(show) => {
                match self.active_plot_mode {
                    PlotMode::TwoD => self.renderer_2d.show_ticks = show,
                    PlotMode::ThreeD => {
                        self.renderer_3d.axes_renderer.show_ticks = show;
                        self.renderer_3d.axes_renderer.show_labels = show;
                    }
                }
                Ok(SessionOutput::None)
            }

            Command::ShowBorder(show) => {
                match self.active_plot_mode {
                    PlotMode::TwoD => self.renderer_2d.show_border = show,
                    PlotMode::ThreeD => self.renderer_3d.show_border = show,
                }
                Ok(SessionOutput::None)
            }

            Command::Show => {
                Ok(SessionOutput::Message(
                    self.settings_text()
                ))
            }

            Command::Help => {
                Ok(SessionOutput::Message(
                    Self::help_text()
                ))
            }

            Command::Quit => {
                Ok(SessionOutput::None)
            }
        }
    }

    /**
     * Render (x, f(x)) points onto buffer using viewport/renderer settings
     */
    pub fn render_2d(&self, points: &[Point], buffer: &mut Buffer) {
        self.renderer_2d.render(
            points,
            &self.viewport_2d,
            buffer,
        );
    }

    /**
     * Render (x, y, z) points onto buffer using viewport/renderer settings
     */
    pub fn render_3d(&self, surface: &Object, buffer: &mut Buffer) {
        self.renderer_3d.render(
            surface, 
            &self.viewport_3d,
            &self.camera_3d,
            buffer,
        );
    }

    /**
     * Parses z = f(x, y) function string and returns (x, y, z)
     * points sampled over surface
     */
    fn sample_surface(
        &self,
        expression: &str,
    ) -> Result<Object, String> {
        // Parse expression w/ meval
        let expr: meval::Expr = expression
            .parse()
            .map_err(|error| {
                format!("invalid expression `{expression}`: {error}")
            })?;

        let function = expr
            .bind2("x", "y")
            .map_err(|error| {
                format!("could not bind variable `x`: {error}")
            })?;

        let viewport = &self.viewport_3d;
        let (x_samples, y_samples) = self.samples_3d;

        let mesh = Mesh::surface(
            viewport.x_min,
            viewport.x_max,
            viewport.y_min,
            viewport.y_max,
            x_samples,
            y_samples,
            function,
        );

        Ok(Object::new(mesh, self.transform_3d))
    }

    /**
     * Parses f(x) expression string and returns (x, f(x))
     * points sampled over viewport range
     */
    fn sample_expression(&self, expression: &str) -> Result<Vec<Point>, String> {
        // Parse expression w/ meval
        let expr: meval::Expr = expression
            .parse()
            .map_err(|error| {
                format!("invalid expression `{expression}`: {error}")
            })?;

        let function = expr
            .bind("x")
            .map_err(|error| {
                format!("could not bind variable `x`: {error}")
            })?;

        // Compute (x, f(x)) over x-samples in viewport range
        let points = (0..self.samples_2d)
            .filter_map(|i| {
                let t = i as f64 / (self.samples_2d - 1) as f64;
                let x = self.viewport_2d.x_min
                    + t * (self.viewport_2d.x_max - self.viewport_2d.x_min);
                let y = function(x);

                y.is_finite().then_some(Point::new(x, y))
            })
            .collect();

        Ok(points)
    }

    /**
     * Print current PlotViewport / PlotRenderer settings
     */
    fn settings_text(&self) -> String {
        let last_plot = match &self.last_plot {
            Some(LastPlot::Plot2d(expression)) => format!("2D: {expression}"),
            Some(LastPlot::Plot3d(expression)) => format!("3D: {expression}"),
            None => "<none>".to_string(),
        };

        let active_settings = match self.active_plot_mode {
            PlotMode::TwoD => format!(
            r#"
Current settings
  Dimension     2
  View          ({}, {}) to ({}, {})
  Samples       {}
  Axes          {}
  Ticks         {}
  Border        {}
  Last plot     {}
"#,
            self.viewport_2d.x_min,
            self.viewport_2d.y_min,
            self.viewport_2d.x_max,
            self.viewport_2d.y_max,
            self.samples_2d,
            on_off(self.renderer_2d.show_axes),
            on_off(self.renderer_2d.show_ticks),
            on_off(self.renderer_2d.show_border),
            last_plot,
            ),
            PlotMode::ThreeD => format!(
            r#"
Current settings
  Dimension     3
  View          ({}, {}, {}) to ({}, {}, {})
  Samples       {} x {}
  Axes          {}
  Ticks         {}
  Border        {}
  Last plot     {}
"#,
                self.viewport_3d.x_min,
                self.viewport_3d.y_min,
                self.viewport_3d.z_min,
                self.viewport_3d.x_max,
                self.viewport_3d.y_max,
                self.viewport_3d.z_max,
                self.samples_3d.0,
                self.samples_3d.1,
                on_off(self.renderer_3d.show_axes),
                on_off(self.renderer_3d.axes_renderer.show_ticks),
                on_off(self.renderer_3d.show_border),
                last_plot,
            ),
        };

        active_settings
    }

    /**
     * Print list of available commands
     */
    fn help_text() -> String {
        return 
            r#"
Termview commands
  plot <expression>                             Plot using the active dimension
  plot3d <expression>                           Plot a function of x and y, and switch to 3D
  replot                                        Plot the last expression again

  set dim <2|3>                                 Set the active plot dimension
  set view <xmin> <xmax> <ymin> <ymax>          Set the visible coordinate range
  set view (<xmin>, <xmax>) (<ymin>, <ymax>)
  set view <xmin> <xmax> <ymin> <ymax> <zmin> <zmax>

  set samples <count>                           Set the number of sampled points

  show                                          Display the current settings
  show axes <true|false|1|0>                    Show or hide the axes
  show ticks <true|false|1|0>                   Show or hide ticks and labels
  show border <true|false|1|0>                  Show or hide the plot border

  help                                          Display this help
  quit | exit                                   Exit termview

Examples
  set dim 2
  set view (-4, 4) (-4, 4)
  set samples 800
  plot sin(x)

  set dim 3
  set view (-3, 3) (-3, 3) (-2, 2)
  set samples 20
  plot sin(x) * cos(y)
"#.to_string();
    }
}

fn on_off(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}

impl Default for Session {
    fn default() -> Self {
        Self {
            active_plot_mode: PlotMode::TwoD,

            // 2D
            viewport_2d: PlotViewport {
                x_min: -10.0,
                x_max: 10.0,
                y_min: -10.0,
                y_max: 10.0,
            },
            renderer_2d: PlotRenderer {
                pad_width: 5,
                pad_height: 2,
                ..Default::default()
            },
            samples_2d: 500,
            
            // 3D
            viewport_3d: PlotViewport3d::default(),
            renderer_3d: Plot3dRenderer::default(),
            camera_3d: Camera::default(),
            transform_3d: Transform::default(),
            samples_3d: (10, 10),

            last_plot: None,
        }
    }
}
