use std::collections::HashMap;

use super::Command;
use crate::{
    geometry::{Mesh, Object, Point, sample_curve_at, sample_surface_at}, 
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
    ui::{Animation, PaneContent, PlotWidget2d, PlotWidget3d},
};


pub enum SessionOutput {
    None,
    Message(String),
    Plot {
        title: String,
        content: PaneContent,
    },
}

#[derive(Debug, Clone)]
struct LastPlot {
    expression: String,
    mode: PlotMode,
    animated: bool,
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

    fn plot_for_pane(
        &mut self,
        pane_id: usize,
        expression: String,
        mode: PlotMode,
        animated: bool,
    ) -> Result<SessionOutput, String> {
        let output = match mode {
            PlotMode::TwoD => {
                self.build_plot_2d(expression.clone(), animated)?
            }
            PlotMode::ThreeD => {
                self.build_plot_3d(expression.clone(), animated)?
            }
        };

        // Replace history after successfully building the plot
        self.last_plots.insert(
            pane_id,
            LastPlot {
                expression,
                mode,
                animated,
            }
        );

        Ok(output)
    }

    /**
     * Tokenizes & parses expression, validates its variable
     * identifier nodes (x, t if animated), and computes y = f(x) samples to
     * build a Widget for PaneContent
     */
    fn build_plot_2d(
        &self,
        expression: String,
        animated: bool,
    ) -> Result<SessionOutput, String> {
        // Tokenize / parse expression into AST 
        let mut tokenizer = Tokenizer::new(&expression);
        let tokens = tokenizer.tokenize()?;

        let mut parser = Parser::new(tokens);
        let tree = parser.parse_expression(0)?;

        // Validate variable identifiers
        tree.validate_variables(2, animated)?;

        // Perform initial sample with t = 0
        // - No effect if not animated
        let mut points = Vec::with_capacity(self.samples_2d);

        sample_curve_at(
            &tree,
            self.viewport_2d.x_min,
            self.viewport_2d.x_max,
            self.samples_2d,
            0.0,
            &mut points,
        ); 

        // Define animation parameters if enabled
        let animation = if animated {
            Some(Animation {
                expression: tree,
                time: 0.0,
                speed: 1.0,
                period: 5.0,
                phase: 0.0,
                playing: true,
            }) 
        } else {
            None
        };

        // Create Widget for PaneContent
        let widget = PlotWidget2d::new(
            points,
            self.samples_2d,
            self.viewport_2d,
            self.renderer_2d.clone(),
            animation,
        );

        Ok(SessionOutput::Plot {
            title: format!(" y = {expression} "),
            content: PaneContent::Plot2d(widget),
        })
    }

    /**
     * Tokenizes & parses expression, validates its variable
     * identifier nodes (x, y, t if animated), and computes 
     * z = f(x, y) samples to build a Widget for PaneContent
     */
    fn build_plot_3d(
        &self,
        expression: String,
        animated: bool,
    ) -> Result<SessionOutput, String> {
        // Tokenize / parse expression into AST 
        let mut tokenizer = Tokenizer::new(&expression);
        let tokens = tokenizer.tokenize()?;

        let mut parser = Parser::new(tokens);
        let tree = parser.parse_expression(0)?;

        // Validate variable identifiers
        tree.validate_variables(3, animated)?;

        // Create placehold Mesh grid to compute samples
        // - Placeholder function z = 0, ensuring all finite vertices initially
        let viewport = &self.viewport_3d;
        let (x_samples, y_samples) = self.samples_3d;

        let mut mesh = Mesh::surface(
            viewport.x_min,
            viewport.x_max,
            viewport.y_min,
            viewport.y_max,
            x_samples,
            y_samples,
            |_: f64, _: f64| { 0.0 },
        );

        sample_surface_at(&tree, 0.0, &mut mesh);

        // Define animation parameters if enabled
        let animation = if animated {
            Some(Animation {
                expression: tree,
                time: 0.0,
                speed: 1.0,
                period: 5.0,
                phase: 0.0,
                playing: true,
            }) 
        } else {
            None
        };

        // Create Widget for PaneContent
        let widget = PlotWidget3d::new(
            Object::new(mesh, self.transform_3d),
            self.camera_3d.clone(),
            self.viewport_3d,
            self.renderer_3d.clone(),
            animation,
        );

        Ok(SessionOutput::Plot { 
            title: format!(" z = {expression} "), 
            content: PaneContent::Plot3d(widget) 
        })
    }

    pub fn execute(
        &mut self, 
        pane_id: usize,
        command: Command
    ) -> Result<SessionOutput, String> {
        match command {
            // -------------------------
            // Plot / Plot3d / Replot
            // -------------------------
            Command::Plot(expression) => {
                self.plot_for_pane(
                    pane_id,
                    expression,
                    self.active_plot_mode,
                    false,
                )
            }

            Command::Plot3d(expression) => {
                let output = self.plot_for_pane(
                    pane_id,
                    expression,
                    PlotMode::ThreeD,
                    false,
                )?;

                self.active_plot_mode = PlotMode::ThreeD;
                Ok(output)
            }

            Command::Replot => {
                let Some(last) = self.last_plots.get(&pane_id).cloned() else {
                    return Ok(SessionOutput::Message(
                        "This pane has no previous plot".into(),
                    ));
                };

                self.plot_for_pane(
                    pane_id,
                    last.expression,
                    last.mode,
                    last.animated,
                )
            }

            // -------------------------
            // Animate / Animate3d
            // -------------------------
            Command::Animate(expression) => {
                self.plot_for_pane(
                    pane_id,
                    expression,
                    self.active_plot_mode,
                    true,
                )
            }

            Command::Animate3d(expression) => {
                let output = self.plot_for_pane(
                    pane_id,
                    expression,
                    PlotMode::ThreeD,
                    true,
                )?;

                self.active_plot_mode = PlotMode::ThreeD;
                Ok(output)
            }

            // -------------------------
            // Animation: Pause, Resume, SetSpeed, SetTime
            // are handled in app.rs
            // -------------------------
            Command::Pause
            | Command::Resume
            | Command::SetSpeed(_)
            | Command::SetTime(_) => {
                Err("Animation controls must be handled by the active pane".into())
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
