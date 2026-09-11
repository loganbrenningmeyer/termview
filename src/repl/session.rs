use std::collections::HashMap;

use super::Command;
use crate::{
    geometry::{Mesh, Object, Point}, 
    math::Transform, 
    parsing::{Parser, Tokenizer},
    rendering::{
        Buffer, 
        Camera, 
        PlotRenderer2d, 
        PlotRenderer3d, 
        PlotViewport2d, 
        PlotViewport3d,
    },
    ui::{PlotWidget2d, PlotWidget3d},
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

    viewport_2d: PlotViewport2d,
    renderer_2d: PlotRenderer2d,
    samples_2d: usize,

    viewport_3d: PlotViewport3d,
    renderer_3d: PlotRenderer3d,
    camera_3d: Camera,
    transform_3d: Transform,
    samples_3d: (usize, usize),

    last_plots: HashMap<usize, LastPlot>,
}

impl Session {
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
     * Create a PlotWidget2d given the 2D plot Points
     */
    pub fn make_widget_2d(&self, points: Vec<Point>) -> PlotWidget2d {
        PlotWidget2d {
            points,
            viewport: self.viewport_2d,
            renderer: self.renderer_2d.clone(),
        }
    }

    /**
     * Create a PlotWidget3d given the 3D surface
     */
    pub fn make_widget_3d(&self, surface: Object) -> PlotWidget3d {
        PlotWidget3d {
            surface,
            camera: self.camera_3d.clone(),
            viewport: self.viewport_3d,
            renderer: self.renderer_3d.clone(),
        }
    }

    /**
     * Parses z = f(x, y) function string and returns (x, y, z)
     * points sampled over surface
     */
    fn sample_surface(
        &self,
        expression: &str,
    ) -> Result<Object, String> {
        // Tokenize expression
        let mut tokenizer = Tokenizer::new(expression);
        let tokens = tokenizer.tokenize()?;

        // Parse tokens into abstract syntax tree
        let mut parser = Parser::new(tokens);
        let tree = parser.parse_expression(0)?;

        // Compute (x, f(x)) over x-samples in viewport range
        let mut vars = HashMap::from([
            ("x".to_string(), 0.0),
            ("y".to_string(), 0.0),
        ]);

        // Define Fn(f64, f64) -> f64 for Mesh::surface()
        let function = |x: f64, y: f64| {
            *vars.get_mut("x").unwrap() = x;
            *vars.get_mut("y").unwrap() = y;
            tree.evaluate(&vars)
        };

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
        // Tokenize expression
        let mut tokenizer = Tokenizer::new(expression);
        let tokens = tokenizer.tokenize()?;

        // Parse tokens into abstract syntax tree
        let mut parser = Parser::new(tokens);
        let tree = parser.parse_expression(0)?;

        // Compute (x, f(x)) over x-samples in viewport range
        let mut vars = HashMap::from([("x".to_string(), 0.0)]);

        let points = (0..self.samples_2d)
            .filter_map(|i| {
                let t = i as f64 / (self.samples_2d - 1) as f64;
                let x = self.viewport_2d.x_min
                    + t * (self.viewport_2d.x_max - self.viewport_2d.x_min);
                
                *vars.get_mut("x").unwrap() = x;
                let y = tree.evaluate(&vars);

                y.is_finite().then_some(Point::new(x, y))
            })
            .collect();

        Ok(points)
    }

    pub fn execute(
        &mut self, 
        pane_id: usize,
        command: Command
    ) -> Result<SessionOutput, String> {
        match command {
            Command::Plot(expression) => {
                match self.active_plot_mode {
                    PlotMode::TwoD => {
                        let points = match self.sample_expression(&expression) {
                            Ok(points) => points,
                            Err(error) => return Ok(SessionOutput::Message(error)),
                        };

                        self.last_plots.insert(
                            pane_id,
                            LastPlot::Plot2d(expression),
                        );
                        Ok(SessionOutput::Plot2d(points))
                    }
                    PlotMode::ThreeD => {
                        let surface = match self.sample_surface(&expression) {
                            Ok(surface) => surface,
                            Err(error) => return Ok(SessionOutput::Message(error)),
                        };

                        self.last_plots.insert(
                            pane_id,
                            LastPlot::Plot3d(expression),
                        );
                        Ok(SessionOutput::Plot3d(surface))
                    }
                }
            }

            Command::Plot3d(expression) => {
                self.active_plot_mode = PlotMode::ThreeD;
                let surface = self.sample_surface(&expression)?;
                self.last_plots.insert(
                    pane_id,
                    LastPlot::Plot3d(expression),
                );

                Ok(SessionOutput::Plot3d(surface))
            }

            Command::Replot => {
                match &self.last_plots.get(&pane_id) {
                    Some(LastPlot::Plot2d(expression)) => {
                        let points = self.sample_expression(&expression)?;
                        Ok(SessionOutput::Plot2d(points))
                    }

                    Some(LastPlot::Plot3d(expression)) => {
                        let surface = self.sample_surface(&expression)?;
                        Ok(SessionOutput::Plot3d(surface))
                    }

                    _ => Ok(SessionOutput::Message(
                        "This pane has no previous plot".into()
                    ))
                }
            }

            Command::SetDimension(dimension) => {
                self.active_plot_mode = match dimension {
                    2 => PlotMode::TwoD,
                    3 => PlotMode::ThreeD,
                    _ => return Ok(SessionOutput::Message("plot dimension must be 2 or 3".into())),
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
                    return Ok(SessionOutput::Message("x_min must be less than x_max".into()));
                }

                if y_min >= y_max {
                    return Ok(SessionOutput::Message("y_min must be less than y_max".into()));
                }

                match self.active_plot_mode {
                    PlotMode::TwoD => {
                        if z_bounds.is_some() {
                            return Ok(SessionOutput::Message("2D view expects x and y bounds only".into()));
                        }

                        self.viewport_2d = PlotViewport2d {
                            x_min,
                            x_max,
                            y_min,
                            y_max,
                        };
                    }
                    PlotMode::ThreeD => {
                        let Some((z_min, z_max)) = z_bounds else {
                            return Ok(SessionOutput::Message("3D view also requires z_min and z_max".into()));
                        };

                        if z_min >= z_max {
                            return Ok(SessionOutput::Message("z_min must be less than z_max".into()));
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

            Command::SetProjection(proj) => {
                self.camera_3d.projection = proj;

                Ok(SessionOutput::None)
            }

            Command::SetSamples(samples) => {
                if samples < 2 {
                    return Ok(SessionOutput::Message("samples must be at least 2".into()));
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

            Command::Config => {
                Ok(SessionOutput::Message(
                    self.config_text()
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
     * Forget pane's plot history upon removal
     */
    pub fn forget_pane(&mut self, pane_id: usize) {
        self.last_plots.remove(&pane_id);
    }

    /**
     * Print current PlotViewport / PlotRenderer configuration
     */
    fn config_text(&self) -> String {
        let active_settings = match self.active_plot_mode {
            PlotMode::TwoD => format!(
            r#"
Current settings
  Dimension          2D
  View               ({}, {}) to ({}, {})
  Projection (3D)    {}
  Samples            {}
  Axes               {}
  Ticks              {}
  Border             {}
"#,
            self.viewport_2d.x_min,
            self.viewport_2d.y_min,
            self.viewport_2d.x_max,
            self.viewport_2d.y_max,
            self.camera_3d.projection,
            self.samples_2d,
            on_off(self.renderer_2d.show_axes),
            on_off(self.renderer_2d.show_ticks),
            on_off(self.renderer_2d.show_border),
            ),
            PlotMode::ThreeD => format!(
            r#"
Current settings
  Dimension          3D
  View               ({}, {}, {}) to ({}, {}, {})
  Projection (3D)    {}
  Samples            {} x {}
  Axes               {}
  Ticks              {}
  Border             {}
"#,
                self.viewport_3d.x_min,
                self.viewport_3d.y_min,
                self.viewport_3d.z_min,
                self.viewport_3d.x_max,
                self.viewport_3d.y_max,
                self.viewport_3d.z_max,
                self.camera_3d.projection,
                self.samples_3d.0,
                self.samples_3d.1,
                on_off(self.renderer_3d.show_axes),
                on_off(self.renderer_3d.axes_renderer.show_ticks),
                on_off(self.renderer_3d.show_border),
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
==============================
      Termview commands 
==============================

  plot <expression>                             Plot using the active dimension
  plot3d <expression>                           Plot a function of x and y, and switch to 3D
  replot                                        Plot the last expression again

  set dim <2|3>                                 Set the active plot dimension

  set view <xmin> <xmax> <ymin> <ymax>          Set the visible coordinate range
  set view (<xmin>, <xmax>) (<ymin>, <ymax>)
  set view <xmin> <xmax> <ymin> <ymax> <zmin> <zmax>

  set proj <p|o>                                Set the 3D projection method 
  set proj <perspective|orthographic>

  set samples <count>                           Set the number of sampled points

  show axes <true|false|1|0>                    Show or hide the axes
  show ticks <true|false|1|0>                   Show or hide ticks and labels
  show border <true|false|1|0>                  Show or hide the plot border
  
  [C] | cfg | config                            Display the current plotting configuration
  [H] | help                                    Display this help

  [Q] | quit | exit                             Exit termview

==============================
        Plot Controls 
==============================

  [↑] [←] [↓] [→]                               Rotate camera [UP] [DOWN] [LEFT] [RIGHT]
  [W] [A] [S] [D] 

  [Q] [E]                                       Zoom [in] [out]

  [R]                                           Reset camera view

  [X] | [Esc] | [Enter]                         Exit plot                                     

=========================
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
            viewport_2d: PlotViewport2d::default(),
            renderer_2d: PlotRenderer2d::default(),
            samples_2d: 500,
            
            // 3D
            viewport_3d: PlotViewport3d::default(),
            renderer_3d: PlotRenderer3d::default(),
            camera_3d: Camera::default(),
            transform_3d: Transform::default(),
            samples_3d: (10, 10),

            last_plots: HashMap::new(),
        }
    }
}
